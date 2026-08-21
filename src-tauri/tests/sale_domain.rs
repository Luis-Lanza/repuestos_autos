use repuestos_autos::domain::sales::{Payment, Sale, SaleLine};
use repuestos_autos::domain::{MoneyCentavos, Quantity, RequestId};

#[test]
fn constructs_explicit_values_and_a_line_total() {
    let quantity = Quantity::new(2).expect("positive whole quantity");
    let unit_price = MoneyCentavos::new(2_500).expect("non-negative centavos");
    let minimum_price = MoneyCentavos::new(2_000).expect("non-negative centavos");
    let line = SaleLine::new(1, quantity, unit_price, minimum_price).expect("valid line");

    assert_eq!(line.product_id(), 1);
    assert_eq!(line.quantity(), quantity);
    assert_eq!(line.total(), MoneyCentavos::new(5_000).unwrap());
    assert_eq!(line.minimum_unit_price_snapshot(), minimum_price);
    assert_eq!(
        RequestId::parse("550e8400-e29b-41d4-a716-446655440000")
            .unwrap()
            .as_uuid()
            .to_string(),
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(
        Payment::cash(
            MoneyCentavos::new(5_000).unwrap(),
            MoneyCentavos::new(6_000).unwrap(),
            MoneyCentavos::new(1_000).unwrap(),
        )
        .unwrap()
        .amount_applied(),
        MoneyCentavos::new(5_000).unwrap()
    );
}

#[test]
fn rejects_invalid_values_and_overflow_prone_line_totals() {
    assert!(Quantity::new(0).is_err());
    assert!(Quantity::new(-1).is_err());
    assert!(MoneyCentavos::new(-1).is_err());
    assert!(RequestId::parse("not-a-uuid").is_err());

    let overflow = SaleLine::new(
        1,
        Quantity::new(2).unwrap(),
        MoneyCentavos::new(i64::MAX).unwrap(),
        MoneyCentavos::new(0).unwrap(),
    );
    assert!(overflow.is_err());
}

#[test]
fn validates_price_floors_and_payment_combinations_against_sale_total() {
    let line = SaleLine::new(
        1,
        Quantity::new(2).unwrap(),
        MoneyCentavos::new(2_500).unwrap(),
        MoneyCentavos::new(2_000).unwrap(),
    )
    .unwrap();

    assert!(SaleLine::new(
        1,
        Quantity::new(1).unwrap(),
        MoneyCentavos::new(1_999).unwrap(),
        MoneyCentavos::new(2_000).unwrap(),
    )
    .is_err());
    assert!(Payment::cash(
        MoneyCentavos::new(2_000).unwrap(),
        MoneyCentavos::new(1_999).unwrap(),
        MoneyCentavos::new(0).unwrap(),
    )
    .is_err());

    let cash = Payment::cash(
        MoneyCentavos::new(2_000).unwrap(),
        MoneyCentavos::new(2_500).unwrap(),
        MoneyCentavos::new(500).unwrap(),
    )
    .unwrap();
    let qr = Payment::qr(MoneyCentavos::new(4_000).unwrap());
    let second_line = SaleLine::new(
        2,
        Quantity::new(1).unwrap(),
        MoneyCentavos::new(1_000).unwrap(),
        MoneyCentavos::new(1_000).unwrap(),
    )
    .unwrap();

    assert_eq!(
        Sale::new(vec![line], vec![Payment::qr(line.total())])
            .unwrap()
            .total(),
        line.total()
    );
    assert_eq!(
        Sale::new(
            vec![line],
            vec![
                Payment::cash(line.total(), line.total(), MoneyCentavos::new(0).unwrap()).unwrap()
            ]
        )
        .unwrap()
        .total(),
        line.total()
    );
    assert_eq!(
        Sale::new(vec![line, second_line], vec![cash, qr])
            .unwrap()
            .total(),
        MoneyCentavos::new(6_000).unwrap()
    );
    assert!(Sale::new(vec![], vec![]).is_err());
    assert!(Sale::new(
        vec![line],
        vec![Payment::qr(MoneyCentavos::new(4_999).unwrap())]
    )
    .is_err());
}
