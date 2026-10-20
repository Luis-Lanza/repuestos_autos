use rusqlite::{
    functions::FunctionFlags,
    types::{Value, ValueRef},
    Connection,
};

pub(super) fn register(connection: &Connection) -> rusqlite::Result<()> {
    // SQLite CAST accepts numeric prefixes; Rust parsing rejects partial values.
    connection.create_scalar_function(
        "repuestos_catalog_finite_number",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |context| {
            let number = match context.get_raw(0) {
                ValueRef::Real(value) => Some(value),
                ValueRef::Integer(value) => Some(value as f64),
                ValueRef::Text(value) => std::str::from_utf8(value)
                    .ok()
                    .and_then(|value| value.trim().parse::<f64>().ok()),
                _ => None,
            };
            Ok(number.filter(|value| value.is_finite()))
        },
    )?;
    connection.create_scalar_function(
        "repuestos_catalog_sku_prefix",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |context| {
            let sku = context.get::<String>(0)?.to_lowercase();
            let query = context.get::<String>(1)?.to_lowercase();
            let sku_tokens = sku
                .split(|character: char| !character.is_alphanumeric())
                .filter(|token| !token.is_empty())
                .collect::<Vec<_>>();
            let matches = query
                .split(|character: char| !character.is_alphanumeric())
                .filter(|token| !token.is_empty())
                .all(|term| sku_tokens.iter().any(|token| token.starts_with(term)));
            Ok(matches)
        },
    )
}

pub(super) struct SearchPredicate {
    pub clause: String,
    pub args: Vec<Value>,
}

pub(super) fn activity_clause(activity: &super::ProductActivityFilter) -> &'static str {
    match activity {
        super::ProductActivityFilter::Active => "p.active = 1 AND c.active = 1",
        super::ProductActivityFilter::Archived => "p.active = 0 AND c.active = 1",
        super::ProductActivityFilter::All => "c.active = 1",
    }
}

pub(super) fn browse_predicate(
    connection: &Connection,
    query: Option<&str>,
    category_id: Option<i64>,
    activity: &super::ProductActivityFilter,
) -> rusqlite::Result<SearchPredicate> {
    register(connection)?;
    Ok(query
        .map(|query| predicate(connection, query, category_id, activity))
        .transpose()?
        .flatten()
        .unwrap_or_else(|| SearchPredicate {
            clause: "1 = 1".into(),
            args: Vec::new(),
        }))
}

pub(super) fn predicate(
    connection: &Connection,
    query: &str,
    category_id: Option<i64>,
    activity: &super::ProductActivityFilter,
) -> rusqlite::Result<Option<SearchPredicate>> {
    if query.trim().is_empty() {
        return Ok(None);
    }
    // Match caller eligibility, but not stock: stock exclusion must not enable fallback.
    let exact_exists = connection.query_row(
        &format!(
            "SELECT EXISTS(
                SELECT 1 FROM products p
                JOIN catalog_product_search search ON search.product_id = p.id
                JOIN categories c ON c.id = p.category_id
                JOIN stock_balances s ON s.product_id = p.id
                WHERE p.sku = ?1 AND (?2 IS NULL OR p.category_id = ?2)
                  AND {}
             )",
            activity_clause(activity)
        ),
        rusqlite::params![query, category_id],
        |row| row.get::<_, bool>(0),
    )?;
    if exact_exists {
        return Ok(Some(SearchPredicate {
            clause: "p.sku = ?1".into(),
            args: vec![Value::Text(query.to_owned())],
        }));
    }
    Ok(fallback_predicate(query))
}

fn fallback_predicate(query: &str) -> Option<SearchPredicate> {
    if query.trim().is_empty() {
        return None;
    }
    // Classify before FTS punctuation splitting so decimals and signs stay atomic.
    let numbers = query
        .split_whitespace()
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()
        .filter(|values| !values.is_empty() && values.iter().all(|value| value.is_finite()));
    if let Some(numbers) = numbers {
        let single_number = numbers.len() == 1;
        let mut groups: Vec<(f64, usize)> = Vec::new();
        for number in numbers {
            if let Some((_, count)) = groups.iter_mut().find(|(value, _)| *value == number) {
                *count += 1;
            } else {
                groups.push((number, 1));
            }
        }
        // Equal terms share a required count; unequal values cannot reuse a row.
        let clauses = groups
            .iter()
            .enumerate()
            .map(|(index, _)| {
                format!(
                    "(SELECT COUNT(*) FROM product_attribute_values v
                      WHERE v.product_id = p.id
                        AND repuestos_catalog_finite_number(
                            COALESCE(v.number_value, v.text_value, v.option_value)
                        ) = ?{}) >= ?{}",
                    index * 2 + 2,
                    index * 2 + 3,
                )
            })
            .collect::<Vec<_>>();
        let mut args = vec![Value::Text(query.to_owned())];
        args.extend(
            groups
                .into_iter()
                .flat_map(|(value, count)| [Value::Real(value), Value::Integer(count as i64)]),
        );
        // Only single-number lookup may use SKU token prefixes, never merged FTS fields.
        let sku_clause = if single_number {
            "p.sku = ?1 OR repuestos_catalog_sku_prefix(p.sku, ?1)"
        } else {
            "p.sku = ?1"
        };
        return Some(SearchPredicate {
            clause: format!("({sku_clause} OR ({}))", clauses.join(" AND ")),
            args,
        });
    }
    let mut args = vec![Value::Text(query.to_owned())];
    let clause = if let Some(fts) = super::normalized_search_query(query) {
        args.push(Value::Text(fts));
        "(p.sku = ?1 OR p.id IN (SELECT product_id FROM catalog_product_search WHERE content MATCH ?2))"
    } else {
        "p.sku = ?1"
    };
    Some(SearchPredicate {
        clause: clause.into(),
        args,
    })
}
