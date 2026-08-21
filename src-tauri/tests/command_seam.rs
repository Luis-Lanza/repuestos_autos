use repuestos_autos::commands::catalog::{search_products, SearchProductsRequest};
use repuestos_autos::commands::confirm_sale::{
    confirm_sale, ConfirmSaleRequest, ConfirmSaleResponse, PaymentRequest, RequestedLine,
};
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;
fn request(request_id: &str) -> ConfirmSaleRequest {
    ConfirmSaleRequest {
        request_id: request_id.into(),
        lines: vec![RequestedLine {
            product_id: 1,
            quantity: 1,
            negotiated_unit_price_centavos: 2_500,
        }],
        payments: vec![PaymentRequest::Qr {
            amount_applied_centavos: 2_500,
        }],
    }
}
#[test]
fn exposes_search_dtos_without_floating_point_conversion() {
    let connection = open_seeded_catalog().unwrap();
    let results = search_products(
        &connection,
        SearchProductsRequest {
            query: "Toyota".into(),
        },
    )
    .unwrap();
    assert_eq!(results[0].minimum_unit_price_centavos, 2_500);
}
#[test]
fn confirms_a_persisted_sale_and_reuses_the_original_summary_for_a_retry() {
    let mut connection = open_seeded_catalog().unwrap();
    let first = confirm_sale(
        &mut connection,
        request("550e8400-e29b-41d4-a716-446655440040"),
    )
    .unwrap();
    let retry = confirm_sale(
        &mut connection,
        ConfirmSaleRequest {
            lines: vec![RequestedLine {
                product_id: 1,
                quantity: 2,
                negotiated_unit_price_centavos: 9_999,
            }],
            payments: vec![PaymentRequest::Qr {
                amount_applied_centavos: 19_998,
            }],
            ..request("550e8400-e29b-41d4-a716-446655440040")
        },
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
}
#[test]
fn maps_malformed_and_rejected_requests_to_stable_public_error_codes() {
    let mut connection = open_seeded_catalog().unwrap();
    let cases = [
        (request("not-a-uuid"), "invalid_request"),
        (
            ConfirmSaleRequest {
                lines: vec![RequestedLine {
                    product_id: 1,
                    quantity: 0,
                    negotiated_unit_price_centavos: 2_500,
                }],
                ..request("550e8400-e29b-41d4-a716-446655440041")
            },
            "invalid_quantity",
        ),
        (
            ConfirmSaleRequest {
                lines: vec![RequestedLine {
                    product_id: 1,
                    quantity: 1,
                    negotiated_unit_price_centavos: 2_499,
                }],
                payments: vec![PaymentRequest::Qr {
                    amount_applied_centavos: 2_499,
                }],
                ..request("550e8400-e29b-41d4-a716-446655440042")
            },
            "price_below_minimum",
        ),
    ];
    for (request, code) in cases {
        let ConfirmSaleResponse::Error(error) = confirm_sale(&mut connection, request).unwrap()
        else {
            panic!("expected error response");
        };
        assert_eq!(error.code, code);
        assert!(!error.message.contains("SQLite"));
    }
}
#[test]
fn rejects_non_integer_and_out_of_range_json_shapes_before_command_invocation() {
    for payload in [
        r#"{\"request_id\":\"550e8400-e29b-41d4-a716-446655440043\",\"lines\":[{\"product_id\":1,\"quantity\":1.5,\"negotiated_unit_price_centavos\":2500}],\"payments\":[]}"#,
        r#"{\"request_id\":\"550e8400-e29b-41d4-a716-446655440043\",\"lines\":[{\"product_id\":1,\"quantity\":1,\"negotiated_unit_price_centavos\":9223372036854775808}],\"payments\":[]}"#,
    ] {
        assert!(serde_json::from_str::<ConfirmSaleRequest>(payload).is_err());
    }
}
