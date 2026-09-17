use repuestos_autos::application::catalog::{
    bootstrap_demo_catalog, BootstrapDemoError, BootstrapDemoOutcome,
};
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;

fn count(connection: &rusqlite::Connection, table: &str) -> i64 {
    connection
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

fn catalog_counts(connection: &rusqlite::Connection) -> (i64, i64, i64, i64, i64, i64) {
    (
        count(connection, "categories"),
        count(connection, "products"),
        count(connection, "stock_balances"),
        count(connection, "inventory_movements"),
        count(connection, "catalog_product_search"),
        count(connection, "catalog_audit"),
    )
}

fn assert_refused_after_tamper(sql: &str, complete: bool) {
    let mut connection = open_seeded_catalog().unwrap();
    if complete {
        bootstrap_demo_catalog(&mut connection).unwrap();
    }
    connection.execute_batch(sql).unwrap();
    let before = catalog_counts(&connection);

    assert_eq!(
        bootstrap_demo_catalog(&mut connection).unwrap_err(),
        BootstrapDemoError::Refused
    );
    assert_eq!(catalog_counts(&connection), before);
}

#[test]
fn bootstrap_demo_loads_exact_counts_and_preserves_catalog_invariants() {
    let mut connection = open_seeded_catalog().unwrap();

    let outcome = bootstrap_demo_catalog(&mut connection).unwrap();

    assert_eq!(
        outcome,
        BootstrapDemoOutcome::Loaded(repuestos_autos::application::catalog::BootstrapDemoSummary {
            categories_added: 8,
            products_added: 98,
            products_reactivated: 1,
        })
    );
    assert_eq!(count(&connection, "categories"), 10);
    assert_eq!(count(&connection, "products"), 100);
    assert_eq!(count(&connection, "stock_balances"), 100);
    assert_eq!(count(&connection, "inventory_movements"), 98);
    assert_eq!(count(&connection, "catalog_product_search"), 100);
    assert_eq!(count(&connection, "catalog_audit"), 1);
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM products WHERE id > 2 AND active = 1 AND list_price_centavos > 0 AND minimum_unit_price_centavos > 0 AND minimum_unit_price_centavos <= list_price_centavos",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        98
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM stock_balances WHERE product_id > 2 AND quantity > 0",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        98
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM inventory_movements WHERE product_id > 2 AND movement_type = 'opening_stock' AND quantity_delta > 0 AND sale_id IS NULL AND sale_line_id IS NULL",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        98
    );
    assert_eq!(
        repuestos_autos::catalog::search_active_products(&connection, "FRN-DEMO-001")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT active FROM products WHERE sku = 'BUJ-001'", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap(),
        1
    );
}

#[test]
fn bootstrap_demo_is_idempotent_after_a_complete_recognized_load() {
    let mut connection = open_seeded_catalog().unwrap();
    bootstrap_demo_catalog(&mut connection).unwrap();
    let before = (
        count(&connection, "categories"),
        count(&connection, "products"),
        count(&connection, "inventory_movements"),
        count(&connection, "catalog_audit"),
    );

    let outcome = bootstrap_demo_catalog(&mut connection).unwrap();

    assert_eq!(
        outcome,
        BootstrapDemoOutcome::AlreadyComplete(
            repuestos_autos::application::catalog::BootstrapDemoSummary {
                categories_added: 0,
                products_added: 0,
                products_reactivated: 0,
            }
        )
    );
    assert_eq!(
        (
            count(&connection, "categories"),
            count(&connection, "products"),
            count(&connection, "inventory_movements"),
            count(&connection, "catalog_audit"),
        ),
        before
    );
}

#[test]
fn bootstrap_demo_refuses_user_data_without_mutation() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute("INSERT INTO categories (name) VALUES ('User category')", [])
        .unwrap();
    let before = (
        count(&connection, "categories"),
        count(&connection, "products"),
        count(&connection, "stock_balances"),
        count(&connection, "inventory_movements"),
    );

    let error = bootstrap_demo_catalog(&mut connection).unwrap_err();

    assert_eq!(error, BootstrapDemoError::Refused);
    assert_eq!(
        (
            count(&connection, "categories"),
            count(&connection, "products"),
            count(&connection, "stock_balances"),
            count(&connection, "inventory_movements"),
        ),
        before
    );
}

