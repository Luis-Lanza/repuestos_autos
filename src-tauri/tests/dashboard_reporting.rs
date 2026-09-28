use repuestos_autos::application::reporting::{
    DashboardRange, DashboardReader, RealizedGrossProfit,
};
use repuestos_autos::infrastructure::sqlite::{
    dashboard_repository::SqliteDashboardReader, open_seeded_catalog,
};
use rusqlite::params;

fn range(from: &str, to: &str) -> DashboardRange { DashboardRange::parse(from, to).unwrap() }

fn sale(connection: &rusqlite::Connection, id: i64, when: &str, total: i64, quantity: i64) {
    sale_with_snapshot(connection, id, when, total, quantity, Some("FLT-001"), Some("Filtro de aceite"));
}

fn sale_with_snapshot(connection: &rusqlite::Connection, id: i64, when: &str, total: i64, quantity: i64, sku: Option<&str>, product_name: Option<&str>) {
    sale_with_financial_snapshots(
        connection,
        id,
        when,
        total,
        quantity,
        sku,
        product_name,
        2_500,
        None,
    );
}

fn sale_with_financial_snapshots(
    connection: &rusqlite::Connection,
    id: i64,
    when: &str,
    total: i64,
    quantity: i64,
    sku: Option<&str>,
    product_name: Option<&str>,
    unit_price: i64,
    unit_cost: Option<i64>,
) {
    connection.execute("INSERT INTO sales (id, request_id, status, total_centavos, confirmed_at) VALUES (?1, ?2, 'confirmed', ?3, ?4)", params![id, format!("sale-{id}"), total, when]).unwrap();
    connection.execute("INSERT INTO sale_lines (id, sale_id, product_id, sku_snapshot, product_name_snapshot, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, unit_cost_snapshot_centavos, line_total_centavos) VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6, 2500, ?7, ?8)", params![id, id, sku, product_name, quantity, unit_price, unit_cost, total]).unwrap();
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
fn realized_profit_uses_net_units_preserves_losses_and_ignores_cancellations() {
    let connection = open_seeded_catalog().unwrap();
    sale_with_financial_snapshots(
        &connection,
        30,
        "2024-03-10 05:00:00",
        2_500,
        2,
        Some("FLT-001"),
        Some("Filtro de aceite"),
        2_500,
        Some(3_000),
    );
    connection.execute("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (130, 'return-130', 'return', 30, 1, X'01', printf('%064d', 1))", []).unwrap();
    connection
        .execute(
            "INSERT INTO sale_returns (id, sale_id) VALUES (130, 30)",
            [],
        )
        .unwrap();
    connection.execute("INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, reason) VALUES (230, 1, 30, 30, 'return', 1, NULL)", []).unwrap();
    connection.execute("INSERT INTO sale_return_lines (return_id, sale_id, sale_line_id, product_id, quantity, movement_id) VALUES (130, 30, 30, 1, 1, 230)", []).unwrap();
    sale(&connection, 31, "2024-03-10 06:00:00", 2_500, 1);
    connection.execute("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (131, 'cancel-131', 'cancellation', 31, 1, X'02', printf('%064d', 2))", []).unwrap();
    connection
        .execute(
            "INSERT INTO sale_cancellations (id, sale_id, reason) VALUES (131, 31, 'correction')",
            [],
        )
        .unwrap();
    connection.execute("INSERT INTO sale_cancellation_lines (cancellation_id, sale_id, sale_line_id, product_id, restored_quantity) VALUES (131, 31, 31, 1, 0)", []).unwrap();

    let reader = SqliteDashboardReader::new(&connection);
    let report = reader
        .read(
            &range("2024-03-10T05:00:00Z", "2024-03-11T04:00:00Z"),
            &range("2024-03-01T05:00:00Z", "2024-04-01T04:00:00Z"),
        )
        .unwrap();
    assert_eq!(
        report.today.metrics.realized_gross_profit,
        RealizedGrossProfit { amount_centavos: -500, missing_cost_line_count: 0 }
    );
    assert_eq!(report.today.metrics.effective_total_centavos, 2_500);
    assert_eq!(report.today.metrics.realized_gross_profit.missing_cost_line_count, 0);
}

#[test]
fn missing_cost_preserves_known_profit_and_empty_period_is_zero() {
    let connection = open_seeded_catalog().unwrap();
    sale_with_financial_snapshots(
        &connection,
        40,
        "2024-03-10 05:00:00",
        2_500,
        1,
        Some("FLT-001"),
        Some("Filtro de aceite"),
        2_500,
        Some(1_000),
    );
    sale(&connection, 41, "2024-03-10 06:00:00", 2_500, 1);
    connection.execute("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (141, 'return-141', 'return', 41, 1, X'01', printf('%064d', 1))", []).unwrap();
    connection.execute("INSERT INTO sale_returns (id, sale_id) VALUES (141, 41)", []).unwrap();
    connection.execute("INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, reason) VALUES (241, 1, 41, 41, 'return', 1, NULL)", []).unwrap();
    connection.execute("INSERT INTO sale_return_lines (return_id, sale_id, sale_line_id, product_id, quantity, movement_id) VALUES (141, 41, 41, 1, 1, 241)", []).unwrap();
    let reader = SqliteDashboardReader::new(&connection);
    let report = reader
        .read(
            &range("2024-03-10T05:00:00Z", "2024-03-11T04:00:00Z"),
            &range("2024-02-01T00:00:00Z", "2024-03-01T00:00:00Z"),
        )
        .unwrap();
    assert_eq!(
        report.today.metrics.realized_gross_profit,
        RealizedGrossProfit { amount_centavos: 1_500, missing_cost_line_count: 1 }
    );
    assert_eq!(
        report.month.metrics.realized_gross_profit,
        RealizedGrossProfit { amount_centavos: 0, missing_cost_line_count: 0 }
    );
}

#[test]
fn realized_profit_serializes_as_signed_amount_and_missing_line_count() {
    assert_eq!(
        serde_json::to_value(RealizedGrossProfit { amount_centavos: -123, missing_cost_line_count: 2 }).unwrap(),
        serde_json::json!({"amount_centavos": -123, "missing_cost_line_count": 2})
    );
}

#[test]
fn realized_profit_overflow_fails_the_atomic_report() {
    let connection = open_seeded_catalog().unwrap();
    sale_with_financial_snapshots(
        &connection,
        50,
        "2024-03-10 05:00:00",
        0,
        2,
        Some("FLT-001"),
        Some("Filtro de aceite"),
        i64::MAX,
        Some(1),
    );
    let reader = SqliteDashboardReader::new(&connection);
    assert!(reader
        .read(
            &range("2024-03-10T05:00:00Z", "2024-03-11T04:00:00Z"),
            &range("2024-03-01T05:00:00Z", "2024-04-01T00:00:00Z")
        )
        .is_err());
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
