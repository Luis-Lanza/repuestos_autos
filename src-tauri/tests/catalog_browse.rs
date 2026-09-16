use repuestos_autos::application::catalog::{browse_active_products, BrowseProductsInput, ProductActivityFilter, ProductStockFilter};
use repuestos_autos::catalog::open_seeded_catalog;

fn browse(connection: &rusqlite::Connection, stock_filter: ProductStockFilter, page: i64, page_size: i64) -> repuestos_autos::application::catalog::ProductBrowsePage {
    browse_active_products(connection, &BrowseProductsInput {
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
    assert!(browse_active_products(&connection, &BrowseProductsInput {
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
    let page = browse_active_products(&connection, &BrowseProductsInput {
        query: Some("FLT".into()),
        category_id: None,
        stock_filter: ProductStockFilter::Available,
        activity_filter: ProductActivityFilter::Active,
        page: 1,
        page_size: 20,
    }).expect("catalog browse succeeds");

    assert_eq!(page.products[0].sku, "FLT-001");
}

#[test]
fn browse_rejects_unbounded_page_sizes() {
    let connection = open_seeded_catalog().expect("a disposable catalog database");
    let result = browse_active_products(&connection, &BrowseProductsInput {
        query: None,
        category_id: None,
        stock_filter: ProductStockFilter::All,
        activity_filter: ProductActivityFilter::Active,
        page: 1,
        page_size: 51,
    });

    assert!(result.is_err());
}
