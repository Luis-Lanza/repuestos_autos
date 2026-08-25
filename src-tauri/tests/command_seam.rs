use repuestos_autos::commands::catalog::{search_products, SearchProductsRequest};
use repuestos_autos::commands::confirm_sale::{
    confirm_sale, ConfirmSaleRequest, ConfirmSaleResponse, PaymentInputRequest, RequestedLine,
};
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;

fn request(request_id: &str, tendered: Option<i64>, qr_applied: Option<i64>) -> ConfirmSaleRequest {
    ConfirmSaleRequest {
        request_id: request_id.into(),
        lines: vec![RequestedLine {
            product_id: 1,
            quantity: 1,
        }],
        payment: PaymentInputRequest {
            amount_tendered_centavos: tendered,
            qr_applied_centavos: qr_applied,
        },
    }
}

#[test]
fn exposes_catalog_price_without_physical_storage_terminology() {
    let connection = open_seeded_catalog().unwrap();
    let results = search_products(
        &connection,
        SearchProductsRequest {
            query: "Toyota".into(),
        },
    )
    .unwrap();

    assert_eq!(results[0].catalog_unit_price_centavos, 2_500);
}

#[test]
fn confirms_persisted_cash_qr_and_mixed_summaries() {
    for (request_id, tendered, qr_applied, expected_total) in [
        (
            "550e8400-e29b-41d4-a716-446655440040",
            Some(3_000),
            None,
            2_500,
        ),
        (
            "550e8400-e29b-41d4-a716-446655440041",
            None,
            Some(2_500),
            2_500,
        ),
        (
            "550e8400-e29b-41d4-a716-446655440042",
            Some(1_000),
            Some(1_500),
            2_500,
        ),
    ] {
        let mut connection = open_seeded_catalog().unwrap();
        let response =
            confirm_sale(&mut connection, request(request_id, tendered, qr_applied)).unwrap();
        let ConfirmSaleResponse::Success(summary) = response else {
            panic!("expected a persisted summary");
        };
        assert_eq!(summary.total_centavos, expected_total);
        assert_eq!(summary.lines[0].unit_price_centavos, 2_500);
    }
}

#[test]
fn returns_repriced_persisted_summary_for_idempotent_retries() {
    let mut connection = open_seeded_catalog().unwrap();
    let first = confirm_sale(
        &mut connection,
        request("550e8400-e29b-41d4-a716-446655440043", None, Some(2_500)),
    )
    .unwrap();
    connection
        .execute(
            "UPDATE products SET sku = 'REN-999', name = 'Renamed filter' WHERE id = 1",
            [],
        )
        .unwrap();
    let retry = confirm_sale(
        &mut connection,
        request("550e8400-e29b-41d4-a716-446655440043", Some(9_999), None),
    )
    .unwrap();

    assert_eq!(retry, first);
    let ConfirmSaleResponse::Success(summary) = first else {
        panic!("expected a persisted summary");
    };
    assert_eq!(
        (
            summary.status.as_str(),
            summary.outcome,
            summary.total_centavos,
        ),
        ("confirmed", "confirmed", 2_500),
    );
    assert!(!summary.confirmed_at.is_empty());
    assert_eq!(summary.lines[0].sku, "FLT-001");
    assert_eq!(summary.lines[0].product_name, "Filtro de aceite");
}

#[test]
fn rejects_legacy_authority_and_invalid_request_shapes_before_confirmation() {
    let base = r#"{"request_id":"550e8400-e29b-41d4-a716-446655440044","lines":[{"product_id":1,"quantity":1}],"payment":{"amount_tendered_centavos":null,"qr_applied_centavos":2500}}"#;
    let cases = [
        base.replace(
            "\"quantity\":1",
            "\"quantity\":1,\"negotiated_unit_price_centavos\":2500",
        ),
        base.replace("\"payment\":{", "\"payments\":[],\"payment\":{"),
        base.replace(
            "\"qr_applied_centavos\":2500",
            "\"qr_applied_centavos\":2500,\"amount_applied_centavos\":2500",
        ),
        base.replace(
            "\"qr_applied_centavos\":2500",
            "\"qr_applied_centavos\":2500,\"change_given_centavos\":0",
        ),
        base.replace("\"quantity\":1", "\"quantity\":1.5"),
        base.replace(
            "\"qr_applied_centavos\":2500",
            "\"qr_applied_centavos\":9223372036854775808",
        ),
    ];

    for payload in cases {
        assert!(
            serde_json::from_str::<ConfirmSaleRequest>(&payload).is_err(),
            "{payload}"
        );
    }

    let mut connection = open_seeded_catalog().unwrap();
    let ConfirmSaleResponse::Error(error) =
        confirm_sale(&mut connection, request("invalid", None, None)).unwrap()
    else {
        panic!("expected invalid request response");
    };
    assert_eq!(error.code, "invalid_request");
}
