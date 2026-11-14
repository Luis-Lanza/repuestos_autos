use repuestos_autos::catalog::open_seeded_catalog;
use rusqlite::params;
use std::time::{Duration, Instant};

#[test]
fn exact_sku_preference_hides_similar_ngk_references_in_legacy_searches() {
    let connection = open_seeded_catalog().unwrap();
    connection.execute_batch(
        "UPDATE products SET sku = 'NGK BKR6E-11', name = 'Z exact plug' WHERE id = 1;
         UPDATE catalog_product_search SET content = 'ngk bkr6e 11' WHERE product_id = 1;
         INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos)
         VALUES (9701, 1, 'NGK BKR6EIX-11', 'A similar plug', 1, 2500, 2500);
         INSERT INTO stock_balances (product_id, quantity) VALUES (9701, 8);
         INSERT INTO catalog_product_search (rowid, product_id, content)
         VALUES (9701, 9701, 'ngk bkr6eix 11');"
    ).unwrap();
    for ids in [
        repuestos_autos::catalog::search_active_products(&connection, "NGK BKR6E-11")
            .unwrap()
            .iter()
            .map(|p| p.product_id)
            .collect::<Vec<_>>(),
        repuestos_autos::application::catalog::search_active_sale_products(
            &connection,
            "NGK BKR6E-11",
        )
        .unwrap()
        .iter()
        .map(|p| p.product_id)
        .collect::<Vec<_>>(),
    ] {
        assert_eq!(
            ids,
            vec![1],
            "an eligible exact SKU excludes similar references"
        );
    }
}

#[test]
fn single_numeric_search_keeps_sku_token_prefixes_isolated_from_other_fields() {
    use repuestos_autos::application::catalog::{
        browse_active_inventory_products, browse_active_products, browse_active_sale_products,
        BrowseProductsInput, ProductActivityFilter, ProductStockFilter,
    };
    use repuestos_autos::infrastructure::sqlite::SqliteCatalogRepository;

    let connection = open_seeded_catalog().unwrap();
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (9601, 1, 'Measurement', 'text', 0);
        INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES
            (9601, 1, '39211-28GX0', 'SKU match', 1, 2500, 2500),
            (9602, 1, '928', '28', 1, 2500, 2500),
            (9603, 1, 'OTHER', '28', 1, 2500, 2500);
        INSERT INTO stock_balances (product_id, quantity) VALUES (9601, 8), (9602, 8), (9603, 8);
        INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES
            (9601, 9601, '280', '280'), (9602, 9601, 'model 28', 'model 28'), (9603, 9601, '280', '280');
        INSERT INTO catalog_product_search (rowid, product_id, content) VALUES
            (9601, 9601, '39211 28gx0 sku match 280'), (9602, 9602, '928 28 model 28'), (9603, 9603, 'other 28 280');").unwrap();
    let catalog = repuestos_autos::catalog::search_active_products(&connection, "28").unwrap();
    assert_eq!(catalog.iter().map(|p| p.product_id).collect::<Vec<_>>(), vec![9601], "28 must match the SKU token prefix, not names or attribute prefixes");
    let sales = repuestos_autos::application::catalog::search_active_sale_products(&connection, "28").unwrap();
    assert_eq!(sales.iter().map(|p| p.product_id).collect::<Vec<_>>(), vec![9601]);
    let input = BrowseProductsInput { query: Some("28".into()), category_id: Some(1), stock_filter: ProductStockFilter::All, activity_filter: ProductActivityFilter::Active, page: 1, page_size: 1 };
    let catalog = browse_active_products(&connection, &SqliteCatalogRepository, &input).unwrap();
    let inventory = browse_active_inventory_products(&connection, &input).unwrap();
    let sales = browse_active_sale_products(&connection, &input).unwrap();
    assert_eq!((catalog.total, inventory.total, sales.total), (1, 1, 1));
    assert_eq!((catalog.products[0].product_id, inventory.products[0].product_id, sales.products[0].product_id), (9601, 9601, 9601));
    assert!(repuestos_autos::catalog::search_active_products(&connection, "28 28 58").unwrap().is_empty(), "partial SKU must not participate in multi-number queries");
    connection.execute("UPDATE products SET sku = '39211-28.5GX0' WHERE id = 9601", []).unwrap();
    let decimal = repuestos_autos::catalog::search_active_products(&connection, "28.5").unwrap();
    assert_eq!(decimal.iter().map(|p| p.product_id).collect::<Vec<_>>(), vec![9601], "decimal SKU lookup keeps token-prefix semantics; attributes remain numeric and atomic");
    connection.execute("UPDATE products SET sku = '280' WHERE id = 9601", []).unwrap();
    assert_eq!(repuestos_autos::catalog::search_active_products(&connection, "28").unwrap()[0].product_id, 9601, "SKU token prefixes intentionally differ from exact numeric attributes");
}

