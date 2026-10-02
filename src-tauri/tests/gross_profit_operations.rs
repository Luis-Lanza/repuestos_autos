use repuestos_autos::{
    application::reporting::GrossProfitOperationKind,
    commands::gross_profit_operations::{
        gross_profit_operations, GrossProfitOperationsRequest, GrossProfitOperationsResponse,
    },
    infrastructure::sqlite::open_seeded_catalog,
};
use rusqlite::params;

fn sale(db: &rusqlite::Connection, id: i64, confirmed: &str, quantity: i64, price: i64, cost: Option<i64>) {
    db.execute("INSERT INTO sales (id, request_id, status, total_centavos, confirmed_at) VALUES (?1, ?2, 'confirmed', ?3, ?4)", params![id, format!("sale-{id}"), price * quantity, confirmed]).unwrap();
    db.execute("INSERT INTO sale_lines (id, sale_id, product_id, sku_snapshot, product_name_snapshot, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, list_price_snapshot_centavos, unit_cost_snapshot_centavos, line_total_centavos) VALUES (?1, ?1, 1, 'OLD-SKU', 'Historic part', ?2, ?3, 0, ?3, ?4, ?5)", params![id, quantity, price, cost, price * quantity]).unwrap();
}

fn returned(db: &rusqlite::Connection, return_id: i64, sale_id: i64, occurred: &str, quantity: i64) {
    db.execute("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (?1, ?2, 'return', ?3, 1, X'01', printf('%064d', ?1))", params![return_id, format!("return-{return_id}"), sale_id]).unwrap();
    db.execute("INSERT INTO sale_returns (id, sale_id, occurred_at) VALUES (?1, ?2, ?3)", params![return_id, sale_id, occurred]).unwrap();
    let movement_id = return_id + 10_000;
    db.execute("INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta) VALUES (?1, 1, ?2, ?2, 'return', ?3)", params![movement_id, sale_id, quantity]).unwrap();
    db.execute("INSERT INTO sale_return_lines (return_id, sale_id, sale_line_id, product_id, quantity, movement_id) VALUES (?1, ?2, ?2, 1, ?3, ?4)", params![return_id, sale_id, quantity, movement_id]).unwrap();
}

fn request(from: &str, to: &str, page: i64, page_size: i64) -> GrossProfitOperationsRequest {
    GrossProfitOperationsRequest { from_utc: from.into(), to_exclusive_utc: to.into(), page, page_size }
}
fn rows(response: GrossProfitOperationsResponse) -> repuestos_autos::application::reporting::GrossProfitOperationsPage {
    match response { GrossProfitOperationsResponse::Success { report } => report, other => panic!("unexpected response: {other:?}") }
}

#[test]
fn sale_and_return_events_obey_inclusive_start_exclusive_end_and_returns_use_their_own_date() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 05:00:00", 3, 2500, Some(1000));
    sale(&db, 2, "2024-03-11 05:00:00", 1, 3000, Some(1000));
    returned(&db, 10, 1, "2024-03-11 05:00:00", 1);
    returned(&db, 11, 1, "2024-03-12 05:00:00", 1);
    let page = rows(gross_profit_operations(&db, request("2024-03-10T05:00:00.000Z", "2024-03-11T05:00:00.000Z", 1, 20)));
    assert_eq!(page.total, 1);
    assert_eq!(page.rows[0].operation_kind, GrossProfitOperationKind::Sale);
    let json = serde_json::to_value(&page).unwrap();
    assert_eq!(json["rows"][0]["operation_kind"], "venta");
    assert_eq!(json["rows"][0]["cost_state"], "known");
    assert!(!json.to_string().contains("path"));
    let next = rows(gross_profit_operations(&db, request("2024-03-11T05:00:00.000Z", "2024-03-12T05:00:00.000Z", 1, 20)));
    assert_eq!(next.total, 2);
    assert_eq!(next.rows[0].sale_id, 2);
    assert_eq!(next.rows[0].operation_kind, GrossProfitOperationKind::Sale);
    assert_eq!(next.rows[1].return_id, Some(10));
    assert_eq!(next.rows[1].signed_quantity, -1);
    let late = rows(gross_profit_operations(&db, request("2024-03-12T05:00:00.000Z", "2024-03-13T05:00:00.000Z", 1, 20)));
    assert_eq!(late.rows[0].return_id, Some(11));
}

#[test]
fn rows_use_historical_snapshots_unknown_cost_state_and_signed_loss_profit() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 05:00:00", 2, 500, Some(1000));
    sale(&db, 2, "2024-03-10 06:00:00", 1, 2500, None);
    returned(&db, 10, 1, "2024-03-10 07:00:00", 1);
    db.execute("UPDATE products SET sku = 'NEW-SKU', name = 'Current part', purchase_price_centavos = 9000 WHERE id = 1", []).unwrap();
    let page = rows(gross_profit_operations(&db, request("2024-03-10T00:00:00.000Z", "2024-03-11T00:00:00.000Z", 1, 20)));
    let unknown = page.rows.iter().find(|row| row.sale_id == 2).unwrap();
    assert_eq!(unknown.product_name, "Historic part");
    assert_eq!(unknown.sku, "OLD-SKU");
    assert_eq!(unknown.signed_gross_profit_centavos, None);
    assert_eq!(unknown.cost_state, repuestos_autos::application::reporting::GrossProfitCostState::Unknown);
    let sale_row = page.rows.iter().find(|row| row.sale_id == 1 && row.return_id.is_none()).unwrap();
    assert_eq!(sale_row.signed_gross_profit_centavos, Some(-1000));
    let return_row = page.rows.iter().find(|row| row.return_id == Some(10)).unwrap();
    assert_eq!(return_row.signed_gross_profit_centavos, Some(500));
}

