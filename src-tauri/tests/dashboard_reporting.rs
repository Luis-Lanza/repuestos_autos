use repuestos_autos::application::reporting::{DashboardRange, DashboardReader};
use repuestos_autos::infrastructure::sqlite::{dashboard_repository::SqliteDashboardReader, open_seeded_catalog};
use rusqlite::params;

fn range(from: &str, to: &str) -> DashboardRange { DashboardRange::parse(from, to).unwrap() }

fn sale(connection: &rusqlite::Connection, id: i64, when: &str, total: i64, quantity: i64) {
    sale_with_snapshot(connection, id, when, total, quantity, Some("FLT-001"), Some("Filtro de aceite"));
}

fn sale_with_snapshot(connection: &rusqlite::Connection, id: i64, when: &str, total: i64, quantity: i64, sku: Option<&str>, product_name: Option<&str>) {
    connection.execute("INSERT INTO sales (id, request_id, status, total_centavos, confirmed_at) VALUES (?1, ?2, 'confirmed', ?3, ?4)", params![id, format!("sale-{id}"), total, when]).unwrap();
    connection.execute("INSERT INTO sale_lines (id, sale_id, product_id, sku_snapshot, product_name_snapshot, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos) VALUES (?1, ?2, 1, ?3, ?4, ?5, 2500, 2500, ?6)", params![id, id, sku, product_name, quantity, total]).unwrap();
    connection.execute("INSERT INTO sale_payments (sale_id, method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos) VALUES (?1, 'cash', ?2, ?2, 0)", params![id, total]).unwrap();
}

#[test]
fn aggregates_boundaries_returns_and_cancellation_without_double_counting_restoration() {
    let connection = open_seeded_catalog().unwrap();
    sale(&connection, 10, "2024-03-10 05:00:00", 2_500, 3);
    connection.execute("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (101, 'return-101', 'return', 10, 1, X'01', printf('%064d', 1))", []).unwrap();
    connection.execute("INSERT INTO sale_returns (id, sale_id) VALUES (101, 10)", []).unwrap();
    connection.execute("INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, reason) VALUES (201, 1, 10, 10, 'return', 1, NULL)", []).unwrap();
    connection.execute("INSERT INTO sale_return_lines (return_id, sale_id, sale_line_id, product_id, quantity, movement_id) VALUES (101, 10, 10, 1, 1, 201)", []).unwrap();
    sale(&connection, 11, "2024-03-10 06:00:00", 5_000, 2);
    connection.execute("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (102, 'cancel-102', 'cancellation', 11, 1, X'02', printf('%064d', 2))", []).unwrap();
    connection.execute("INSERT INTO sale_cancellations (id, sale_id, reason) VALUES (102, 11, 'correction')", []).unwrap();
    connection.execute("INSERT INTO sale_cancellation_lines (cancellation_id, sale_id, sale_line_id, product_id, restored_quantity) VALUES (102, 11, 11, 1, 0)", []).unwrap();
    sale(&connection, 12, "2024-03-11 04:00:00", 7_500, 1);

    let reader = SqliteDashboardReader::new(&connection);
    let report = reader.read(&range("2024-03-10T05:00:00Z", "2024-03-11T04:00:00Z"), &range("2024-03-01T05:00:00Z", "2024-04-01T04:00:00Z")).unwrap();
    assert_eq!(report.today.metrics.effective_sale_count, 1);
    assert_eq!(report.today.metrics.effective_total_centavos, 2_500);
    assert_eq!(report.today.metrics.net_units_out, 2);
    assert_eq!(report.today.metrics.cancelled_sale_count, 1);
    assert_eq!(report.payment_distribution[0].amount_applied_centavos, 10_000);
    assert_eq!(report.top_products[0].net_units_out, 3);
    assert_eq!(report.recent_sales.iter().map(|sale| sale.sale_id).collect::<Vec<_>>(), vec![12, 11, 10]);
}

#[test]
fn groups_top_products_by_identity_and_uses_latest_non_null_snapshot() {
    let connection = open_seeded_catalog().unwrap();
    connection.execute("UPDATE products SET sku = 'FLT-CURRENT', name = 'Filtro actual' WHERE id = 1", []).unwrap();
    sale_with_snapshot(&connection, 20, "2024-03-10 05:00:00", 5_000, 2, Some("FLT-OLD"), Some("Filtro antiguo"));
    sale_with_snapshot(&connection, 21, "2024-03-10 06:00:00", 7_500, 3, Some("FLT-NEW"), Some("Filtro renombrado"));
    sale_with_snapshot(&connection, 22, "2024-03-10 07:00:00", 2_500, 1, None, None);

    let reader = SqliteDashboardReader::new(&connection);
    let report = reader.read(&range("2024-03-10T05:00:00Z", "2024-03-11T04:00:00Z"), &range("2024-03-01T05:00:00Z", "2024-04-01T04:00:00Z")).unwrap();
    assert_eq!(report.top_products.len(), 1);
    assert_eq!(report.top_products[0].product_id, 1);
    assert_eq!(report.top_products[0].sku, "FLT-NEW");
    assert_eq!(report.top_products[0].product_name, "Filtro renombrado");
    assert_eq!(report.top_products[0].net_units_out, 6);
}

#[test]
fn returns_empty_safe_sections_and_fixed_recent_order() {
    let connection = open_seeded_catalog().unwrap();
    let reader = SqliteDashboardReader::new(&connection);
    let report = reader.read(&range("2024-03-10T05:00:00Z", "2024-03-11T04:00:00Z"), &range("2024-03-01T05:00:00Z", "2024-04-01T04:00:00Z")).unwrap();
    assert_eq!(report.today.metrics.effective_sale_count, 0);
    assert!(report.top_products.is_empty());
    assert!(report.payment_distribution.is_empty());
    assert!(report.recent_sales.is_empty());
}
