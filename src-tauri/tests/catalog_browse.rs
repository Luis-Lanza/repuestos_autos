use repuestos_autos::application::catalog::{browse_active_products, create_product, AttributeValueInput, BrowseProductsInput, CreateProductInput, ProductActivityFilter, ProductStockFilter};
use repuestos_autos::application::catalog::locations::{assign_product_location, create_product_location, save_location_schema, CreateProductLocationInput, SaveLocationSchemaInput};
use repuestos_autos::catalog::open_seeded_catalog;
use repuestos_autos::infrastructure::sqlite::{dashboard_repository::SqliteDashboardReader, SqliteCatalogRepository};
use repuestos_autos::application::reporting::{DashboardRange, DashboardReader};

fn browse(connection: &rusqlite::Connection, stock_filter: ProductStockFilter, page: i64, page_size: i64) -> repuestos_autos::application::catalog::ProductBrowsePage {
    browse_active_products(connection, &SqliteCatalogRepository, &BrowseProductsInput {
        query: None,
        category_id: None,
        stock_filter,
        activity_filter: ProductActivityFilter::Active,
        page,
        page_size,
    }).expect("catalog browse succeeds")
}

#[test]
fn blank_query_returns_the_first_bounded_page_with_metadata() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    let page = browse(&connection, ProductStockFilter::All, 1, 1);

    assert_eq!(page.products.len(), 1);
    assert_eq!(page.total, 1);
    assert_eq!(page.total_pages, 1);
    assert_eq!(page.products[0].category_id, 1);
    assert_eq!(page.categories.len(), 2);
}

#[test]
fn category_and_stock_filters_are_applied_server_side() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    connection.execute("UPDATE stock_balances SET quantity = 0 WHERE product_id = 1", []).unwrap();
    let page = browse(&connection, ProductStockFilter::OutOfStock, 1, 20);

    assert_eq!(page.products.len(), 1);
    assert_eq!(page.products[0].available_quantity, 0);
    assert_eq!(page.products[0].category_name, "Filtros");
    assert!(browse_active_products(&connection, &SqliteCatalogRepository, &BrowseProductsInput {
        query: None,
        category_id: Some(2),
        stock_filter: ProductStockFilter::All,
        activity_filter: ProductActivityFilter::Active,
        page: 1,
        page_size: 20,
    }).unwrap().products.is_empty());
}

#[test]
fn browse_order_is_stable_and_query_remains_searchable() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    let page = browse_active_products(&connection, &SqliteCatalogRepository, &BrowseProductsInput {
        query: Some("FLT".into()),
        category_id: None,
        stock_filter: ProductStockFilter::Available,
        activity_filter: ProductActivityFilter::Active,
        page: 1,
        page_size: 20,
    }).expect("catalog browse succeeds");

    assert_eq!(page.products[0].sku, "FLT-001");
    assert!(page.products[0].attribute_values.is_empty());
}

#[test]
fn page_attributes_include_every_definition_in_id_order_and_default_missing_values_to_empty() {
    let mut connection = open_seeded_catalog().expect("a disposable catalog database");
    connection.execute("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (12, 1, 'Diameter', 'text', 0), (5, 1, 'Material', 'text', 0)", []).unwrap();
    connection.execute("INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 12, '50', '50')", []).unwrap();
    create_product(&mut connection, CreateProductInput {
        sku: "ZZZ-001".into(), name: "Zulu filter".into(), category_id: 1,
        purchase_price_centavos: 100, sale_price_centavos: 200,
        minimum_sale_price_centavos: 100, low_stock_threshold: None, opening_quantity: 2,
        attribute_values: vec![AttributeValueInput { definition_id: 12, value: "99".into() }],
    }).unwrap();

    let first = browse(&connection, ProductStockFilter::All, 1, 1);
    assert_eq!(first.products[0].attribute_values.iter().map(|attribute| (attribute.definition_id, attribute.label.as_str(), attribute.value.as_str())).collect::<Vec<_>>(), vec![(5, "Material", ""), (12, "Diameter", "50")]);
    let second = browse(&connection, ProductStockFilter::All, 2, 1);
    assert_eq!(second.products[0].name, "Zulu filter");
    assert_eq!(second.products[0].attribute_values.iter().map(|attribute| (attribute.definition_id, attribute.value.as_str())).collect::<Vec<_>>(), vec![(5, ""), (12, "99")]);
}

#[test]
fn browse_includes_optional_generated_primary_location_without_changing_global_stock() {
    let mut connection = open_seeded_catalog().expect("a disposable catalog database");
    let unassigned = browse(&connection, ProductStockFilter::All, 1, 20);
    assert_eq!(unassigned.products[0].primary_location_code, None);

    save_location_schema(&mut connection, SaveLocationSchemaInput {
        expected_revision: 0,
        segments: vec!["Zone".into(), "Shelf".into()],
    }).unwrap();
    let location = create_product_location(&mut connection, CreateProductLocationInput {
        values: vec!["A-1".into(), "Shelf 2".into()],
    }).unwrap();
    assert_eq!(location.code, "A1-SHELF2");
    assert_eq!(assign_product_location(&mut connection, 1, 0, Some(location.location_id)), Ok(1));

    let assigned = browse(&connection, ProductStockFilter::All, 1, 20);
    assert_eq!(assigned.products[0].primary_location_code.as_deref(), Some("A1-SHELF2"));
    assert_eq!(assigned.products[0].available_quantity, unassigned.products[0].available_quantity);
}

