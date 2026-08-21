use repuestos_autos::catalog::open_seeded_catalog;

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
        assert_eq!(product.minimum_unit_price_centavos, 2_500);
    }
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