#[test]
fn bootstrap_demo_pristine_recognition_rejects_tampered_integrity_facts() {
    for sql in [
        "INSERT INTO catalog_audit (entity_type, entity_id, operation, before_json, after_json, revision) VALUES ('product', 1, 'tamper', '{}', '{}', 1)",
        "UPDATE catalog_product_search SET product_id = 99 WHERE rowid = 1",
        "UPDATE catalog_product_search SET content = 'tampered' WHERE rowid = 1",
        "UPDATE stock_balances SET quantity = 9 WHERE product_id = 1",
        "UPDATE product_searchable_values SET value = 'Honda' WHERE product_id = 1 AND field_name = 'vehicle'",
        "UPDATE categories SET name = 'Tampered category' WHERE id = 1",
        "UPDATE categories SET active = 0 WHERE id = 1",
        "UPDATE categories SET revision = 1 WHERE id = 1",
        "UPDATE products SET sku = 'TAMPERED-SKU' WHERE id = 1",
        "UPDATE products SET name = 'Tampered product' WHERE id = 1",
        "UPDATE products SET list_price_centavos = 2600 WHERE id = 1",
        "UPDATE products SET minimum_unit_price_centavos = 2400 WHERE id = 1",
        "UPDATE products SET active = 0 WHERE id = 1",
        "UPDATE products SET revision = 1 WHERE id = 1",
        "UPDATE catalog_product_search SET content = 'tampered' WHERE rowid = 2",
    ] {
        assert_refused_after_tamper(sql, false);
    }
}

#[test]
fn bootstrap_demo_complete_recognition_rejects_tampered_integrity_facts() {
    for sql in [
        "DROP TRIGGER catalog_audit_immutable_update; UPDATE catalog_audit SET after_json = '{}' WHERE id = 1",
        "DROP TRIGGER catalog_audit_immutable_update; UPDATE catalog_audit SET occurred_at = '2025-01-01T00:00:01Z' WHERE id = 1",
        "UPDATE catalog_product_search SET product_id = 99 WHERE rowid = 3",
        "UPDATE catalog_product_search SET content = 'tampered' WHERE rowid = 3",
        "UPDATE stock_balances SET quantity = 99 WHERE product_id = 3",
        "UPDATE product_searchable_values SET value = 'Honda' WHERE product_id = 1 AND field_name = 'vehicle'",
        "UPDATE categories SET name = 'Tampered category' WHERE id = 3",
        "UPDATE categories SET active = 0 WHERE id = 3",
        "UPDATE categories SET revision = 1 WHERE id = 3",
        "UPDATE products SET sku = 'TAMPERED-SKU' WHERE id = 3",
        "UPDATE products SET name = 'Tampered product' WHERE id = 3",
        "UPDATE products SET list_price_centavos = 9999 WHERE id = 3",
        "UPDATE products SET minimum_unit_price_centavos = 1 WHERE id = 3",
        "UPDATE products SET active = 0 WHERE id = 3",
        "UPDATE products SET revision = 1 WHERE id = 3",
        "DROP TRIGGER inventory_movements_immutable_update; UPDATE inventory_movements SET quantity_delta = 99 WHERE id = 1",
        "DROP TRIGGER inventory_movements_immutable_update; UPDATE inventory_movements SET occurred_at = '2025-01-01T00:00:01Z' WHERE id = 1",
        "DROP TRIGGER inventory_movements_immutable_update; UPDATE inventory_movements SET id = 999 WHERE id = 1",
        "DROP TRIGGER inventory_movements_immutable_update; UPDATE inventory_movements SET product_id = 1 WHERE id = 1",
    ] {
        assert_refused_after_tamper(sql, true);
    }
}

#[test]
fn bootstrap_demo_rolls_back_all_facts_when_opening_stock_fails() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER reject_bootstrap_opening BEFORE INSERT ON inventory_movements
             WHEN new.movement_type = 'opening_stock'
             BEGIN SELECT RAISE(ABORT, 'forced bootstrap failure'); END;",
        )
        .unwrap();

    let error = bootstrap_demo_catalog(&mut connection).unwrap_err();

    assert_eq!(error, BootstrapDemoError::Persistence);
    assert_eq!(count(&connection, "categories"), 2);
    assert_eq!(count(&connection, "products"), 2);
    assert_eq!(count(&connection, "stock_balances"), 2);
    assert_eq!(count(&connection, "inventory_movements"), 0);
    assert_eq!(count(&connection, "catalog_audit"), 0);
}