#[test]
fn repeated_numeric_terms_require_distinct_attribute_rows() {
    let connection = open_seeded_catalog().unwrap();
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (9101, 1, 'Width', 'number', 0), (9102, 1, 'Height', 'number', 0), (9103, 1, 'Length', 'number', 0);
        INSERT INTO product_attribute_values (product_id, definition_id, number_value, searchable_value) VALUES (1, 9101, 28, '28'), (1, 9102, 58, '58');
        UPDATE catalog_product_search SET content = 'flt filtro 28 58' WHERE product_id = 1;").unwrap();
    let results = repuestos_autos::catalog::search_active_products(&connection, "28 28 58").unwrap();
    assert!(results.is_empty(), "one 28 row must not satisfy two occurrences");
    connection.execute("INSERT INTO product_attribute_values (product_id, definition_id, number_value, searchable_value) VALUES (1, 9103, 28, '28')", []).unwrap();
    let results = repuestos_autos::catalog::search_active_products(&connection, "28 28 58").unwrap();
    assert_eq!(results.iter().map(|p| p.product_id).collect::<Vec<_>>(), vec![1]);
    connection.execute("UPDATE product_attribute_values SET number_value = 280, searchable_value = '280' WHERE product_id = 1", []).unwrap();
    assert!(repuestos_autos::catalog::search_active_products(&connection, "28").unwrap().is_empty(), "single numeric terms cannot match attribute prefixes");
}

#[test]
fn whole_numeric_text_and_options_participate_without_prefix_or_unit_coercion() {
    let connection = open_seeded_catalog().unwrap();
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (9201, 1, 'Width', 'text', 0), (9202, 1, 'Height', 'option', 0), (9203, 1, 'Length', 'number', 0);
        INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 9201, '28.0', '28.0');
        INSERT INTO product_attribute_values (product_id, definition_id, option_value, searchable_value) VALUES (1, 9202, '+2.8e1', '+2.8e1');
        INSERT INTO product_attribute_values (product_id, definition_id, number_value, searchable_value) VALUES (1, 9203, 58, '58');").unwrap();
    for query in ["28", "28 28 58", "58 28.0 +2.8e1", "28 58 28"] {
        let results = repuestos_autos::catalog::search_active_products(&connection, query).unwrap();
        assert_eq!(results.iter().map(|p| p.product_id).collect::<Vec<_>>(), vec![1], "{query}");
        let sales = repuestos_autos::application::catalog::search_active_sale_products(&connection, query).unwrap();
        assert_eq!(sales.iter().map(|p| p.product_id).collect::<Vec<_>>(), vec![1], "{query}");
    }
    // Persisted values stay searchable even when their definitions are retired.
    connection.execute("UPDATE attribute_definitions SET active = 0 WHERE id = 9202", []).unwrap();
    assert_eq!(repuestos_autos::catalog::search_active_products(&connection, "28 28 58").unwrap()[0].product_id, 1);
    for value in ["280", "model 28", "28mm", "28,0", "NaN", "inf", "1e999"] {
        connection.execute("UPDATE product_attribute_values SET text_value = ?1, searchable_value = ?1 WHERE definition_id = 9201", [value]).unwrap();
        assert!(repuestos_autos::catalog::search_active_products(&connection, "28 28 58").unwrap().is_empty(), "{value}");
    }
}

