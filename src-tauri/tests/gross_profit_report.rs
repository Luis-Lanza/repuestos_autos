use repuestos_autos::{
    application::reporting::{DashboardRange, DashboardReader, GrossProfitReader},
    commands::gross_profit::{gross_profit, GrossProfitRequest, GrossProfitResponse},
    infrastructure::sqlite::{dashboard_repository::SqliteDashboardReader, open_seeded_catalog},
};
use rusqlite::params;

fn sale(db: &rusqlite::Connection, id: i64, confirmed: &str, quantity: i64, price: i64, cost: Option<i64>) {
    db.execute("INSERT INTO sales (id, request_id, status, total_centavos, confirmed_at) VALUES (?1, ?2, 'confirmed', ?3, ?4)", params![id, format!("sale-{id}"), price * quantity, confirmed]).unwrap();
    db.execute("INSERT INTO sale_lines (id, sale_id, product_id, sku_snapshot, product_name_snapshot, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, list_price_snapshot_centavos, unit_cost_snapshot_centavos, line_total_centavos) VALUES (?1, ?1, 1, 'SKU', 'Part', ?2, ?3, 0, ?3 + 999, ?4, ?5)", params![id, quantity, price, cost, price * quantity]).unwrap();
}
fn returned(db: &rusqlite::Connection, id: i64, sale_id: i64, occurred: &str, quantity: i64) {
    db.execute("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (?1, ?2, 'return', ?3, 1, X'01', printf('%064d', ?1))", params![id + 1000, format!("return-{id}"), sale_id]).unwrap();
    db.execute("INSERT INTO sale_returns (id, sale_id, occurred_at) VALUES (?1, ?2, ?3)", params![id + 1000, sale_id, occurred]).unwrap();
    db.execute("INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta) VALUES (?1, 1, ?2, ?2, 'return', ?3)", params![id + 2000, sale_id, quantity]).unwrap();
    db.execute("INSERT INTO sale_return_lines (return_id, sale_id, sale_line_id, product_id, quantity, movement_id) VALUES (?1, ?2, ?2, 1, ?3, ?4)", params![id + 1000, sale_id, quantity, id + 2000]).unwrap();
}
fn range(from: &str, to: &str) -> DashboardRange { DashboardRange::parse(from, to).unwrap() }

#[test]
fn includes_sale_contribution_and_returns_by_return_event_date_using_snapshots() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 05:00:00", 3, 2500, Some(1000));
    returned(&db, 1, 1, "2024-03-11 05:00:00", 1);
    sale(&db, 2, "2024-03-01 05:00:00", 2, 3000, Some(1500));
    returned(&db, 2, 2, "2024-03-11 06:00:00", 1);
    // Current catalog purchase cost is deliberately unrelated to the immutable snapshots.
    db.execute("UPDATE products SET purchase_price_centavos = 999999 WHERE id = 1", []).unwrap();
    let reader = SqliteDashboardReader::new(&db);
    let report = reader.read_gross_profit(&range("2024-03-10T00:00:00Z", "2024-03-11T00:00:00Z")).unwrap();
    assert_eq!(report.amount_centavos, 4500); // Two sold units: 1,500 each; return events are outside this UTC period.
    let returns = reader.read_gross_profit(&range("2024-03-11T00:00:00Z", "2024-03-12T00:00:00Z")).unwrap();
    assert_eq!(returns.amount_centavos, -3000);
}

#[test]
fn cancelled_lines_are_omitted_and_unknown_costs_are_disclosed_even_when_returned() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 3, "2024-03-10 05:00:00", 1, 2000, None);
    returned(&db, 3, 3, "2024-03-10 06:00:00", 1);
    sale(&db, 4, "2024-03-10 05:00:00", 1, 2000, Some(1000));
    db.execute("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (1004, 'cancel-4', 'cancellation', 4, 1, X'01', printf('%064d', 4))", []).unwrap();
    db.execute("INSERT INTO sale_cancellations (id, sale_id, reason) VALUES (1004, 4, 'void')", []).unwrap();
    db.execute("INSERT INTO sale_cancellation_lines (cancellation_id, sale_id, sale_line_id, product_id, restored_quantity) VALUES (1004, 4, 4, 1, 0)", []).unwrap();
    let report = SqliteDashboardReader::new(&db).read_gross_profit(&range("2024-03-10T00:00:00Z", "2024-03-11T00:00:00Z")).unwrap();
    assert_eq!(report.amount_centavos, 0);
    assert_eq!(report.missing_cost_line_count, 1);
}

#[test]
fn aggregates_multiple_return_events_and_sales_inside_one_period_using_negotiated_prices() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 20, "2024-03-10 05:00:00", 3, 2500, Some(1000));
    returned(&db, 20, 20, "2024-03-10 06:00:00", 1);
    returned(&db, 21, 20, "2024-03-10 07:00:00", 1);
    sale(&db, 21, "2024-03-10 05:00:00", 1, 1800, Some(1000));
    let report = SqliteDashboardReader::new(&db)
        .read_gross_profit(&range("2024-03-10T05:00:00.000Z", "2024-03-11T05:00:00.000Z"))
        .unwrap();
    assert_eq!(report.amount_centavos, 2300); // (2,500 - 1,000) * (3 - 2) + (1,800 - 1,000).
    assert_eq!(report.activity_count, 4); // Two sales and two distinct return events.
}

