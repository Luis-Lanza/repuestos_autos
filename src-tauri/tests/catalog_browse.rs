use repuestos_autos::application::catalog::{browse_active_products, create_product, AttributeValueInput, BrowseProductsInput, CreateProductInput, ProductActivityFilter, ProductStockFilter};
use repuestos_autos::catalog::open_seeded_catalog;
use repuestos_autos::infrastructure::sqlite::SqliteCatalogRepository;

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
        minimum_sale_price_centavos: 100, opening_quantity: 2,
        attribute_values: vec![AttributeValueInput { definition_id: 12, value: "99".into() }],
    }).unwrap();

    let first = browse(&connection, ProductStockFilter::All, 1, 1);
    assert_eq!(first.products[0].attribute_values.iter().map(|attribute| (attribute.definition_id, attribute.label.as_str(), attribute.value.as_str())).collect::<Vec<_>>(), vec![(5, "Material", ""), (12, "Diameter", "50")]);
    let second = browse(&connection, ProductStockFilter::All, 2, 1);
    assert_eq!(second.products[0].name, "Zulu filter");
    assert_eq!(second.products[0].attribute_values.iter().map(|attribute| (attribute.definition_id, attribute.value.as_str())).collect::<Vec<_>>(), vec![(5, ""), (12, "99")]);
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
