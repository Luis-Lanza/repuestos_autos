use rusqlite::{Connection, Result};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct ProductSearchResult {
    pub product_id: i64,
    pub sku: String,
    pub name: String,
    pub category_name: String,
    pub available_quantity: i64,
    pub minimum_unit_price_centavos: i64,
}

pub fn search_active_products(
    connection: &Connection,
    query: &str,
) -> Result<Vec<ProductSearchResult>> {
    let pattern = format!("%{}%", query.trim().to_lowercase());
    let mut statement = connection.prepare(
        "SELECT DISTINCT p.id, p.sku, p.name, c.name, s.quantity, p.minimum_unit_price_centavos
         FROM products p JOIN categories c ON c.id = p.category_id
         JOIN stock_balances s ON s.product_id = p.id
         LEFT JOIN product_searchable_values v ON v.product_id = p.id
         WHERE p.active = 1 AND (lower(p.sku) LIKE ?1 OR lower(p.name) LIKE ?1
           OR lower(c.name) LIKE ?1 OR lower(v.value) LIKE ?1) ORDER BY p.name",
    )?;
    let results = statement
        .query_map([pattern], |row| {
            Ok(ProductSearchResult {
                product_id: row.get(0)?,
                sku: row.get(1)?,
                name: row.get(2)?,
                category_name: row.get(3)?,
                available_quantity: row.get(4)?,
                minimum_unit_price_centavos: row.get(5)?,
            })
        })?
        .collect();
    results
}