#[test]
fn exact_full_sku_is_an_independent_alternative_to_numeric_attributes() {
    let connection = open_seeded_catalog().unwrap();
    for sku in ["28", "28.5", "28 28 58", "28-28-58", "---"] {
        connection.execute("UPDATE products SET sku = ?1 WHERE id = 1", [sku]).unwrap();
        for results in [
            repuestos_autos::catalog::search_active_products(&connection, sku).unwrap().iter().map(|p| p.product_id).collect::<Vec<_>>(),
            repuestos_autos::application::catalog::search_active_sale_products(&connection, sku).unwrap().iter().map(|p| p.product_id).collect::<Vec<_>>(),
        ] {
            assert_eq!(results, vec![1], "exact SKU {sku}");
        }
    }
    connection.execute("UPDATE products SET sku = '928', name = '28 28 58' WHERE id = 1", []).unwrap();
    connection.execute("UPDATE catalog_product_search SET content = '928 28 28 58' WHERE product_id = 1", []).unwrap();
    for query in ["28", "28 28 58", "28.5"] {
        assert!(repuestos_autos::catalog::search_active_products(&connection, query).unwrap().is_empty(), "{query} must not use SKU substrings or name terms");
    }
    connection.execute("UPDATE products SET sku = '58' WHERE id = 1", []).unwrap();
    assert!(repuestos_autos::catalog::search_active_products(&connection, "28 28 58").unwrap().is_empty(), "SKU 58 cannot supply measurements");
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (9251, 1, 'First', 'number', 0), (9252, 1, 'Second', 'number', 0);
        INSERT INTO product_attribute_values (product_id, definition_id, number_value, searchable_value) VALUES (1, 9251, 28, '28'), (1, 9252, 28, '28');
        INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (9250, 1, '28 28 58', 'Exact SKU', 1, 2500, 2500);
        INSERT INTO stock_balances (product_id, quantity) VALUES (9250, 8);
        INSERT INTO catalog_product_search (rowid, product_id, content) VALUES (9250, 9250, '28 28 58');").unwrap();
    let results = repuestos_autos::catalog::search_active_products(&connection, "28 28 58").unwrap();
    assert_eq!(results.iter().map(|p| p.product_id).collect::<Vec<_>>(), vec![9250], "SKU must be a whole-product alternative, not a source for missing 58");
}

#[test]
fn decimals_remain_atomic_and_distinct_values_are_order_independent() {
    let connection = open_seeded_catalog().unwrap();
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (9401, 1, 'First', 'number', 0), (9402, 1, 'Second', 'number', 0), (9403, 1, 'Third', 'number', 0);
        INSERT INTO product_attribute_values (product_id, definition_id, number_value, searchable_value) VALUES (1, 9401, 28.5, '28.5'), (1, 9402, -12, '-12'), (1, 9403, 58, '58');
        UPDATE products SET sku = 'ATOMIC' WHERE id = 1;
        UPDATE catalog_product_search SET content = 'atomic 28 5 12 58' WHERE product_id = 1;").unwrap();
    for query in ["28.5", "28.5 -12 58", "58 28.5 -12", "-12 58 28.5", "+2.85e1 -1.2e1 58.0"] {
        let catalog = repuestos_autos::catalog::search_active_products(&connection, query).unwrap();
        let sales = repuestos_autos::application::catalog::search_active_sale_products(&connection, query).unwrap();
        assert_eq!(catalog.iter().map(|p| p.product_id).collect::<Vec<_>>(), vec![1], "{query}");
        assert_eq!(sales.iter().map(|p| p.product_id).collect::<Vec<_>>(), vec![1], "{query}");
    }
    for query in ["28", "5", "28 5", "28.5 28.5", "12", "0"] {
        assert!(repuestos_autos::catalog::search_active_products(&connection, query).unwrap().is_empty(), "{query}");
    }
    connection.execute("UPDATE categories SET name = '28 28 58' WHERE id = 1", []).unwrap();
    assert!(repuestos_autos::catalog::search_active_products(&connection, "28 28 58").unwrap().is_empty());
    for query in ["", "   ", "\t\n"] {
        assert!(repuestos_autos::catalog::search_active_products(&connection, query).unwrap().is_empty());
        assert!(repuestos_autos::application::catalog::search_active_sale_products(&connection, query).unwrap().is_empty());
    }
}

