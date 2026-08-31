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
    assert_eq!(
        (sales[0].status.as_str(), sales[0].has_corrections),
        ("confirmed", false)
    );
    assert_eq!(sales[0].total_centavos.value(), 2_500);
    assert!(matches!(
        sales[0].payment_methods.as_slice(),
        [repuestos_autos::application::sales::PaymentMethod::Cash]
    ));

    let detail = sale_history_detail(&connection, 71);
    let SalesHistoryDetailResponse::Success { detail } = detail else {
        panic!("expected a tagged detail success")
    };
    assert_eq!(detail.total_centavos.value(), 2_500);
    assert_eq!(detail.lines[0].sku, None);
    assert_eq!(detail.lines[0].product_name, None);
    assert_eq!(detail.lines[0].quantity.value(), 1);
    assert_eq!(
        (
            detail.lines[0].returned_quantity,
            detail.lines[0].cancellation_restored_quantity,
            detail.lines[0].remaining_returnable_quantity,
        ),
        (0, 0, 1),
    );
    assert!(detail.returns.is_empty());
    assert!(detail.cancellation.is_none());

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

fn request_id(value: &str) -> repuestos_autos::domain::RequestId {
    repuestos_autos::domain::RequestId::parse(value).unwrap()
}

fn corrected_sale(connection: &mut rusqlite::Connection) -> i64 {
    use repuestos_autos::application::sales::{
        confirm_sale, CancelSaleRequest, ConfirmSaleRequest, CreateReturnRequest,
        PostSaleLifecycleUseCase, PostSaleUseCase, RequestedLine,
    };
    use repuestos_autos::domain::sales::{Payment, RequestedReturnLine};
    use repuestos_autos::domain::{MoneyCentavos, Quantity};
    use repuestos_autos::infrastructure::sqlite::{
        SqlitePostSaleRepository, SqlitePostSaleTransactionFactory,
    };

    let sale_id = confirm_sale(
        connection,
        ConfirmSaleRequest {
            request_id: request_id("550e8400-e29b-41d4-a716-446655440201"),
            lines: vec![RequestedLine {
                product_id: 1,
                quantity: Quantity::new(2).unwrap(),
                negotiated_unit_price: MoneyCentavos::new(2500).unwrap(),
            }],
            payments: vec![Payment::cash(
                MoneyCentavos::new(5000).unwrap(),
                MoneyCentavos::new(5000).unwrap(),
                MoneyCentavos::new(0).unwrap(),
            )
            .unwrap()],
        },
    )
    .unwrap()
    .sale_id;
    let line_id = connection
        .query_row(
            "SELECT id FROM sale_lines WHERE sale_id = ?1",
            [sale_id],
            |row| row.get(0),
        )
        .unwrap();
    let repository = SqlitePostSaleRepository;
    let mut transactions = SqlitePostSaleTransactionFactory::new(connection);
    let mut use_case = PostSaleUseCase::new(&mut transactions, &repository);
    for request in [
        "550e8400-e29b-41d4-a716-446655440202",
        "550e8400-e29b-41d4-a716-446655440203",
    ] {
        use_case
            .create_return(
                CreateReturnRequest::new(
                    request_id(request),
                    sale_id,
                    vec![RequestedReturnLine {
                        sale_line_id: line_id,
                        quantity: 1,
                    }],
                )
                .unwrap(),
            )
            .unwrap();
    }
    use_case
        .cancel_sale(
            CancelSaleRequest::new(
                request_id("550e8400-e29b-41d4-a716-446655440204"),
                sale_id,
                "inventory count".into(),
            )
            .unwrap(),
        )
        .unwrap();
    sale_id
}

