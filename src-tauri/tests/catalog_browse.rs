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

mod exact_sku_preference {
    use super::*;
    use repuestos_autos::application::catalog::{
        browse_active_inventory_products, browse_active_sale_products, search_active_products,
        search_active_sale_products,
    };
    use rusqlite::{types::Value, Connection};

    fn fixture() -> Connection {
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
        connection
    }

    fn input(query: &str) -> BrowseProductsInput {
        BrowseProductsInput {
            query: Some(query.into()),
            category_id: None,
            stock_filter: ProductStockFilter::All,
            activity_filter: ProductActivityFilter::Active,
            page: 1,
            page_size: 1,
        }
    }

    // The three public projections must agree on counts, pages and identities.
    fn browse_results(
        connection: &Connection,
        input: &BrowseProductsInput,
    ) -> [(i64, i64, Vec<i64>); 3] {
        let catalog = browse_active_products(connection, &SqliteCatalogRepository, input).unwrap();
        let inventory = browse_active_inventory_products(connection, input).unwrap();
        let sales = browse_active_sale_products(connection, input).unwrap();
        [
            (
                catalog.total,
                catalog.total_pages,
                catalog.products.iter().map(|p| p.product_id).collect(),
            ),
            (
                inventory.total,
                inventory.total_pages,
                inventory.products.iter().map(|p| p.product_id).collect(),
            ),
            (
                sales.total,
                sales.total_pages,
                sales.products.iter().map(|p| p.product_id).collect(),
            ),
        ]
    }

    #[test]
    fn exact_selection_precedes_counts_and_both_pages() {
        let connection = fixture();
        for page in [1, 2] {
            let input = BrowseProductsInput {
                page,
                ..input("NGK BKR6E-11")
            };
            let expected = if page == 1 { vec![1] } else { vec![] };
            for result in browse_results(&connection, &input) {
                assert_eq!(result, (1, 1, expected.clone()), "page {page}");
            }
        }
        for result in browse_results(&connection, &input("NGK BKR6E-1")) {
            assert_eq!(
                result,
                (2, 2, vec![9701]),
                "no exact SKU keeps broad ordering/counts"
            );
        }
    }