#[test]
fn numeric_search_filters_twenty_thousand_products_before_limits_and_pages() {
    use repuestos_autos::application::catalog::{browse_active_inventory_products, BrowseProductsInput, ProductActivityFilter, ProductStockFilter};
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (9501, 1, 'First', 'number', 0), (9502, 1, 'Second', 'text', 0), (9503, 1, 'Third', 'option', 0);").unwrap();
    let transaction = connection.transaction().unwrap();
    let mut products = transaction.prepare("INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (1, ?1, ?2, 1, 2500, 2500)").unwrap();
    let mut balances = transaction.prepare("INSERT INTO stock_balances (product_id, quantity) VALUES (?1, 8)").unwrap();
    let mut attributes = transaction.prepare("INSERT INTO product_attribute_values (product_id, definition_id, number_value, text_value, option_value, searchable_value) VALUES (?1, ?2, ?3, ?4, ?5, ?6)").unwrap();
    let mut documents = transaction.prepare("INSERT INTO catalog_product_search (rowid, product_id, content) VALUES (?1, ?1, 'benchmark 28 58')").unwrap();
    let mut matches = Vec::new();
    for index in 0..20_000 {
        products.execute(params![format!("NUM-{index:05}"), format!("Numeric benchmark {index:05}")]).unwrap();
        let id = transaction.last_insert_rowid();
        balances.execute([id]).unwrap();
        documents.execute([id]).unwrap();
        attributes.execute(params![id, 9501, Some(28.0), None::<String>, None::<String>, "28"]).unwrap();
        attributes.execute(params![id, 9502, None::<f64>, Some("58"), None::<String>, "58"]).unwrap();
        let third = if index >= 19_970 { matches.push(id); "28.0" } else { "280" };
        attributes.execute(params![id, 9503, None::<f64>, None::<String>, Some(third), third]).unwrap();
    }
    drop((products, balances, attributes, documents));
    transaction.commit().unwrap();
    let input = BrowseProductsInput { query: Some("28 28 58".into()), category_id: Some(1), stock_filter: ProductStockFilter::Available, activity_filter: ProductActivityFilter::Active, page: 2, page_size: 20 };
    let started = Instant::now();
    let page = browse_active_inventory_products(&connection, &input).unwrap();
    let legacy = repuestos_autos::catalog::search_active_products(&connection, "28 28 58").unwrap();
    let elapsed = started.elapsed();
    eprintln!("20,000-product numeric count, second page and legacy search: {elapsed:?}");
    assert_eq!((page.total, page.total_pages), (30, 2));
    assert_eq!(page.products.iter().map(|p| p.product_id).collect::<Vec<_>>(), matches[20..]);
    assert_eq!(legacy.iter().map(|p| p.product_id).collect::<Vec<_>>(), matches[..20]);
    let budget = if cfg!(debug_assertions) { Duration::from_secs(5) } else { Duration::from_secs(1) };
    assert!(elapsed <= budget, "numeric search took {elapsed:?}");
}

#[test]
fn finds_active_seeded_products_by_every_searchable_catalog_field() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");

    for query in ["filtro", "FLT-001", "Filtros", "Toyota"] {
        let results = repuestos_autos::catalog::search_active_products(&connection, query)
            .expect("catalog search succeeds");
        assert_eq!(results.len(), 1, "query {query}");
        let product = &results[0];
        assert_eq!(product.sku, "FLT-001");
        assert_eq!(product.available_quantity, 8);
        assert_eq!(product.sale_price_centavos, 2_500);
        assert_eq!(product.purchase_price_centavos, None);
    }
}

#[test]
fn sale_search_contract_contains_only_safe_sale_facts() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    connection.execute("UPDATE products SET purchase_price_centavos = 7777 WHERE id = 1", []).unwrap();
    let result = repuestos_autos::application::catalog::search_active_sale_products(&connection, "filtro").unwrap();
    let value = serde_json::to_value(&result[0]).unwrap();
    assert_eq!(value.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>(), [
        "available_quantity", "category_name", "minimum_sale_price_centavos", "name", "product_id", "sale_price_centavos", "sku",
    ]);
    assert!(!value.to_string().contains("7777"));
}

