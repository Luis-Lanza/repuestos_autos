use repuestos_autos::{
    application::sales::{confirm_sale, ConfirmSaleRequest, RequestedLine},
    commands::post_sale::{
        cancel_sale, create_sale_return, CancelSaleCommandResponse, CancelSaleRequest,
        CreateSaleReturnRequest, PostSaleCommandResponse, RequestedReturnLine,
    },
    domain::{sales::Payment, MoneyCentavos, Quantity, RequestId},
    infrastructure::sqlite::open_seeded_catalog,
};

fn confirmed_sale(connection: &mut rusqlite::Connection) -> (i64, [i64; 2]) {
    connection
        .execute(
            "INSERT INTO products (id, category_id, sku, name, active, minimum_unit_price_centavos) \
             VALUES (3, 1, 'FLT-002', 'Filter two', 1, 3000)",
            [],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (3, 4)",
            [],
        )
        .unwrap();
    let sale = confirm_sale(
        connection,
        ConfirmSaleRequest {
            request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440031").unwrap(),
            lines: vec![
                RequestedLine {
                    product_id: 1,
                    quantity: Quantity::new(2).unwrap(),
                    negotiated_unit_price: MoneyCentavos::new(2500).unwrap(),
                },
                RequestedLine {
                    product_id: 3,
                    quantity: Quantity::new(1).unwrap(),
                    negotiated_unit_price: MoneyCentavos::new(3000).unwrap(),
                },
            ],
            payments: vec![Payment::cash(
                MoneyCentavos::new(8000).unwrap(),
                MoneyCentavos::new(8000).unwrap(),
                MoneyCentavos::new(0).unwrap(),
            )
            .unwrap()],
        },
    )
    .unwrap();
    let lines = connection
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY id")
        .unwrap()
        .query_map([sale.sale_id], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<i64>, _>>()
        .unwrap();
    (sale.sale_id, [lines[0], lines[1]])
}

fn assert_safe_transport_text(text: &str) {
    const FORBIDDEN: &[&str] = &[
        "sql",
        "schema",
        "driver",
        "select",
        "sqlite",
        "table",
        "constraint",
        "rusqlite",
        "refund",
        "reimbursement",
        "reversal",
        "credit",
        "settlement",
        "payment",
    ];
    assert!(["sql", "schema", "driver"]
        .iter()
        .all(|word| FORBIDDEN.contains(word)));
    assert!(FORBIDDEN
        .iter()
        .all(|word| !text.to_lowercase().contains(word)));
}

#[test]
fn command_dtos_are_owned_and_strict() {
    for value in [
        serde_json::json!({"request_id":"550e8400-e29b-41d4-a716-446655440001","sale_id":1,"lines":[],"extra":true}),
        serde_json::json!({"request_id":"550e8400-e29b-41d4-a716-446655440001","sale_id":1,"lines":[{"sale_line_id":1,"quantity":1,"extra":true}]}),
    ] {
        assert!(serde_json::from_value::<CreateSaleReturnRequest>(value).is_err());
    }
    assert!(serde_json::from_value::<CancelSaleRequest>(serde_json::json!({"request_id":"550e8400-e29b-41d4-a716-446655440002","sale_id":1,"reason":"x","extra":true})).is_err());
    assert_eq!(serde_json::from_value::<CreateSaleReturnRequest>(serde_json::json!({"request_id":"id","sale_id":1,"lines":[{"sale_line_id":1,"quantity":1}]})).unwrap().request_id, "id");
    assert_eq!(
        serde_json::from_value::<CancelSaleRequest>(
            serde_json::json!({"request_id":"id","sale_id":1,"reason":"x"})
        )
        .unwrap()
        .reason,
        "x"
    );
}

#[test]
fn commands_serialize_complete_return_and_cancellation_results() {
    let mut connection = open_seeded_catalog().unwrap();
    let (sale_id, [first, second]) = confirmed_sale(&mut connection);
    let returned = create_sale_return(
        &mut connection,
        CreateSaleReturnRequest {
            request_id: "550E8400-E29B-41D4-A716-446655440002".into(),
            sale_id,
            lines: vec![
                RequestedReturnLine {
                    sale_line_id: second,
                    quantity: 1,
                },
                RequestedReturnLine {
                    sale_line_id: first,
                    quantity: 1,
                },
            ],
        },
    );
    let return_json = serde_json::to_value(&returned).unwrap();
    assert_eq!(return_json["kind"], "success");
    assert_eq!(
        return_json["result"]["request_id"],
        "550e8400-e29b-41d4-a716-446655440002"
    );
    assert_eq!(return_json["result"]["sale_id"], sale_id);
    assert_eq!(return_json["result"]["status"], "confirmed");
    assert!(return_json["result"]["return_id"].as_i64().is_some());
    assert!(return_json["result"]["occurred_at"]
        .as_str()
        .is_some_and(|time| !time.is_empty()));
    assert_eq!(return_json["result"]["lines"].as_array().unwrap().len(), 2);
    assert!(
        matches!(returned, PostSaleCommandResponse::Success { ref result }
        if result.request_id == "550e8400-e29b-41d4-a716-446655440002" && result.sale_id == sale_id
        && result.return_id > 0 && !result.occurred_at.is_empty() && result.status == "confirmed"
        && result.lines.iter().any(|line| line.sale_line_id == first && line.product_id == 1 && line.quantity == 1)
        && result.lines.iter().any(|line| line.sale_line_id == second && line.product_id == 3 && line.quantity == 1))
    );

    let cancelled = cancel_sale(
        &mut connection,
        CancelSaleRequest {
            request_id: "550e8400-e29b-41d4-a716-446655440003".into(),
            sale_id,
            reason: " inventory correction ".into(),
        },
    );
    let cancellation_json = serde_json::to_value(&cancelled).unwrap();
    assert_eq!(cancellation_json["kind"], "success");
    assert_eq!(
        cancellation_json["result"]["request_id"],
        "550e8400-e29b-41d4-a716-446655440003"
    );
    assert_eq!(cancellation_json["result"]["sale_id"], sale_id);
    assert_eq!(cancellation_json["result"]["status"], "cancelled");
    assert_eq!(
        cancellation_json["result"]["reason"],
        "inventory correction"
    );
    assert!(cancellation_json["result"]["cancellation_id"]
        .as_i64()
        .is_some());
    assert!(cancellation_json["result"]["occurred_at"]
        .as_str()
        .is_some_and(|time| !time.is_empty()));
    assert_eq!(
        cancellation_json["result"]["lines"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        matches!(cancelled, CancelSaleCommandResponse::Success { ref result }
        if result.request_id == "550e8400-e29b-41d4-a716-446655440003" && result.sale_id == sale_id
        && result.cancellation_id > 0 && !result.occurred_at.is_empty() && result.status == "cancelled"
        && result.reason == "inventory correction"
        && result.lines.iter().any(|line| line.sale_line_id == first && line.product_id == 1 && line.restored_quantity == 1)
        && result.lines.iter().any(|line| line.sale_line_id == second && line.product_id == 3 && line.restored_quantity == 0))
    );
}

#[test]
fn command_errors_are_tagged_and_do_not_leak_storage_or_money_terms() {
    let mut connection = open_seeded_catalog().unwrap();
    let returned = create_sale_return(
        &mut connection,
        CreateSaleReturnRequest {
            request_id: "550e8400-e29b-41d4-a716-446655440004".into(),
            sale_id: 1,
            lines: vec![RequestedReturnLine {
                sale_line_id: 1,
                quantity: 0,
            }],
        },
    );
    let cancelled = cancel_sale(
        &mut connection,
        CancelSaleRequest {
            request_id: "550e8400-e29b-41d4-a716-446655440005".into(),
            sale_id: 1,
            reason: " ".into(),
        },
    );
    for (response, code) in [
        (serde_json::to_value(returned).unwrap(), "invalid_request"),
        (
            serde_json::to_value(cancelled).unwrap(),
            "cancellation_reason_required",
        ),
    ] {
        assert_eq!(response["kind"], "error");
        assert_eq!(response["code"], code);
        assert!(response.get("message").is_some());
        assert!(response.get("result").is_none());
        assert_safe_transport_text(&response.to_string());
    }
}