#[test]
fn history_commands_serialize_lifecycle_corrections_without_transport_leakage() {
    let mut connection = open_seeded_catalog().unwrap();
    let sale_id = corrected_sale(&mut connection);
    let listed = serde_json::to_value(list_sales_history(
        &connection,
        ListSalesHistoryRequest {
            from_utc: "2000-01-01T00:00:00Z".into(),
            to_exclusive_utc: "2100-01-01T00:00:00Z".into(),
        },
    ))
    .unwrap();
    let detail = serde_json::to_value(sale_history_detail(&connection, sale_id)).unwrap();

    assert_eq!(
        (
            listed["kind"].as_str(),
            listed["sales"][0]["status"].as_str(),
            listed["sales"][0]["has_corrections"].as_bool()
        ),
        (Some("success"), Some("cancelled"), Some(true))
    );
    assert_eq!(
        (
            detail["detail"]["lines"][0]["sale_line_id"].as_i64(),
            detail["detail"]["lines"][0]["returned_quantity"].as_i64(),
            detail["detail"]["lines"][0]["cancellation_restored_quantity"].as_i64(),
            detail["detail"]["lines"][0]["remaining_returnable_quantity"].as_i64()
        ),
        (Some(1), Some(2), Some(0), Some(0))
    );
    assert_eq!(
        detail["detail"]["returns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|returned| returned["request_id"].as_str())
            .collect::<Vec<_>>(),
        [
            Some("550e8400-e29b-41d4-a716-446655440202"),
            Some("550e8400-e29b-41d4-a716-446655440203")
        ]
    );
    let returns = detail["detail"]["returns"].as_array().unwrap();
    assert!(returns.iter().all(
        |returned| returned["return_id"].as_i64().is_some_and(|id| id > 0)
            && returned["occurred_at"]
                .as_str()
                .is_some_and(|time| !time.is_empty())
    ));
    assert_eq!(
        returns
            .iter()
            .flat_map(|returned| returned["lines"].as_array().unwrap())
            .map(|line| (
                line["sale_line_id"].as_i64(),
                line["product_id"].as_i64(),
                line["quantity"].as_i64()
            ))
            .collect::<Vec<_>>(),
        [(Some(1), Some(1), Some(1)), (Some(1), Some(1), Some(1))]
    );
    assert_eq!(
        (
            detail["detail"]["cancellation"]["request_id"].as_str(),
            detail["detail"]["cancellation"]["reason"].as_str(),
            detail["detail"]["cancellation"]["lines"][0]["restored_quantity"].as_i64()
        ),
        (
            Some("550e8400-e29b-41d4-a716-446655440204"),
            Some("inventory count"),
            Some(0)
        )
    );
    assert_eq!(
        (
            detail["detail"]["cancellation"]["lines"][0]["sale_line_id"].as_i64(),
            detail["detail"]["cancellation"]["lines"][0]["product_id"].as_i64(),
            detail["detail"]["cancellation"]["lines"][0]["restored_quantity"].as_i64()
        ),
        (Some(1), Some(1), Some(0))
    );
    assert!(
        detail["detail"]["cancellation"]["cancellation_id"]
            .as_i64()
            .is_some_and(|id| id > 0)
            && detail["detail"]["cancellation"]["occurred_at"]
                .as_str()
                .is_some_and(|time| !time.is_empty())
    );
    assert_eq!(
        (
            detail["detail"]["total_centavos"].as_i64(),
            detail["detail"]["payments"][0]["method"].as_str(),
            detail["detail"]["payments"][0]["amount_applied_centavos"].as_i64(),
            detail["detail"]["payments"][0]["amount_tendered_centavos"].as_i64(),
            detail["detail"]["payments"][0]["change_given_centavos"].as_i64()
        ),
        (Some(5000), Some("cash"), Some(5000), Some(5000), Some(0))
    );
    let transport = detail.to_string().to_lowercase();
    assert!([
        "sql",
        "schema",
        "driver",
        "refund",
        "reimbursement",
        "reversal",
        "credit",
        "settlement"
    ]
    .iter()
    .all(|word| !transport.contains(word)));
}
