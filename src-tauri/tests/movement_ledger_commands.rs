use repuestos_autos::{
    commands::movement_ledger::{list_movement_ledger, list_movement_ledger_product_options, MovementLedgerProductOptionsRequest, MovementLedgerRequest, MovementLedgerResponse},
    infrastructure::sqlite::open_seeded_catalog,
};
use serde_json::json;

fn request() -> MovementLedgerRequest {
    MovementLedgerRequest {
        from_utc: "2025-01-01T00:00:00Z".into(),
        to_exclusive_utc: "2025-01-03T00:00:00Z".into(),
        product_id: None,
        movement_type: None,
        page: 1,
        page_size: 50,
    }
}

fn code(response: MovementLedgerResponse) -> &'static str {
    match response {
        MovementLedgerResponse::Error(error) => error.code,
        MovementLedgerResponse::Success { .. } => panic!("expected an error response"),
    }
}

#[test]
fn command_returns_stable_errors_for_malformed_or_invalid_requests() {
    let connection = open_seeded_catalog().unwrap();
    let unknown = json!({
        "from_utc": "2025-01-01T00:00:00Z",
        "to_exclusive_utc": "2025-01-03T00:00:00Z",
        "unexpected": true
    });
    assert!(serde_json::from_value::<MovementLedgerRequest>(unknown).is_err());
    assert!(serde_json::from_value::<MovementLedgerRequest>(json!([])).is_err());

    let mut invalid_range = request();
    invalid_range.from_utc = "not a date".into();
    assert_eq!(code(list_movement_ledger(&connection, invalid_range)), "invalid_range");

    for movement_type in ["opening_stock", "stock_entry", "sale", "return", "adjustment", "cancellation"] {
        let filtered = MovementLedgerRequest { movement_type: Some(movement_type.into()), ..request() };
        assert!(matches!(list_movement_ledger(&connection, filtered), MovementLedgerResponse::Success { .. }), "{movement_type}");
    }
    let invalid_filter = MovementLedgerRequest { movement_type: Some("invented_type".into()), ..request() };
    assert_eq!(code(list_movement_ledger(&connection, invalid_filter)), "invalid_filter");
    let invalid_product = MovementLedgerRequest { product_id: Some(0), ..request() };
    assert_eq!(code(list_movement_ledger(&connection, invalid_product)), "invalid_filter");

    let invalid_page = MovementLedgerRequest { page: 0, ..request() };
    assert_eq!(code(list_movement_ledger(&connection, invalid_page)), "invalid_page");
}

#[test]
fn product_option_contract_includes_archived_catalog_rows() {
    let connection = open_seeded_catalog().unwrap();
    connection.execute("UPDATE products SET active = 0 WHERE id = 1", []).unwrap();
    connection.execute("UPDATE categories SET active = 0 WHERE id = 1", []).unwrap();
    let request = MovementLedgerProductOptionsRequest { query: "FLT-001".into(), page: 1, page_size: 20 };
    let response = list_movement_ledger_product_options(&connection, request);
    let value = serde_json::to_value(response).unwrap();
    assert_eq!(value["kind"], "success");
    assert_eq!(value["page"], 1);
    assert_eq!(value["page_size"], 20);
    assert_eq!(value["has_more"], false);
    let product = value["products"].as_array().unwrap().iter().find(|item| item["product_id"] == 1).unwrap();
    assert_eq!(product["active"], false);
    assert_eq!(product["product_sku"], "FLT-001");

    let malformed = json!({ "query": "filtro", "page": 1, "page_size": 20, "activity": "active" });
    assert!(serde_json::from_value::<MovementLedgerProductOptionsRequest>(malformed).is_err());
    let bad_page = MovementLedgerProductOptionsRequest { query: "filtro".into(), page: 0, page_size: 20 };
    let invalid = serde_json::to_value(list_movement_ledger_product_options(&connection, bad_page)).unwrap();
    assert_eq!(invalid["code"], "invalid_page");
    let long_query = MovementLedgerProductOptionsRequest { query: "x".repeat(101), page: 1, page_size: 20 };
    let invalid = serde_json::to_value(list_movement_ledger_product_options(&connection, long_query)).unwrap();
    assert_eq!(invalid["code"], "invalid_filter");
}

#[test]
fn command_projects_a_read_only_success_with_paging_metadata() {
    let connection = open_seeded_catalog().unwrap();
    let response = list_movement_ledger(&connection, request());
    let json = serde_json::to_value(response).unwrap();
    assert_eq!(json["kind"], "success");
    assert_eq!(json["page"], 1);
    assert_eq!(json["page_size"], 50);
    assert_eq!(json["rows"], json!([]));
    assert_eq!(json["has_more"], false);
}
