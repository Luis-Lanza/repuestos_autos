use repuestos_autos::domain::sales::{
    Payment, PaymentBreakdown, PaymentError, PaymentInput, Sale, SaleError, SaleLine,
};
use repuestos_autos::domain::{MoneyCentavos, Quantity, RequestId};

fn money(value: i64) -> MoneyCentavos {
    MoneyCentavos::new(value).unwrap()
}

fn line(unit_price: i64, quantity: i64) -> SaleLine {
    SaleLine::priced(1, Quantity::new(quantity).unwrap(), money(unit_price)).unwrap()
}

fn derive(
    total: i64,
    tendered: Option<i64>,
    qr: Option<i64>,
) -> Result<PaymentBreakdown, PaymentError> {
    PaymentBreakdown::derive(
        money(total),
        PaymentInput {
            amount_tendered: tendered.map(money),
            qr_applied: qr.map(money),
        },
    )
}

#[test]
fn value_objects_accept_valid_values_and_reject_invalid_values() {
    assert_eq!(Quantity::new(2).unwrap().value(), 2);
    assert!(Quantity::new(0).is_err());
    assert!(Quantity::new(-1).is_err());
    assert_eq!(money(0).value(), 0);
    assert!(MoneyCentavos::new(-1).is_err());
}

#[test]
fn request_id_requires_a_valid_uuid() {
    assert_eq!(
        RequestId::parse("550e8400-e29b-41d4-a716-446655440000")
            .unwrap()
            .as_uuid()
            .to_string(),
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert!(RequestId::parse("not-a-uuid").is_err());
}

#[test]
fn priced_line_exposes_the_authoritative_unit_price_and_checked_total() {
    let priced = line(2_500, 2);
    assert_eq!(priced.product_id(), 1);
    assert_eq!(priced.quantity(), Quantity::new(2).unwrap());
    assert_eq!(priced.unit_price(), money(2_500));
    assert_eq!(priced.total(), money(5_000));
}

#[test]
fn agreed_line_keeps_final_list_and_minimum_prices_independent() {
    let agreed = SaleLine::agreed(
        1,
        Quantity::new(2).unwrap(),
        money(2_750),
        Some(money(3_000)),
        money(2_500),
    )
    .unwrap();

    assert_eq!(agreed.unit_price(), money(2_750));
    assert_eq!(agreed.list_price_snapshot(), Some(money(3_000)));
    assert_eq!(agreed.minimum_unit_price_snapshot(), money(2_500));
    assert_eq!(agreed.total(), money(5_500));
}

#[test]
fn agreed_line_rejects_minimum_price_above_list_price() {
    assert_eq!(
        SaleLine::agreed(
            1,
            Quantity::new(1).unwrap(),
            money(3_500),
            Some(money(3_000)),
            money(3_001),
        ),
        Err(SaleError::MinimumPriceAboveListPrice),
    );
}

#[test]
fn agreed_line_accepts_minimum_equal_to_list_price() {
    let agreed = SaleLine::agreed(
        1,
        Quantity::new(1).unwrap(),
        money(3_000),
        Some(money(3_000)),
        money(3_000),
    )
    .unwrap();

    assert_eq!(agreed.list_price_snapshot(), Some(money(3_000)));
}

#[test]
fn agreed_line_accepts_legacy_missing_list_price_snapshot() {
    let agreed = SaleLine::agreed(
        1,
        Quantity::new(1).unwrap(),
        money(3_000),
        None,
        money(3_000),
    )
    .unwrap();

    assert_eq!(agreed.list_price_snapshot(), None);
}

#[test]
fn agreed_line_rejects_non_positive_or_below_minimum_final_prices() {
    assert!(SaleLine::agreed(
        1,
        Quantity::new(1).unwrap(),
        money(0),
        Some(money(3_000)),
        money(2_500),
    )
    .is_err());
    assert!(SaleLine::agreed(
        1,
        Quantity::new(1).unwrap(),
        money(2_499),
        Some(money(3_000)),
        money(2_500),
    )
    .is_err());
}

#[test]
fn priced_line_rejects_checked_total_overflow() {
    assert!(SaleLine::priced(1, Quantity::new(2).unwrap(), money(i64::MAX)).is_err());
}

#[test]
fn cash_only_derives_exact_applied_amount_and_change() {
    assert_eq!(
        derive(5_000, Some(6_000), None).unwrap().payments(),
        &[Payment::Cash {
            amount_applied: money(5_000),
            amount_tendered: money(6_000),
            change_given: money(1_000),
        }]
    );
}

#[test]
fn cash_only_exact_tender_has_zero_change() {
    assert_eq!(
        derive(5_000, Some(5_000), None).unwrap().payments(),
        &[Payment::Cash {
            amount_applied: money(5_000),
            amount_tendered: money(5_000),
            change_given: money(0),
        }]
    );
}

#[test]
fn qr_only_derives_one_qr_payment() {
    assert_eq!(
        derive(5_000, None, Some(5_000)).unwrap().payments(),
        &[Payment::Qr {
            amount_applied: money(5_000)
        }]
    );
}

#[test]
fn mixed_payment_emits_qr_before_cash_and_derives_exact_or_change() {
    assert_eq!(
        derive(5_000, Some(3_000), Some(2_000)).unwrap().payments()[1],
        Payment::Cash {
            amount_applied: money(3_000),
            amount_tendered: money(3_000),
            change_given: money(0)
        }
    );
    assert_eq!(
        derive(5_000, Some(4_000), Some(2_000)).unwrap().payments(),
        &[
            Payment::Qr {
                amount_applied: money(2_000)
            },
            Payment::Cash {
                amount_applied: money(3_000),
                amount_tendered: money(4_000),
                change_given: money(1_000)
            }
        ]
    );
}

#[test]
fn explicit_zero_qr_creates_no_qr_row() {
    let payments = derive(5_000, Some(5_000), Some(0))
        .unwrap()
        .payments()
        .to_vec();
    assert_eq!(payments.len(), 1);
    assert!(matches!(payments[0], Payment::Cash { .. }));
}

#[test]
fn qr_above_total_is_rejected() {
    assert_eq!(
        derive(5_000, None, Some(5_001)),
        Err(PaymentError::QrExceedsTotal)
    );
}

#[test]
fn missing_or_insufficient_cash_for_remaining_total_is_rejected() {
    assert_eq!(
        derive(5_000, None, Some(2_000)),
        Err(PaymentError::CashTenderRequired)
    );
    assert_eq!(
        derive(5_000, Some(2_999), Some(2_000)),
        Err(PaymentError::InsufficientCashTender)
    );
}

#[test]
fn positive_cash_tender_after_full_qr_is_rejected() {
    assert_eq!(
        derive(5_000, Some(1), Some(5_000)),
        Err(PaymentError::UnexpectedCashTender)
    );
}

#[test]
fn zero_tender_after_full_qr_creates_no_cash_row() {
    assert_eq!(
        derive(5_000, Some(0), Some(5_000)).unwrap().payments(),
        &[Payment::Qr {
            amount_applied: money(5_000)
        }]
    );
}

#[test]
fn sale_rejects_empty_lines() {
    assert!(Sale::new(vec![], vec![]).is_err());
}

#[test]
fn persisted_cash_payment_requires_consistent_tender_and_change() {
    assert!(Payment::cash(money(5_000), money(6_000), money(999)).is_err());
    assert_eq!(
        Payment::cash(money(5_000), money(6_000), money(1_000)).unwrap(),
        Payment::Cash {
            amount_applied: money(5_000),
            amount_tendered: money(6_000),
            change_given: money(1_000),
        }
    );
}

#[test]
fn sale_defense_in_depth_rejects_applied_payments_that_do_not_equal_total() {
    assert!(Sale::new(vec![line(5_000, 1)], vec![Payment::qr(money(4_999))]).is_err());
}

#[test]
fn aggregate_total_overflow_is_rejected() {
    assert!(Sale::new(
        vec![line(i64::MAX, 1), line(1, 1)],
        vec![Payment::qr(money(i64::MAX))],
    )
    .is_err());
}
