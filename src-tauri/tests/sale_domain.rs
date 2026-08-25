use repuestos_autos::domain::sales::{
    Payment, PaymentBreakdown, PaymentError, PaymentInput, Sale, SaleLine,
};
use repuestos_autos::domain::{MoneyCentavos, Quantity};

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
fn priced_line_exposes_the_authoritative_unit_price_and_checked_total() {
    let priced = line(2_500, 2);
    assert_eq!(priced.product_id(), 1);
    assert_eq!(priced.quantity(), Quantity::new(2).unwrap());
    assert_eq!(priced.unit_price(), money(2_500));
    assert_eq!(priced.total(), money(5_000));
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