#[test]
fn includes_exact_start_excludes_exact_end_and_counts_only_in_period_activity() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 30, "2024-03-10 05:00:00", 1, 2000, Some(1000));
    sale(&db, 31, "2024-03-11 05:00:00", 1, 3000, Some(1000));
    returned(&db, 30, 30, "2024-03-11 05:00:00", 1);
    let report = SqliteDashboardReader::new(&db)
        .read_gross_profit(&range("2024-03-10T05:00:00.000Z", "2024-03-11T05:00:00.000Z"))
        .unwrap();
    assert_eq!(report.amount_centavos, 1000);
    assert_eq!(report.activity_count, 1);
}

#[test]
fn zero_margin_known_profit_and_mixed_known_unknown_costs_remain_active() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 40, "2024-03-10 05:00:00", 1, 1000, Some(1000));
    sale(&db, 41, "2024-03-10 05:00:00", 1, 2000, Some(1000));
    sale(&db, 42, "2024-03-10 05:00:00", 1, 2500, None);
    let report = SqliteDashboardReader::new(&db)
        .read_gross_profit(&range("2024-03-10T05:00:00.000Z", "2024-03-11T05:00:00.000Z"))
        .unwrap();
    assert_eq!(report.amount_centavos, 1000);
    assert_eq!(report.missing_cost_line_count, 1);
    assert_eq!(report.activity_count, 3);
}

#[test]
fn checked_arithmetic_and_invalid_persisted_values_fail_closed() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 50, "2024-03-10 05:00:00", 1, 1000, Some(1));
    db.execute_batch("DROP TRIGGER confirmed_sale_lines_immutable_price").unwrap();
    db.execute("UPDATE sale_lines SET quantity = 2, negotiated_unit_price_centavos = ?1 WHERE id = 50", [i64::MAX]).unwrap();
    assert_eq!(
        SqliteDashboardReader::new(&db).read_gross_profit(&range("2024-03-10T05:00:00.000Z", "2024-03-11T05:00:00.000Z")),
        Err(repuestos_autos::application::reporting::ReportingError::PersistedDataInvalid),
    );

    db.execute_batch("PRAGMA ignore_check_constraints = ON").unwrap();
    db.execute("UPDATE sale_lines SET quantity = 1, negotiated_unit_price_centavos = 1000, unit_cost_snapshot_centavos = -1 WHERE id = 50", []).unwrap();
    db.execute_batch("PRAGMA ignore_check_constraints = OFF").unwrap();
    assert_eq!(
        SqliteDashboardReader::new(&db).read_gross_profit(&range("2024-03-10T05:00:00.000Z", "2024-03-11T05:00:00.000Z")),
        Err(repuestos_autos::application::reporting::ReportingError::PersistedDataInvalid),
    );
}

#[test]
fn zero_cost_snapshot_is_corrupt_in_gross_profit_and_dashboard_reporting() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 60, "2024-03-10 05:00:00", 1, 2000, Some(1000));
    db.execute_batch("DROP TRIGGER confirmed_sale_lines_immutable_price; PRAGMA ignore_check_constraints = ON").unwrap();
    db.execute("UPDATE sale_lines SET unit_cost_snapshot_centavos = 0 WHERE id = 60", []).unwrap();
    db.execute_batch("PRAGMA ignore_check_constraints = OFF").unwrap();

    let range = range("2024-03-10T05:00:00.000Z", "2024-03-11T05:00:00.000Z");
    let reader = SqliteDashboardReader::new(&db);
    assert!(matches!(
        reader.read_gross_profit(&range),
        Err(repuestos_autos::application::reporting::ReportingError::PersistedDataInvalid)
    ));
    assert!(matches!(
        reader.read(&range, &range),
        Err(repuestos_autos::application::reporting::ReportingError::PersistedDataInvalid)
    ));
}

#[test]
fn command_returns_bounded_invalid_range_for_reversed_or_noncanonical_bounds() {
    let db = open_seeded_catalog().unwrap();
    for (from_utc, to_exclusive_utc) in [
        ("2024-03-11T00:00:00.000Z", "2024-03-10T00:00:00.000Z"),
        ("2024-03-10T00:00:00Z", "2024-03-11T00:00:00.000Z"),
        ("2024-03-10T00:00:00.123Z", "2024-03-11T00:00:00.000Z"),
        ("2024-03-10T00:00:00.000Z", "2024-03-11T00:00:00.000+00:00"),
    ] {
        assert!(matches!(gross_profit(&db, GrossProfitRequest { from_utc: from_utc.into(), to_exclusive_utc: to_exclusive_utc.into() }), GrossProfitResponse::Error(error) if error.code == "invalid_range"));
    }
    assert!(serde_json::from_str::<GrossProfitRequest>(r#"{"from_utc":"2024-03-10T00:00:00.000Z","to_exclusive_utc":"2024-03-11T00:00:00.000Z","extra":true}"#).is_err());
}