#[test]
fn sales_browse_projection_contains_only_active_filter_and_sale_facts() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    connection.execute("UPDATE products SET purchase_price_centavos = 7777, revision = 9 WHERE id = 1", []).unwrap();
    connection.execute("INSERT INTO attribute_definitions (id, category_id, label, field_type, required, active) VALUES (9001, 1, 'Material', 'text', 0, 1), (9002, 1, 'Retired field', 'text', 0, 0)", []).unwrap();
    connection.execute("INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 9001, 'Acero', 'Acero'), (1, 9002, 'Hidden', 'Hidden')", []).unwrap();
    let page = repuestos_autos::application::catalog::browse_active_sale_products(&connection, &repuestos_autos::application::catalog::BrowseProductsInput {
        query: Some("filtro".into()), category_id: Some(1), stock_filter: repuestos_autos::application::catalog::ProductStockFilter::Available,
        activity_filter: repuestos_autos::application::catalog::ProductActivityFilter::Active, page: 1, page_size: 20,
    }).unwrap();
    assert_eq!(page.categories.len(), 2);
    assert_eq!(page.products.len(), 1);
    let value = serde_json::to_value(&page.products[0]).unwrap();
    assert_eq!(value.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>(), [
        "attribute_values", "available_quantity", "category_id", "category_name", "minimum_sale_price_centavos", "name", "primary_location_code", "product_id", "purchase_price_centavos", "sale_price_centavos", "sku",
    ]);
    assert_eq!(value["attribute_values"], serde_json::json!([
        { "definition_id": 9001, "label": "Material", "value": "Acero" }
    ]));
    assert_eq!(value["minimum_sale_price_centavos"], 2_500);
    assert!(!value.to_string().contains("Hidden"));
    assert_eq!(value["purchase_price_centavos"], 7777);
    assert!(!value.to_string().contains("revision"));
    assert_eq!(value["primary_location_code"], serde_json::Value::Null);
    assert!(!value.to_string().contains("active"));
    connection.execute("UPDATE categories SET active = 0 WHERE id = 1", []).unwrap();
    let page = repuestos_autos::application::catalog::browse_active_sale_products(&connection, &repuestos_autos::application::catalog::BrowseProductsInput {
        query: None, category_id: None, stock_filter: repuestos_autos::application::catalog::ProductStockFilter::Available,
        activity_filter: repuestos_autos::application::catalog::ProductActivityFilter::Active, page: 1, page_size: 20,
    }).unwrap();
    assert!(page.products.is_empty());
    assert!(!page.categories.iter().any(|category| category.category_id == 1));
}

#[test]
fn sales_browse_rejects_attribute_facts_over_the_projection_bounds() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    connection.execute("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (9002, 1, 'Material', 'text', 0)", []).unwrap();
    connection.execute("INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 9002, ?1, ?1)", ["x".repeat(257)]).unwrap();
    let result = repuestos_autos::application::catalog::browse_active_sale_products(&connection, &repuestos_autos::application::catalog::BrowseProductsInput {
        query: Some("filtro".into()), category_id: Some(1), stock_filter: repuestos_autos::application::catalog::ProductStockFilter::Available,
        activity_filter: repuestos_autos::application::catalog::ProductActivityFilter::Active, page: 1, page_size: 20,
    });
    assert!(result.is_err(), "oversized persisted attribute text must fail closed");
}

#[test]
fn inventory_browse_is_bounded_to_operational_facts() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    connection.execute("UPDATE products SET purchase_price_centavos = 7777, revision = 9 WHERE id = 1", []).unwrap();
    connection.execute("INSERT INTO attribute_definitions (id, category_id, label, field_type, required, active) VALUES (9001, 1, 'Retired secret', 'text', 0, 0)", []).unwrap();
    connection.execute("INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 9001, 'hidden-attribute', 'hidden-attribute')", []).unwrap();
    let page = repuestos_autos::application::catalog::browse_active_inventory_products(&connection, &repuestos_autos::application::catalog::BrowseProductsInput {
        query: Some("filtro".into()), category_id: Some(1), stock_filter: repuestos_autos::application::catalog::ProductStockFilter::All,
        activity_filter: repuestos_autos::application::catalog::ProductActivityFilter::Active, page: 1, page_size: 20,
    }).unwrap();
    let value = serde_json::to_value(&page).unwrap();
    assert_eq!(value["products"][0].as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>(), [
        "available_quantity", "category_id", "category_name", "minimum_sale_price_centavos", "name", "primary_location_code", "product_id", "sale_price_centavos", "sku",
    ]);
    for forbidden in ["purchase_price_centavos", "revision", "attribute_values", "hidden-attribute", "7777", "active"] {
        assert!(!value.to_string().contains(forbidden), "unexpected Inventory projection field {forbidden}: {value}");
    }
}