#[test]
fn sales_browse_serializes_nullable_location_without_sensitive_catalog_facts_or_writes() {
    let mut connection = open_seeded_catalog().unwrap();
    let input = BrowseProductsInput {
        query: Some("FLT".into()), category_id: Some(1), stock_filter: ProductStockFilter::All,
        activity_filter: ProductActivityFilter::Active, page: 1, page_size: 20,
    };
    let unassigned = serde_json::to_value(repuestos_autos::application::catalog::browse_active_sale_products(&connection, &input).unwrap()).unwrap();
    assert_eq!(unassigned["products"].as_array().unwrap().len(), 1);
    assert!(unassigned["products"][0].as_object().unwrap().contains_key("primary_location_code"));
    assert_eq!(unassigned["products"][0]["primary_location_code"], serde_json::Value::Null);
    save_location_schema(&mut connection, SaveLocationSchemaInput { expected_revision: 0, segments: vec!["Zone".into(), "Shelf".into()] }).unwrap();
    let location = create_product_location(&mut connection, CreateProductLocationInput { values: vec!["A-1".into(), "Shelf 2".into()] }).unwrap();
    assign_product_location(&mut connection, 1, 0, Some(location.location_id)).unwrap();
    let changes_before = connection.total_changes();
    let assigned = serde_json::to_value(repuestos_autos::application::catalog::browse_active_sale_products(&connection, &input).unwrap()).unwrap();
    assert_eq!(assigned["products"][0]["primary_location_code"], "A1-SHELF2");
    assert_eq!(assigned["products"][0]["available_quantity"], unassigned["products"][0]["available_quantity"]);
    assert_eq!(connection.total_changes(), changes_before, "Sales browse must remain read-only");
    for forbidden in ["purchase_price_centavos", "revision", "primary_location_id", "low_stock_threshold", "activity", "active", "active_product_count"] {
        assert!(!assigned["products"][0].as_object().unwrap().contains_key(forbidden));
    }
}

#[test]
fn stock_filters_use_each_products_threshold_and_keep_zero_out_of_low_stock() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    connection.execute("UPDATE products SET low_stock_threshold = 4 WHERE id = 1", []).unwrap();
    connection.execute("UPDATE stock_balances SET quantity = 4 WHERE product_id = 1", []).unwrap();

    assert_eq!(browse(&connection, ProductStockFilter::LowStock, 1, 20).total, 1);
    assert_eq!(browse(&connection, ProductStockFilter::Alerts, 1, 20).total, 1);
    assert_eq!(browse(&connection, ProductStockFilter::Available, 1, 20).total, 0);

    connection.execute("UPDATE stock_balances SET quantity = 0 WHERE product_id = 1", []).unwrap();
    assert_eq!(browse(&connection, ProductStockFilter::LowStock, 1, 20).total, 0);
    assert_eq!(browse(&connection, ProductStockFilter::Alerts, 1, 20).total, 1);
    assert_eq!(browse(&connection, ProductStockFilter::OutOfStock, 1, 20).total, 1);
}

#[test]
fn dashboard_alerts_use_product_threshold_and_preserve_zero_classification() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    connection.execute("UPDATE products SET low_stock_threshold = 4 WHERE id = 1", []).unwrap();
    connection.execute("UPDATE stock_balances SET quantity = 4 WHERE product_id = 1", []).unwrap();
    let range = DashboardRange::parse("2024-03-01T00:00:00Z", "2024-04-01T00:00:00Z").unwrap();
    let report = SqliteDashboardReader::new(&connection).read(&range, &range).unwrap();
    assert_eq!(report.stock_alerts.iter().map(|alert| (alert.quantity, alert.classification.as_str())).collect::<Vec<_>>(), vec![(4, "low_stock")]);

    connection.execute("UPDATE stock_balances SET quantity = 0 WHERE product_id = 1", []).unwrap();
    let report = SqliteDashboardReader::new(&connection).read(&range, &range).unwrap();
    assert_eq!(report.stock_alerts.iter().map(|alert| (alert.quantity, alert.classification.as_str())).collect::<Vec<_>>(), vec![(0, "out_of_stock")]);
}

#[test]
fn browse_rejects_unbounded_page_sizes() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    let result = browse_active_products(&connection, &SqliteCatalogRepository, &BrowseProductsInput {
        query: None,
        category_id: None,
        stock_filter: ProductStockFilter::All,
        activity_filter: ProductActivityFilter::Active,
        page: 1,
        page_size: 51,
    });

    assert!(result.is_err());
}
