use repuestos_autos::catalog::open_seeded_catalog;
use rusqlite::params;
use std::time::{Duration, Instant};

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
        "attribute_values", "available_quantity", "category_id", "category_name", "minimum_sale_price_centavos", "name", "product_id", "sale_price_centavos", "sku",
    ]);
    assert_eq!(value["attribute_values"], serde_json::json!([
        { "definition_id": 9001, "label": "Material", "value": "Acero" }
    ]));
    assert_eq!(value["minimum_sale_price_centavos"], 2_500);
    assert!(!value.to_string().contains("Hidden"));
    assert!(!value.to_string().contains("7777"));
    assert!(!value.to_string().contains("revision"));
    assert!(!value.to_string().contains("location"));
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