#[test]
fn exposes_sale_minimum_and_purchase_prices_separately() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    connection
        .execute("UPDATE products SET purchase_price_centavos = 3200, list_price_centavos = 5000, minimum_unit_price_centavos = 2500 WHERE id = 1", [])
        .unwrap();

    let product = &repuestos_autos::catalog::search_active_products(&connection, "filtro")
        .unwrap()[0];

    assert_eq!(product.purchase_price_centavos, Some(3_200));
    assert_eq!(product.sale_price_centavos, 5_000);
    assert_eq!(product.minimum_sale_price_centavos, 2500);
}

#[test]
fn enables_foreign_keys_for_the_disposable_database() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    let foreign_keys: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("foreign key status");

    assert_eq!(foreign_keys, 1);
}

#[test]
fn excludes_inactive_products() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    let results = repuestos_autos::catalog::search_active_products(&connection, "archivado")
        .expect("catalog search succeeds");
    assert!(results.is_empty());
}

#[test]
fn searches_the_canonical_fts_document_with_prefixes_and_a_bounded_result_set() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    connection
        .execute_batch("DROP TABLE product_searchable_values;")
        .expect("legacy search table can be absent after FTS backfill");

    for index in 0..21 {
        connection
            .execute(
                "INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (1, ?1, ?2, 1, 2500, 2500)",
                [format!("BRG-{index:02}"), format!("Bearing {index:02}")],
            )
            .expect("product persists");
        let product_id = connection.last_insert_rowid();
        connection
            .execute(
                "INSERT INTO stock_balances (product_id, quantity) VALUES (?1, 1)",
                [product_id],
            )
            .expect("stock persists");
        connection
            .execute(
                "INSERT INTO catalog_product_search (rowid, product_id, content) VALUES (?1, ?1, ?2)",
                params![product_id, format!("brg {index:02} bearing")],
            )
            .expect("canonical search document persists");
    }

    let results = repuestos_autos::catalog::search_active_products(&connection, "bea")
        .expect("FTS search succeeds without legacy searchable values");

    assert_eq!(results.len(), 20);
    assert!(results
        .iter()
        .all(|product| product.name.starts_with("Bearing")));
}

#[test]
fn searches_twenty_thousand_catalog_products_within_the_release_target() {
    let mut connection = open_seeded_catalog().expect("a disposable catalog database");
    let transaction = connection
        .transaction()
        .expect("benchmark catalog transaction starts");
    let mut products = transaction
        .prepare("INSERT INTO products (category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (1, ?1, ?2, 1, 2500, 2500)")
        .expect("product statement prepares");
    let mut balances = transaction
        .prepare("INSERT INTO stock_balances (product_id, quantity) VALUES (?1, 1)")
        .expect("balance statement prepares");
    let mut search_documents = transaction
        .prepare(
            "INSERT INTO catalog_product_search (rowid, product_id, content) VALUES (?1, ?1, ?2)",
        )
        .expect("search document statement prepares");

    for index in 0..20_000 {
        let sku = format!("BNCH-{index:05}");
        let name = format!("Benchmark product {index:05}");
        products
            .execute(params![sku, name])
            .expect("benchmark product persists");
        let product_id = transaction.last_insert_rowid();
        balances
            .execute([product_id])
            .expect("benchmark balance persists");
        search_documents
            .execute(params![
                product_id,
                format!("bnch benchmark product {index:05}")
            ])
            .expect("benchmark search document persists");
    }
    drop((products, balances, search_documents));
    transaction.commit().expect("benchmark catalog commits");

    let started = Instant::now();
    let results = repuestos_autos::catalog::search_active_products(&connection, "bench")
        .expect("benchmark prefix search succeeds");
    let elapsed = started.elapsed();

    eprintln!("20,000-product prefix search: {elapsed:?}");

    assert_eq!(results.len(), 20);
    assert!(results
        .iter()
        .all(|product| product.name.starts_with("Benchmark product")));
    assert!(
        elapsed <= Duration::from_secs(1),
        "20,000-product prefix search took {elapsed:?}"
    );
}