    #[test]
    fn discovery_respects_category_activity_and_active_categories() {
        for (
            exact_category,
            exact_active,
            category_active,
            selected_category,
            activity,
            catalog_ids,
            sales_ids,
        ) in [
            (
                2,
                1,
                1,
                Some(1),
                ProductActivityFilter::Active,
                vec![9701],
                vec![9701],
            ),
            (
                2,
                1,
                1,
                None,
                ProductActivityFilter::Active,
                vec![1],
                vec![1],
            ),
            (
                1,
                0,
                1,
                None,
                ProductActivityFilter::Active,
                vec![9701],
                vec![9701],
            ),
            (
                1,
                0,
                1,
                None,
                ProductActivityFilter::Archived,
                vec![1],
                vec![9701],
            ),
            (
                1,
                0,
                1,
                None,
                ProductActivityFilter::All,
                vec![1],
                vec![9701],
            ),
            (
                2,
                1,
                0,
                None,
                ProductActivityFilter::All,
                vec![9701],
                vec![9701],
            ),
        ] {
            let connection = fixture();
            connection
                .execute(
                    "UPDATE products SET category_id = ?1, active = ?2 WHERE id = 1",
                    rusqlite::params![exact_category, exact_active],
                )
                .unwrap();
            connection
                .execute(
                    "UPDATE categories SET active = ?1 WHERE id = 2",
                    [category_active],
                )
                .unwrap();
            let input = BrowseProductsInput {
                category_id: selected_category,
                activity_filter: activity,
                ..input("NGK BKR6E-11")
            };
            let results = browse_results(&connection, &input);
            for result in &results[..2] {
                assert_eq!(
                    result,
                    &(1, 1, catalog_ids.clone()),
                    "Catalog/Inventory scope"
                );
            }
            assert_eq!(
                results[2],
                (1, 1, sales_ids.clone()),
                "Sales stays active-only"
            );
            // Legacy has no category/activity input, so always uses global active scope.
            let expected = if exact_active == 1 && category_active == 1 {
                vec![1]
            } else {
                vec![9701]
            };
            assert_eq!(
                search_active_products(&connection, "NGK BKR6E-11")
                    .unwrap()
                    .iter()
                    .map(|p| p.product_id)
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                search_active_sale_products(&connection, "NGK BKR6E-11")
                    .unwrap()
                    .iter()
                    .map(|p| p.product_id)
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }

    #[test]
    fn exact_without_a_required_join_does_not_hide_eligible_fallback() {
        for has_stock in [true, false] {
            let connection = fixture();
            connection.execute(
                "INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos)
                 VALUES (9702, 1, 'NGK BKR6E-1', 'Ineligible exact plug', 1, 2500, 2500)",
                [],
            ).unwrap();
            if has_stock {
                connection
                    .execute(
                        "INSERT INTO stock_balances (product_id, quantity) VALUES (9702, 8)",
                        [],
                    )
                    .unwrap();
            } else {
                connection.execute("INSERT INTO catalog_product_search (rowid, product_id, content) VALUES (9702, 9702, 'ngk bkr6e 1')", []).unwrap();
            }
            let input = BrowseProductsInput {
                page_size: 20,
                ..input("NGK BKR6E-1")
            };
            for result in browse_results(&connection, &input) {
                assert_eq!(result, (2, 1, vec![9701, 1]));
            }
            assert_eq!(
                search_active_products(&connection, "NGK BKR6E-1")
                    .unwrap()
                    .iter()
                    .map(|p| p.product_id)
                    .collect::<Vec<_>>(),
                vec![9701, 1]
            );
            assert_eq!(
                search_active_sale_products(&connection, "NGK BKR6E-1")
                    .unwrap()
                    .iter()
                    .map(|p| p.product_id)
                    .collect::<Vec<_>>(),
                vec![9701, 1]
            );
        }
    }

    #[test]
    fn stock_exclusion_never_substitutes_similar_references() {
        let connection = fixture();
        connection
            .execute(
                "UPDATE stock_balances SET quantity = 0 WHERE product_id = 1",
                [],
            )
            .unwrap();
        for stock_filter in [ProductStockFilter::Available, ProductStockFilter::LowStock] {
            let input = BrowseProductsInput {
                stock_filter,
                ..input("NGK BKR6E-11")
            };
            let results = browse_results(&connection, &input);
            assert_eq!(results[0], (0, 0, vec![]));
            assert_eq!(results[1], (0, 0, vec![]));
            assert_eq!(
                results[2],
                (1, 1, vec![1]),
                "Sales still ignores stock filters"
            );
        }
        for stock_filter in [
            ProductStockFilter::All,
            ProductStockFilter::OutOfStock,
            ProductStockFilter::Alerts,
        ] {
            let input = BrowseProductsInput {
                stock_filter,
                ..input("NGK BKR6E-11")
            };
            for result in browse_results(&connection, &input) {
                assert_eq!(result, (1, 1, vec![1]));
            }
        }
    }

    #[test]
    fn numeric_exact_collision_wins_and_absence_keeps_attribute_semantics() {
        let connection = fixture();
        connection.execute_batch(
            "INSERT INTO attribute_definitions (id, category_id, label, field_type, required)
             VALUES (9701, 1, 'Width', 'number', 0), (9702, 1, 'Height', 'number', 0), (9703, 1, 'Length', 'number', 0);
             INSERT INTO product_attribute_values (product_id, definition_id, number_value, searchable_value)
             VALUES (9701, 9701, 28, '28'), (9701, 9702, 28, '28'), (9701, 9703, 58, '58');"
        ).unwrap();
        for query in ["28", "28.5", "28 28 58"] {
            connection
                .execute("UPDATE products SET sku = ?1 WHERE id = 1", [query])
                .unwrap();
            let number = if query == "28.5" { 28.5 } else { 28.0 };
            connection.execute("UPDATE product_attribute_values SET number_value = ?1 WHERE product_id = 9701 AND definition_id = 9701", [number]).unwrap();
            for result in browse_results(&connection, &input(query)) {
                assert_eq!(result, (1, 1, vec![1]), "exact numeric {query}");
            }
            assert_eq!(
                search_active_products(&connection, query)
                    .unwrap()
                    .iter()
                    .map(|p| p.product_id)
                    .collect::<Vec<_>>(),
                vec![1]
            );
            assert_eq!(
                search_active_sale_products(&connection, query)
                    .unwrap()
                    .iter()
                    .map(|p| p.product_id)
                    .collect::<Vec<_>>(),
                vec![1]
            );
            connection
                .execute("UPDATE products SET sku = 'NO-EXACT' WHERE id = 1", [])
                .unwrap();
            for result in browse_results(&connection, &input(query)) {
                assert_eq!(result, (1, 1, vec![9701]), "numeric fallback {query}");
            }
            assert_eq!(
                search_active_products(&connection, query)
                    .unwrap()
                    .iter()
                    .map(|p| p.product_id)
                    .collect::<Vec<_>>(),
                vec![9701]
            );
            assert_eq!(
                search_active_sale_products(&connection, query)
                    .unwrap()
                    .iter()
                    .map(|p| p.product_id)
                    .collect::<Vec<_>>(),
                vec![9701]
            );
        }
    }

    #[test]
    fn original_equality_case_whitespace_and_punctuation_are_not_normalized() {
        let connection = fixture();
        for query in ["ngk bkr6e-11", " NGK BKR6E-11 ", "NGK BKR6E-1"] {
            for result in browse_results(&connection, &input(query)) {
                assert_eq!(result, (2, 2, vec![9701]), "non-exact {query:?}");
            }
            assert_eq!(search_active_products(&connection, query).unwrap().len(), 2);
            assert_eq!(
                search_active_sale_products(&connection, query)
                    .unwrap()
                    .len(),
                2
            );
        }
        connection
            .execute("UPDATE products SET sku = '---' WHERE id = 1", [])
            .unwrap();
        for query in ["---", " --- ", "!!!", "x' OR 1=1 --"] {
            let expected = if query == "---" { vec![1] } else { vec![] };
            for result in browse_results(&connection, &input(query)) {
                let count = expected.len() as i64;
                assert_eq!(result, (count, count, expected.clone()), "{query:?}");
            }
            assert_eq!(
                search_active_products(&connection, query)
                    .unwrap()
                    .iter()
                    .map(|p| p.product_id)
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                search_active_sale_products(&connection, query)
                    .unwrap()
                    .iter()
                    .map(|p| p.product_id)
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }

    fn snapshot(connection: &Connection) -> Vec<Vec<Vec<Value>>> {
        [
            "SELECT * FROM products ORDER BY id",
            "SELECT * FROM categories ORDER BY id",
            "SELECT * FROM stock_balances ORDER BY product_id",
            "SELECT * FROM product_attribute_values ORDER BY product_id, definition_id",
            "SELECT rowid, * FROM catalog_product_search ORDER BY rowid",
            "SELECT * FROM sqlite_schema ORDER BY name",
        ]
        .iter()
        .map(|sql| {
            let mut statement = connection.prepare(sql).unwrap();
            let columns = statement.column_count();
            statement
                .query_map([], |row| {
                    (0..columns)
                        .map(|column| row.get(column))
                        .collect::<rusqlite::Result<Vec<Value>>>()
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
        })
        .collect()
    }

    #[test]
    fn every_search_projection_is_read_only_for_exact_fallback_and_blank_queries() {
        let connection = fixture();
        let before = snapshot(&connection);
        let changes = connection.total_changes();
        connection.execute_batch("PRAGMA query_only = ON").unwrap();
        for query in ["NGK BKR6E-11", "ngk", "28", "---", "", "  "] {
            let _ = browse_results(&connection, &input(query));
            search_active_products(&connection, query).unwrap();
            search_active_sale_products(&connection, query).unwrap();
            if query.trim().is_empty() {
                for result in browse_results(&connection, &input(query)) {
                    assert_eq!(result, (2, 2, vec![9701]));
                }
                assert!(search_active_products(&connection, query)
                    .unwrap()
                    .is_empty());
                assert!(search_active_sale_products(&connection, query)
                    .unwrap()
                    .is_empty());
            }
        }
        assert_eq!(connection.total_changes(), changes);
        assert_eq!(
            snapshot(&connection),
            before,
            "fixture facts, FTS and schema stay unchanged"
        );
    }

    #[test]
    fn lookup_errors_propagate_through_every_public_projection() {
        let connection = Connection::open_in_memory().unwrap();
        let input = input("NGK BKR6E-11");
        assert!(browse_active_products(&connection, &SqliteCatalogRepository, &input).is_err());
        assert!(browse_active_inventory_products(&connection, &input).is_err());
        assert!(browse_active_sale_products(&connection, &input).is_err());
        assert!(search_active_products(&connection, "NGK BKR6E-11").is_err());
        assert!(search_active_sale_products(&connection, "NGK BKR6E-11").is_err());
    }
}

#[test]
fn numeric_matching_precedes_counts_and_pagination_in_every_browse_projection() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (9301, 1, 'Width', 'text', 0), (9302, 1, 'Height', 'text', 0), (9303, 1, 'Length', 'number', 0);").unwrap();
    for (sku, name, values, quantity) in [
        ("58", "A false positive", ["28", "280", "58"], 8),
        ("MATCH-B", "B matching", ["28", "28.0", "58"], 1),
        ("PARTIAL", "C partial", ["28", "model 28", "58"], 8),
        ("MATCH-D", "D matching", ["+2.8e1", "28", "58"], 8),
    ] {
        create_product(&mut connection, CreateProductInput {
            sku: sku.into(), name: name.into(), category_id: 1,
            purchase_price_centavos: 100, sale_price_centavos: 200,
            minimum_sale_price_centavos: 100, low_stock_threshold: None, opening_quantity: quantity,
            attribute_values: vec![
                AttributeValueInput { definition_id: 9301, value: values[0].into() },
                AttributeValueInput { definition_id: 9302, value: values[1].into() },
                AttributeValueInput { definition_id: 9303, value: values[2].into() },
            ],
        }).unwrap();
    }
    // Discover fixture IDs through the public browse seam, not write-adapter details.
    let all = browse(&connection, ProductStockFilter::All, 1, 50);
    let b = all.products.iter().find(|p| p.sku == "MATCH-B").unwrap().product_id;
    let d = all.products.iter().find(|p| p.sku == "MATCH-D").unwrap().product_id;
    connection.execute("UPDATE stock_balances SET quantity = 0 WHERE product_id = ?1", [b]).unwrap();
    for (number, expected) in [(1, b), (2, d)] {
        let input = BrowseProductsInput { query: Some("28 28 58".into()), category_id: Some(1), stock_filter: ProductStockFilter::All, activity_filter: ProductActivityFilter::Active, page: number, page_size: 1 };
        let catalog = browse_active_products(&connection, &SqliteCatalogRepository, &input).unwrap();
        let inventory = repuestos_autos::application::catalog::browse_active_inventory_products(&connection, &input).unwrap();
        let sales = repuestos_autos::application::catalog::browse_active_sale_products(&connection, &input).unwrap();
        for (total, pages, product_ids) in [
            (catalog.total, catalog.total_pages, catalog.products.iter().map(|p| p.product_id).collect::<Vec<_>>()),
            (inventory.total, inventory.total_pages, inventory.products.iter().map(|p| p.product_id).collect::<Vec<_>>()),
            (sales.total, sales.total_pages, sales.products.iter().map(|p| p.product_id).collect::<Vec<_>>()),
        ] {
            assert_eq!((total, pages, product_ids), (2, 2, vec![expected]), "page {number}");
        }
    }
    let input = BrowseProductsInput { query: Some("58 28 28".into()), category_id: Some(1), stock_filter: ProductStockFilter::Available, activity_filter: ProductActivityFilter::Active, page: 1, page_size: 20 };
    let available = browse_active_products(&connection, &SqliteCatalogRepository, &input).unwrap();
    assert_eq!((available.total, available.products[0].product_id), (1, d));
    let other_category = BrowseProductsInput { category_id: Some(2), ..input.clone() };
    assert_eq!(browse_active_products(&connection, &SqliteCatalogRepository, &other_category).unwrap().total, 0);
    connection.execute("UPDATE products SET active = 0 WHERE id = ?1", [d]).unwrap();
    assert_eq!(browse_active_products(&connection, &SqliteCatalogRepository, &input).unwrap().total, 0);
    let archived = BrowseProductsInput { activity_filter: ProductActivityFilter::Archived, ..input.clone() };
    assert_eq!(browse_active_products(&connection, &SqliteCatalogRepository, &archived).unwrap().products[0].product_id, d);
    assert_eq!(repuestos_autos::application::catalog::browse_active_sale_products(&connection, &input).unwrap().total, 1, "Sales retains active-only and existing stock behavior");
    connection.execute("UPDATE categories SET active = 0 WHERE id = 1", []).unwrap();
    assert_eq!(repuestos_autos::application::catalog::browse_active_sale_products(&connection, &input).unwrap().total, 0);
}

#[test]
fn browse_keeps_blank_text_and_original_full_sku_access_in_all_projections() {
    let connection = open_seeded_catalog().unwrap();
    for (sku, query) in [
        ("FLT-001", "   "),
        ("FLT-001", "filt"),
        ("FLT-001", "FLT-001"),
        ("28", "28"),
        ("28.5", "28.5"),
        ("28 28 58", "28 28 58"),
        ("28-28-58", "28-28-58"),
        ("---", "---"),
    ] {
        connection.execute("UPDATE products SET sku = ?1 WHERE id = 1", [sku]).unwrap();
        let input = BrowseProductsInput { query: Some(query.into()), category_id: None, stock_filter: ProductStockFilter::All, activity_filter: ProductActivityFilter::Active, page: 1, page_size: 1 };
        let catalog = browse_active_products(&connection, &SqliteCatalogRepository, &input).unwrap();
        let inventory = repuestos_autos::application::catalog::browse_active_inventory_products(&connection, &input).unwrap();
        let sales = repuestos_autos::application::catalog::browse_active_sale_products(&connection, &input).unwrap();
        assert_eq!((catalog.total, inventory.total, sales.total), (1, 1, 1), "{query}");
        assert_eq!((catalog.products[0].product_id, inventory.products[0].product_id, sales.products[0].product_id), (1, 1, 1), "{query}");
        assert_eq!(catalog.products[0].sku, sku, "SKU spelling is not normalized");
    }
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
fn sales_browse_serializes_current_nullable_cost_without_protected_metadata_or_writes() {
    let mut connection = open_seeded_catalog().unwrap();
    let input = BrowseProductsInput {
        query: Some("FLT".into()), category_id: Some(1), stock_filter: ProductStockFilter::All,
        activity_filter: ProductActivityFilter::Active, page: 1, page_size: 20,
    };
    connection.execute("UPDATE products SET purchase_price_centavos = 3200 WHERE id = 1", []).unwrap();
    let unassigned = serde_json::to_value(repuestos_autos::application::catalog::browse_active_sale_products(&connection, &input).unwrap()).unwrap();
    assert_eq!(unassigned["products"][0]["purchase_price_centavos"], 3200);
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
    for forbidden in ["revision", "primary_location_id", "low_stock_threshold", "activity", "active", "active_product_count", "profit_margin"] {
        assert!(!assigned["products"][0].as_object().unwrap().contains_key(forbidden));
    }
    connection.execute("UPDATE products SET purchase_price_centavos = NULL WHERE id = 1", []).unwrap();
    let changes_before = connection.total_changes();
    let unknown = serde_json::to_value(repuestos_autos::application::catalog::browse_active_sale_products(&connection, &input).unwrap()).unwrap();
    assert!(unknown["products"][0].as_object().unwrap().contains_key("purchase_price_centavos"));
    assert_eq!(unknown["products"][0]["purchase_price_centavos"], serde_json::Value::Null);
    assert_eq!(unknown["total"], 1);
    assert_eq!(connection.total_changes(), changes_before);
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