#[test]
fn cancelled_sales_and_their_returns_are_excluded_and_multiple_partial_returns_are_separate() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 05:00:00", 4, 2000, Some(1000));
    returned(&db, 10, 1, "2024-03-10 06:00:00", 1);
    returned(&db, 11, 1, "2024-03-10 07:00:00", 2);
    sale(&db, 2, "2024-03-10 05:00:00", 1, 2000, Some(1000));
    db.execute("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (20, 'cancel-2', 'cancellation', 2, 1, X'01', printf('%064d', 20))", []).unwrap();
    db.execute("INSERT INTO sale_cancellations (id, sale_id, reason) VALUES (20, 2, 'void')", []).unwrap();
    db.execute("INSERT INTO sale_cancellation_lines (cancellation_id, sale_id, sale_line_id, product_id, restored_quantity) VALUES (20, 2, 2, 1, 0)", []).unwrap();
    let page = rows(gross_profit_operations(&db, request("2024-03-10T00:00:00.000Z", "2024-03-11T00:00:00.000Z", 1, 20)));
    assert_eq!(page.total, 3);
    assert!(page.rows.iter().all(|row| row.sale_id == 1));
    assert_eq!(page.rows.iter().filter(|row| row.return_id.is_some()).count(), 2);
}

#[test]
fn deterministic_newest_first_order_and_pagination_report_exact_totals() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 05:00:00", 1, 2000, Some(1000));
    sale(&db, 2, "2024-03-10 06:00:00", 1, 2000, Some(1000));
    returned(&db, 10, 1, "2024-03-10 07:00:00", 1);
    let first = rows(gross_profit_operations(&db, request("2024-03-10T00:00:00.000Z", "2024-03-11T00:00:00.000Z", 1, 2)));
    assert_eq!(first.total, 3);
    assert_eq!(first.total_pages, 2);
    assert_eq!(first.rows[0].return_id, Some(10));
    assert_eq!(first.rows[1].sale_id, 2);
    let second = rows(gross_profit_operations(&db, request("2024-03-10T00:00:00.000Z", "2024-03-11T00:00:00.000Z", 2, 2)));
    assert_eq!(second.rows.len(), 1);
    assert_eq!(second.rows[0].sale_id, 1);
    let empty = rows(gross_profit_operations(&db, request("2024-03-10T00:00:00.000Z", "2024-03-11T00:00:00.000Z", 4, 1)));
    assert_eq!((empty.total, empty.total_pages, empty.rows.len()), (3, 3, 0));
}

#[test]
fn invalid_bounds_pagination_unknown_fields_and_zero_snapshot_cost_are_rejected_safely() {
    let db = open_seeded_catalog().unwrap();
    for (from, to, page, size) in [
        ("2024-03-10T00:00:00Z", "2024-03-11T00:00:00.000Z", 1, 20),
        ("2024-03-11T00:00:00.000Z", "2024-03-10T00:00:00.000Z", 1, 20),
        ("2024-03-10T00:00:00.000Z", "2024-03-11T00:00:00.000Z", 0, 20),
        ("2024-03-10T00:00:00.000Z", "2024-03-11T00:00:00.000Z", 1, 101),
    ] {
        assert!(matches!(gross_profit_operations(&db, request(from, to, page, size)), GrossProfitOperationsResponse::Error(error) if error.code == "invalid_request"));
    }
    assert!(serde_json::from_str::<GrossProfitOperationsRequest>(r#"{"from_utc":"2024-03-10T00:00:00.000Z","to_exclusive_utc":"2024-03-11T00:00:00.000Z","page":1,"page_size":20,"path":"bad"}"#).is_err());
    sale(&db, 1, "2024-03-10 05:00:00", 1, 2000, Some(1000));
    db.execute_batch("DROP TRIGGER confirmed_sale_lines_immutable_price; PRAGMA ignore_check_constraints = ON").unwrap();
    db.execute("UPDATE sale_lines SET unit_cost_snapshot_centavos = 0 WHERE id = 1", []).unwrap();
    db.execute_batch("PRAGMA ignore_check_constraints = OFF").unwrap();
    assert!(matches!(gross_profit_operations(&db, request("2024-03-10T00:00:00.000Z", "2024-03-11T00:00:00.000Z", 1, 20)), GrossProfitOperationsResponse::Error(error) if error.code == "persistence_failure"));
}

#[test]
fn corrupt_return_quantity_and_arithmetic_overflow_fail_closed() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 05:00:00", 2, 2000, Some(1000));
    db.execute_batch("DROP TRIGGER confirmed_sale_lines_immutable_price; PRAGMA ignore_check_constraints = ON").unwrap();
    db.execute("UPDATE sale_lines SET quantity = 2, negotiated_unit_price_centavos = ?1 WHERE id = 1", [i64::MAX]).unwrap();
    db.execute_batch("PRAGMA ignore_check_constraints = OFF").unwrap();
    assert!(matches!(gross_profit_operations(&db, request("2024-03-10T00:00:00.000Z", "2024-03-11T00:00:00.000Z", 1, 20)), GrossProfitOperationsResponse::Error(error) if error.code == "persistence_failure"));
    assert!(repuestos_autos::application::reporting::GrossProfitOperationsPagination::validate(i64::MAX, 100).is_err());
}
