use repuestos_autos::commands::sales_history::{
    list_sales_history, sale_history_detail, ListSalesHistoryRequest, SalesHistoryDetailResponse,
    SalesHistoryListResponse,
};
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;

fn insert_sale(connection: &rusqlite::Connection, id: i64, confirmed_at: &str) {
    connection
        .execute(
            "INSERT INTO sales (id, request_id, status, total_centavos, confirmed_at) VALUES (?1, ?2, 'confirmed', 2500, ?3)",
            rusqlite::params![id, format!("history-command-{id}"), confirmed_at],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO sale_lines (sale_id, product_id, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos, sku_snapshot, product_name_snapshot) VALUES (?1, 1, 1, 2500, 2500, 2500, NULL, NULL)",
            [id],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO sale_payments (sale_id, method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos) VALUES (?1, 'cash', 2500, 3000, 500)",
            [id],
        )
        .unwrap();
}

#[test]
fn sales_history_commands_project_tagged_read_only_outcomes() {
    let connection = open_seeded_catalog().unwrap();
    insert_sale(&connection, 71, "2024-03-10 05:00:00");

    let listed = list_sales_history(
        &connection,
        ListSalesHistoryRequest {
            from_utc: "2024-03-10T05:00:00Z".into(),
            to_exclusive_utc: "2024-03-11T04:00:00Z".into(),
        },
    );
    let SalesHistoryListResponse::Success { sales, has_more } = listed else {
        panic!("expected a tagged list success")
    };
    assert!(!has_more);
    assert_eq!(sales.len(), 1);
    assert_eq!(sales[0].total_centavos.value(), 2_500);
    assert!(matches!(sales[0].payment_methods.as_slice(), [repuestos_autos::application::sales::PaymentMethod::Cash]));

    let detail = sale_history_detail(&connection, 71);
    let SalesHistoryDetailResponse::Success { detail } = detail else {
        panic!("expected a tagged detail success")
    };
    assert_eq!(detail.total_centavos.value(), 2_500);
    assert_eq!(detail.lines[0].sku, None);
    assert_eq!(detail.lines[0].product_name, None);
    assert_eq!(detail.lines[0].quantity.value(), 1);

    let invalid = list_sales_history(
        &connection,
        ListSalesHistoryRequest {
            from_utc: "2024-03-11T04:00:00Z".into(),
            to_exclusive_utc: "2024-03-10T05:00:00Z".into(),
        },
    );
    assert_eq!(
        serde_json::to_value(invalid).unwrap()["code"],
        "invalid_range"
    );

    let missing = sale_history_detail(&connection, 999);
    assert_eq!(
        serde_json::to_value(missing).unwrap()["code"],
        "sale_not_found"
    );
}

#[test]
fn sales_history_commands_hide_persistence_details() {
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    let response = list_sales_history(
        &connection,
        ListSalesHistoryRequest {
            from_utc: "2024-03-10T05:00:00Z".into(),
            to_exclusive_utc: "2024-03-11T04:00:00Z".into(),
        },
    );
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("persistence_failure"));
    assert!(!json.contains("no such table"));
}
