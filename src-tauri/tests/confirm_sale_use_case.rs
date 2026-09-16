use repuestos_autos::application::sales::{
    confirm_sale, ApplicationConfirmSaleRequest, ApplicationRequestedLine, ConfirmSaleError,
    ConfirmSaleRequest, ConfirmSaleUseCase, RequestedLine,
};
use repuestos_autos::catalog::open_seeded_catalog;
use repuestos_autos::domain::sales::{Payment, PaymentInput};
use repuestos_autos::domain::{MoneyCentavos, Quantity, RequestId};
use repuestos_autos::infrastructure::sqlite::sale_repository::SqliteSaleRepository;
use repuestos_autos::infrastructure::sqlite::{open_database, production_database_config};

fn money(value: i64) -> MoneyCentavos {
    MoneyCentavos::new(value).unwrap()
}

fn request() -> ConfirmSaleRequest {
    ConfirmSaleRequest {
        request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        lines: vec![(1, 2, 2_500), (3, 1, 3_000)]
            .into_iter()
            .map(|(product_id, quantity, price)| RequestedLine {
                product_id,
                quantity: Quantity::new(quantity).unwrap(),
                negotiated_unit_price: MoneyCentavos::new(price).unwrap(),
            })
            .collect(),
        payments: vec![Payment::cash(
            MoneyCentavos::new(8_000).unwrap(),
            MoneyCentavos::new(9_000).unwrap(),
            MoneyCentavos::new(1_000).unwrap(),
        )
        .unwrap()],
    }
}

fn single_line_request(
    request_id: &str,
    product_id: i64,
    price: i64,
    payments: Vec<Payment>,
) -> ConfirmSaleRequest {
    ConfirmSaleRequest {
        request_id: RequestId::parse(request_id).unwrap(),
        lines: vec![RequestedLine {
            product_id,
            quantity: Quantity::new(1).unwrap(),
            negotiated_unit_price: MoneyCentavos::new(price).unwrap(),
        }],
        payments,
    }
}

fn authoritative_request(
    request_id: &str,
    lines: &[(i64, i64)],
    payment: PaymentInput,
) -> ApplicationConfirmSaleRequest {
    ApplicationConfirmSaleRequest {
        request_id: RequestId::parse(request_id).unwrap(),
        lines: lines
            .iter()
            .map(|(product_id, quantity)| ApplicationRequestedLine {
                product_id: *product_id,
                quantity: Quantity::new(*quantity).unwrap(),
                captured_unit_price: MoneyCentavos::new(if *product_id == 3 {
                    3_000
                } else {
                    2_500
                })
                .unwrap(),
                captured_revision: 0,
                final_unit_price: None,
                acknowledged_price: None,
                acknowledged_revision: None,
            })
            .collect(),
        payment,
    }
}

fn confirm_authoritative(
    connection: &mut rusqlite::Connection,
    request: ApplicationConfirmSaleRequest,
) -> Result<repuestos_autos::application::sales::PersistedSaleSummary, ConfirmSaleError> {
    ConfirmSaleUseCase::new(connection, &SqliteSaleRepository).confirm(request)
}

#[derive(Debug, PartialEq, Eq)]
struct PersistenceSnapshot {
    sales: i64,
    lines: i64,
    payments: i64,
    movements: i64,
    stock: Vec<(i64, i64)>,
}

fn snapshot(connection: &rusqlite::Connection) -> PersistenceSnapshot {
    let count = |table| {
        connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap()
    };
    let stock = connection
        .prepare("SELECT product_id, quantity FROM stock_balances ORDER BY product_id")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<Vec<(i64, i64)>, _>>()
        .unwrap();
    PersistenceSnapshot {
        sales: count("sales"),
        lines: count("sale_lines"),
        payments: count("sale_payments"),
        movements: count("inventory_movements"),
        stock,
    }
}

fn assert_no_sale_effects(connection: &rusqlite::Connection) {
    for table in [
        "sales",
        "sale_lines",
        "sale_payments",
        "inventory_movements",
    ] {
        assert_eq!(
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0,
            "{table} must be empty after rejection"
        );
    }
    assert_eq!(
        connection
            .query_row(
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        8
    );
}

fn captured_request(
    request_id: &str,
    price: i64,
    revision: i64,
    acknowledged: Option<(i64, i64)>,
) -> ApplicationConfirmSaleRequest {
    ApplicationConfirmSaleRequest {
        request_id: RequestId::parse(request_id).unwrap(),
        lines: vec![ApplicationRequestedLine {
            product_id: 1,
            quantity: Quantity::new(1).unwrap(),
            captured_unit_price: MoneyCentavos::new(price).unwrap(),
            captured_revision: revision,
            final_unit_price: None,
            acknowledged_price: acknowledged.map(|(price, _)| MoneyCentavos::new(price).unwrap()),
            acknowledged_revision: acknowledged.map(|(_, revision)| revision),
        }],
        payment: PaymentInput {
            amount_tendered: None,
            qr_applied: Some(
                MoneyCentavos::new(acknowledged.map_or(price, |(price, _)| price)).unwrap(),
            ),
        },
    }
}

#[test]
fn stale_prices_require_an_exact_acknowledgement_and_confirmed_lines_remain_historical() {
    let mut connection = open_seeded_catalog().unwrap();
    let before = snapshot(&connection);
    connection
        .execute(
            "UPDATE products SET list_price_centavos = 2700, minimum_unit_price_centavos = 2700, revision = 1 WHERE id = 1",
            [],
        )
        .unwrap();
    let stale = captured_request("550e8400-e29b-41d4-a716-446655440127", 2_500, 0, None);
    assert_eq!(
        confirm_authoritative(&mut connection, stale),
        Err(ConfirmSaleError::StaleCatalogPrice {
            product_id: 1,
            current_unit_price: MoneyCentavos::new(2_700).unwrap(),
            current_revision: 1
        })
    );
    assert_eq!(snapshot(&connection), before);
    connection
        .execute(
            "UPDATE products SET list_price_centavos = 2800, minimum_unit_price_centavos = 2800, revision = 2 WHERE id = 1",
            [],
        )
        .unwrap();
    let stale_again = captured_request(
        "550e8400-e29b-41d4-a716-446655440128",
        2_500,
        0,
        Some((2_700, 1)),
    );
    assert_eq!(
        confirm_authoritative(&mut connection, stale_again),
        Err(ConfirmSaleError::StaleCatalogPrice {
            product_id: 1,
            current_unit_price: MoneyCentavos::new(2_800).unwrap(),
            current_revision: 2
        })
    );
    assert_eq!(snapshot(&connection), before);
    let confirmed = confirm_authoritative(
        &mut connection,
        captured_request(
            "550e8400-e29b-41d4-a716-446655440129",
            2_500,
            0,
            Some((2_800, 2)),
        ),
    )
    .unwrap();
    connection
        .execute(
            "UPDATE products SET list_price_centavos = 3000, minimum_unit_price_centavos = 3000, revision = 3 WHERE id = 1",
            [],
        )
        .unwrap();
    assert_eq!(
        confirmed.lines[0].negotiated_unit_price,
        MoneyCentavos::new(2_800).unwrap()
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT negotiated_unit_price_centavos FROM sale_lines WHERE sale_id = ?1",
                [confirmed.sale_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        2_800
    );
}

#[test]
fn rejects_invalid_authoritative_payments_without_persisting_any_effects() {
    let cases = [
        (
            PaymentInput {
                amount_tendered: None,
                qr_applied: Some(MoneyCentavos::new(2_501).unwrap()),
            },
            ConfirmSaleError::QrExceedsTotal,
        ),
        (
            PaymentInput {
                amount_tendered: Some(MoneyCentavos::new(2_499).unwrap()),
                qr_applied: None,
            },
            ConfirmSaleError::InsufficientCashTender,
        ),
    ];

    for (index, (payment, expected)) in cases.into_iter().enumerate() {
        let mut connection = open_seeded_catalog().unwrap();
        let before = snapshot(&connection);
        assert_eq!(
            confirm_authoritative(
                &mut connection,
                authoritative_request(
                    &format!("550e8400-e29b-41d4-a716-44665544012{index}"),
                    &[(1, 1)],
                    payment,
                ),
            ),
            Err(expected),
        );
        assert_eq!(snapshot(&connection), before);
    }
}

#[test]
fn rolls_back_later_stock_failure_and_allows_the_same_request_to_retry() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute("INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (3, 1, 'FLT-002', 'Filtro de aire', 1, 3000, 3000)", []).unwrap();
    connection
        .execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (3, 0)",
            [],
        )
        .unwrap();
    let request = authoritative_request(
        "550e8400-e29b-41d4-a716-446655440121",
        &[(1, 1), (3, 1)],
        PaymentInput {
            amount_tendered: Some(MoneyCentavos::new(5_500).unwrap()),
            qr_applied: None,
        },
    );
    let before = snapshot(&connection);

    assert_eq!(
        confirm_authoritative(&mut connection, request),
        Err(ConfirmSaleError::InsufficientStock),
    );
    assert_eq!(snapshot(&connection), before);

    connection
        .execute(
            "UPDATE stock_balances SET quantity = 1 WHERE product_id = 3",
            [],
        )
        .unwrap();
    let retried = confirm_authoritative(
        &mut connection,
        authoritative_request(
            "550e8400-e29b-41d4-a716-446655440121",
            &[(1, 1), (3, 1)],
            PaymentInput {
                amount_tendered: Some(MoneyCentavos::new(5_500).unwrap()),
                qr_applied: None,
            },
        ),
    )
    .unwrap();

    assert_eq!(retried.total, MoneyCentavos::new(5_500).unwrap());
    assert_eq!(snapshot(&connection).sales, before.sales + 1);
    assert_eq!(snapshot(&connection).movements, before.movements + 2);
}

#[test]
fn rolls_back_all_effects_when_post_write_summary_readback_fails() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER delete_payments_after_confirmation
                 AFTER UPDATE OF status ON sales
                 WHEN NEW.status = 'confirmed'
                 BEGIN
                     DELETE FROM sale_payments WHERE sale_id = NEW.id;
                 END;",
        )
        .unwrap();
    let before = snapshot(&connection);

    assert_eq!(
        confirm_authoritative(
            &mut connection,
            authoritative_request(
                "550e8400-e29b-41d4-a716-446655440126",
                &[(1, 1)],
                PaymentInput {
                    amount_tendered: None,
                    qr_applied: Some(MoneyCentavos::new(2_500).unwrap()),
                },
            ),
        ),
        Err(ConfirmSaleError::PersistedDataInvalid),
    );
    assert_eq!(snapshot(&connection), before);
}

#[test]
fn returns_each_payment_mode_from_stored_sqlite_facts() {
    let cases = [
        (
            "550e8400-e29b-41d4-a716-446655440123",
            PaymentInput {
                amount_tendered: Some(MoneyCentavos::new(3_000).unwrap()),
                qr_applied: None,
            },
            vec![Payment::cash(
                MoneyCentavos::new(2_500).unwrap(),
                MoneyCentavos::new(3_000).unwrap(),
                MoneyCentavos::new(500).unwrap(),
            )
            .unwrap()],
            vec![("cash", 2_500, Some(3_000), Some(500))],
        ),
        (
            "550e8400-e29b-41d4-a716-446655440124",
            PaymentInput {
                amount_tendered: None,
                qr_applied: Some(MoneyCentavos::new(2_500).unwrap()),
            },
            vec![Payment::qr(MoneyCentavos::new(2_500).unwrap())],
            vec![("qr", 2_500, None, None)],
        ),
        (
            "550e8400-e29b-41d4-a716-446655440125",
            PaymentInput {
                amount_tendered: Some(MoneyCentavos::new(1_500).unwrap()),
                qr_applied: Some(MoneyCentavos::new(1_000).unwrap()),
            },
            vec![
                Payment::qr(MoneyCentavos::new(1_000).unwrap()),
                Payment::cash(
                    MoneyCentavos::new(1_500).unwrap(),
                    MoneyCentavos::new(1_500).unwrap(),
                    MoneyCentavos::new(0).unwrap(),
                )
                .unwrap(),
            ],
            vec![
                ("qr", 1_000, None, None),
                ("cash", 1_500, Some(1_500), Some(0)),
            ],
        ),
    ];

    for (request_id, payment, expected_payments, expected_rows) in cases {
        let mut connection = open_seeded_catalog().unwrap();
        let sale = confirm_authoritative(
            &mut connection,
            authoritative_request(request_id, &[(1, 1)], payment),
        )
        .unwrap();
        let rows = connection
            .prepare("SELECT method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos FROM sale_payments WHERE sale_id = ?1 ORDER BY id")
            .unwrap()
            .query_map([sale.sale_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<(String, i64, Option<i64>, Option<i64>)>, _>>()
            .unwrap();

        assert_eq!(sale.payments, expected_payments);
        assert_eq!(
            rows,
            expected_rows
                .into_iter()
                .map(|(method, applied, tendered, change)| (
                    method.to_owned(),
                    applied,
                    tendered,
                    change
                ))
                .collect::<Vec<_>>(),
        );
    }
}

#[test]
fn rejects_corrupt_confirmed_reservations_without_new_effects() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute(
            "INSERT INTO sales (request_id, status, total_centavos, confirmed_at) VALUES (?1, 'confirmed', 0, CURRENT_TIMESTAMP)",
            ["550e8400-e29b-41d4-a716-446655440122"],
        )
        .unwrap();
    let before = snapshot(&connection);

    assert_eq!(
        confirm_authoritative(
            &mut connection,
            authoritative_request(
                "550e8400-e29b-41d4-a716-446655440122",
                &[(1, 1)],
                PaymentInput {
                    amount_tendered: None,
                    qr_applied: Some(MoneyCentavos::new(2_500).unwrap()),
                },
            ),
        ),
        Err(ConfirmSaleError::RequestConflict),
    );
    assert_eq!(snapshot(&connection), before);
}

#[test]
fn resolves_catalog_prices_in_request_order_and_writes_compatibility_snapshots() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute("INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (3, 1, 'FLT-002', 'Filtro de aire', 1, 3000, 3000)", []).unwrap();
    connection
        .execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (3, 4)",
            [],
        )
        .unwrap();
    let sale = confirm_authoritative(
        &mut connection,
        authoritative_request(
            "550e8400-e29b-41d4-a716-446655440099",
            &[(3, 1), (1, 2)],
            PaymentInput {
                amount_tendered: Some(MoneyCentavos::new(9_000).unwrap()),
                qr_applied: Some(MoneyCentavos::new(1_000).unwrap()),
            },
        ),
    )
    .unwrap();

    assert_eq!(sale.lines[0].product_id, 3);
    assert_eq!(
        sale.lines[0].negotiated_unit_price,
        MoneyCentavos::new(3_000).unwrap()
    );
    assert_eq!(sale.lines[1].product_id, 1);
    assert_eq!(
        sale.lines[1].negotiated_unit_price,
        MoneyCentavos::new(2_500).unwrap()
    );
    assert_eq!(sale.total, MoneyCentavos::new(8_000).unwrap());
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM sale_lines WHERE negotiated_unit_price_centavos = minimum_unit_price_snapshot_centavos", [], |row| row.get::<_, i64>(0)).unwrap(), 2);
}

#[test]
fn reservation_conflicts_when_retry_identity_changes() {
    let mut connection = open_seeded_catalog().unwrap();
    let _first = confirm_authoritative(
        &mut connection,
        authoritative_request(
            "550e8400-e29b-41d4-a716-446655440098",
            &[(1, 1)],
            PaymentInput {
                amount_tendered: None,
                qr_applied: Some(MoneyCentavos::new(2_500).unwrap()),
            },
        ),
    )
    .unwrap();
    connection
        .execute(
            "UPDATE products SET sku = 'REN-999', name = 'Renamed filter', list_price_centavos = 9999, minimum_unit_price_centavos = 9999 WHERE id = 1",
            [],
        )
        .unwrap();

    let retry = confirm_authoritative(
        &mut connection,
        authoritative_request(
            "550e8400-e29b-41d4-a716-446655440098",
            &[(99, 2)],
            PaymentInput {
                amount_tendered: Some(MoneyCentavos::new(19_998).unwrap()),
                qr_applied: None,
            },
        ),
    );

    assert!(retry.is_err());
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM sales", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM sale_lines", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM sale_payments", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        7
    );
}

#[test]
fn confirms_a_multi_line_cash_sale_with_persisted_stock_movements_and_summary() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute("INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (3, 1, 'FLT-002', 'Filtro de aire', 1, 3000, 3000)", []).unwrap();
    connection
        .execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (3, 4)",
            [],
        )
        .unwrap();
    let sale = confirm_sale(&mut connection, request()).unwrap();

    assert_eq!(sale.total, MoneyCentavos::new(8_000).unwrap());
    assert_eq!(sale.lines.len(), 2);
    assert_eq!(
        sale.lines[0].minimum_unit_price_snapshot,
        MoneyCentavos::new(2_500).unwrap()
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        6
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        2
    );
}

#[test]
fn rolls_back_every_effect_when_a_later_line_has_insufficient_stock() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute("INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (3, 1, 'FLT-002', 'Filtro de aire', 1, 3000, 3000)", []).unwrap();
    connection
        .execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (3, 0)",
            [],
        )
        .unwrap();

    assert_eq!(
        confirm_sale(&mut connection, request()),
        Err("insufficient stock".into())
    );
    for table in [
        "sales",
        "sale_lines",
        "sale_payments",
        "inventory_movements",
    ] {
        assert_eq!(
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    assert_eq!(
        connection
            .query_row(
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        8
    );
}

#[test]
fn returns_the_original_summary_without_reapplying_a_changed_retry() {
    let mut connection = open_seeded_catalog().unwrap();
    let first = confirm_sale(
        &mut connection,
        ConfirmSaleRequest {
            request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            lines: vec![RequestedLine {
                product_id: 1,
                quantity: Quantity::new(1).unwrap(),
                negotiated_unit_price: MoneyCentavos::new(2_500).unwrap(),
            }],
            payments: vec![Payment::qr(MoneyCentavos::new(2_500).unwrap())],
        },
    )
    .unwrap();
    let retry = confirm_sale(
        &mut connection,
        ConfirmSaleRequest {
            request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            lines: vec![RequestedLine {
                product_id: 1,
                quantity: Quantity::new(2).unwrap(),
                negotiated_unit_price: MoneyCentavos::new(9_999).unwrap(),
            }],
            payments: vec![Payment::qr(MoneyCentavos::new(19_998).unwrap())],
        },
    )
    .unwrap();

    assert_eq!(retry, first);
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM sales", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM sale_lines", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        7
    );
}

#[test]
fn confirms_qr_only_and_mixed_payment_sales() {
    let mut connection = open_seeded_catalog().unwrap();
    let qr_sale = confirm_sale(
        &mut connection,
        single_line_request(
            "550e8400-e29b-41d4-a716-446655440010",
            1,
            2_500,
            vec![Payment::qr(MoneyCentavos::new(2_500).unwrap())],
        ),
    )
    .unwrap();
    let mixed_sale = confirm_sale(
        &mut connection,
        single_line_request(
            "550e8400-e29b-41d4-a716-446655440011",
            1,
            2_500,
            vec![
                Payment::cash(
                    MoneyCentavos::new(1_000).unwrap(),
                    MoneyCentavos::new(1_000).unwrap(),
                    MoneyCentavos::new(0).unwrap(),
                )
                .unwrap(),
                Payment::qr(MoneyCentavos::new(1_500).unwrap()),
            ],
        ),
    )
    .unwrap();

    assert_eq!(qr_sale.payments.len(), 1);
    assert_eq!(mixed_sale.payments.len(), 2);
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM sale_payments", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        3
    );
}

#[test]
fn rejects_inactive_missing_stale_and_unequal_payment_requests_without_effects() {
    let cases = [
        (
            2,
            1_800,
            vec![Payment::qr(MoneyCentavos::new(1_800).unwrap())],
            "product is inactive",
        ),
        (
            99,
            2_500,
            vec![Payment::qr(MoneyCentavos::new(2_500).unwrap())],
            "product is missing",
        ),
        (
            1,
            2_499,
            vec![Payment::qr(MoneyCentavos::new(2_499).unwrap())],
            "negotiated price is below the current minimum",
        ),
        (
            1,
            2_500,
            vec![Payment::qr(MoneyCentavos::new(2_499).unwrap())],
            "applied payments must equal the sale total",
        ),
    ];

    for (index, (product_id, price, payments, expected)) in cases.into_iter().enumerate() {
        let mut connection = open_seeded_catalog().unwrap();
        assert_eq!(
            confirm_sale(
                &mut connection,
                single_line_request(
                    &format!("550e8400-e29b-41d4-a716-44665544002{index}"),
                    product_id,
                    price,
                    payments,
                ),
            ),
            Err(expected.into())
        );
        assert_no_sale_effects(&connection);
    }
}

#[test]
fn reports_persistence_integrity_for_an_incomplete_reserved_sale() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute(
            "INSERT INTO sales (request_id, status, total_centavos) VALUES (?1, 'pending', 0)",
            ["550e8400-e29b-41d4-a716-446655440000"],
        )
        .unwrap();

    assert_eq!(
        confirm_sale(&mut connection, request()),
        Err("persistence integrity failure".into())
    );
}

#[test]
fn persists_confirmed_sales_when_reopening_the_production_database() {
    let directory = std::env::temp_dir().join(format!(
        "repuestos-autos-persistence-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let config = production_database_config(&directory);
    assert_eq!(config.path(), directory.join("repuestos-autos.sqlite3"));

    let first = {
        let mut connection = open_database(&config).unwrap();
        confirm_sale(
            &mut connection,
            single_line_request(
                "550e8400-e29b-41d4-a716-446655440031",
                1,
                2_500,
                vec![Payment::qr(MoneyCentavos::new(2_500).unwrap())],
            ),
        )
        .unwrap()
    };

    let mut reopened = open_database(&config).unwrap();
    let retry = confirm_sale(
        &mut reopened,
        single_line_request(
            "550e8400-e29b-41d4-a716-446655440031",
            1,
            9_999,
            vec![Payment::qr(MoneyCentavos::new(9_999).unwrap())],
        ),
    )
    .unwrap();

    assert_eq!(retry, first);
    assert_eq!(
        reopened
            .query_row("SELECT COUNT(*) FROM sales", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        reopened
            .query_row(
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        7
    );
    drop(reopened);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn sqlite_enforces_foreign_keys_request_id_row_checks_and_immutable_movements() {
    let mut connection = open_seeded_catalog().unwrap();
    assert!(connection
        .execute("INSERT INTO sales (request_id, status, total_centavos) VALUES ('duplicate', 'confirmed', -1)", [])
        .is_err());
    connection
        .execute("INSERT INTO sales (request_id, status, total_centavos) VALUES ('duplicate', 'confirmed', 0)", [])
        .unwrap();
    assert!(connection
        .execute("INSERT INTO sales (request_id, status, total_centavos) VALUES ('duplicate', 'confirmed', 0)", [])
        .is_err());
    assert!(connection
        .execute("INSERT INTO sale_lines (sale_id, product_id, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos) VALUES (999, 1, 1, 2500, 2500, 2500)", [])
        .is_err());

    confirm_sale(
        &mut connection,
        single_line_request(
            "550e8400-e29b-41d4-a716-446655440030",
            1,
            2_500,
            vec![Payment::qr(MoneyCentavos::new(2_500).unwrap())],
        ),
    )
    .unwrap();
    assert!(connection
        .execute("UPDATE inventory_movements SET quantity_delta = -2", [])
        .is_err());
    assert!(connection
        .execute("DELETE FROM inventory_movements", [])
        .is_err());
}

#[test]
fn authoritative_identity_replays_exactly_and_conflicts_without_effects() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute("INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (3, 1, 'FLT-002', 'Filtro de aire', 1, 3000, 3000)", []).unwrap();
    connection
        .execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (3, 4)",
            [],
        )
        .unwrap();
    let request_id = "550e8400-e29b-41d4-a716-446655440140";
    let first = confirm_authoritative(
        &mut connection,
        authoritative_request(
            request_id,
            &[(3, 1), (1, 1)],
            PaymentInput {
                amount_tendered: None,
                qr_applied: Some(MoneyCentavos::new(5_500).unwrap()),
            },
        ),
    )
    .unwrap();
    let before_retry = snapshot(&connection);
    let persisted_identity = connection.query_row(
        "SELECT operation_kind, payload_version, length(canonical_payload), payload_sha256 FROM sales WHERE id = ?1",
        [first.sale_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?, row.get::<_, String>(3)?)),
    ).unwrap();
    assert_eq!(persisted_identity.0, "confirm_sale");
    assert_eq!(persisted_identity.1, 1);
    assert!(persisted_identity.2 > 0);
    assert_eq!(persisted_identity.3.len(), 64);

    let replay = confirm_authoritative(
        &mut connection,
        authoritative_request(
            request_id,
            &[(3, 1), (1, 1)],
            PaymentInput {
                amount_tendered: None,
                qr_applied: Some(MoneyCentavos::new(5_500).unwrap()),
            },
        ),
    )
    .unwrap();
    assert_eq!(replay, first);
    assert_eq!(snapshot(&connection), before_retry);

    for changed in [
        authoritative_request(
            request_id,
            &[(3, 2), (1, 1)],
            PaymentInput {
                amount_tendered: None,
                qr_applied: Some(MoneyCentavos::new(8_500).unwrap()),
            },
        ),
        authoritative_request(
            request_id,
            &[(3, 1), (1, 1)],
            PaymentInput {
                amount_tendered: Some(MoneyCentavos::new(5_500).unwrap()),
                qr_applied: None,
            },
        ),
        authoritative_request(
            request_id,
            &[(1, 1), (3, 1)],
            PaymentInput {
                amount_tendered: None,
                qr_applied: Some(MoneyCentavos::new(5_500).unwrap()),
            },
        ),
    ] {
        assert_eq!(
            confirm_authoritative(&mut connection, changed),
            Err(ConfirmSaleError::RequestConflict)
        );
        assert_eq!(snapshot(&connection), before_retry);
    }
}

#[test]
fn mixed_final_price_request_replays_exactly_and_conflicts_on_changed_identity() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute("INSERT INTO products (id, category_id, sku, name, active, list_price_centavos, minimum_unit_price_centavos) VALUES (3, 1, 'FLT-002', 'Filtro de aire', 1, 3000, 3000)", []).unwrap();
    connection
        .execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (3, 4)",
            [],
        )
        .unwrap();
    let request = |final_unit_price, legacy_captured_unit_price| ApplicationConfirmSaleRequest {
        request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440153").unwrap(),
        lines: vec![
            ApplicationRequestedLine {
                product_id: 1,
                quantity: Quantity::new(1).unwrap(),
                captured_unit_price: money(2_500),
                captured_revision: 0,
                final_unit_price: Some(money(final_unit_price)),
                acknowledged_price: None,
                acknowledged_revision: None,
            },
            ApplicationRequestedLine {
                product_id: 3,
                quantity: Quantity::new(1).unwrap(),
                captured_unit_price: money(legacy_captured_unit_price),
                captured_revision: 0,
                final_unit_price: None,
                acknowledged_price: None,
                acknowledged_revision: None,
            },
        ],
        payment: PaymentInput {
            amount_tendered: None,
            qr_applied: Some(money(final_unit_price + 3_000)),
        },
    };

    let first = confirm_authoritative(&mut connection, request(2_750, 3_000)).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT payload_version FROM sales WHERE id = ?1",
                [first.sale_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        2
    );
    let before_retry = snapshot(&connection);

    let replay = confirm_authoritative(&mut connection, request(2_750, 3_000)).unwrap();
    assert_eq!(replay, first);
    assert_eq!(snapshot(&connection), before_retry);

    assert_eq!(
        confirm_authoritative(&mut connection, request(2_800, 3_000)),
        Err(ConfirmSaleError::RequestConflict)
    );
    assert_eq!(snapshot(&connection), before_retry);

    assert_eq!(
        confirm_authoritative(&mut connection, request(2_750, 2_999)),
        Err(ConfirmSaleError::RequestConflict)
    );
    assert_eq!(snapshot(&connection), before_retry);
}

#[test]
fn acknowledged_price_and_revision_nullability_is_part_of_identity() {
    let mut connection = open_seeded_catalog().unwrap();
    connection
        .execute(
            "UPDATE products SET list_price_centavos = 2700, minimum_unit_price_centavos = 2700, revision = 1 WHERE id = 1",
            [],
        )
        .unwrap();
    let request_id = "550e8400-e29b-41d4-a716-446655440141";
    confirm_authoritative(
        &mut connection,
        captured_request(request_id, 2_500, 0, Some((2_700, 1))),
    )
    .unwrap();

    assert_eq!(
        confirm_authoritative(
            &mut connection,
            captured_request(request_id, 2_500, 0, None),
        ),
        Err(ConfirmSaleError::RequestConflict),
    );
}

#[test]
fn legacy_sale_reuse_returns_conflict_without_new_sale_effects() {
    let mut connection = open_seeded_catalog().unwrap();
    let request_id = "550e8400-e29b-41d4-a716-446655440143";
    let first = confirm_authoritative(
        &mut connection,
        captured_request(request_id, 2_500, 0, None),
    )
    .unwrap();
    connection
        .execute(
            "UPDATE sales SET operation_kind = NULL, payload_version = NULL, canonical_payload = NULL, payload_sha256 = NULL WHERE id = ?1",
            [first.sale_id],
        )
        .unwrap();
    let before_retry = snapshot(&connection);

    assert_eq!(
        confirm_authoritative(
            &mut connection,
            captured_request(request_id, 2_500, 0, None),
        ),
        Err(ConfirmSaleError::RequestConflict),
    );
    assert_eq!(snapshot(&connection), before_retry);
}

#[test]
fn malformed_persisted_identity_fails_closed_as_invalid_data() {
    let mut connection = open_seeded_catalog().unwrap();
    let request_id = "550e8400-e29b-41d4-a716-446655440142";
    let first = confirm_authoritative(
        &mut connection,
        captured_request(request_id, 2_500, 0, None),
    )
    .unwrap();
    connection
        .execute(
            "UPDATE sales SET payload_sha256 = 'broken' WHERE id = ?1",
            [first.sale_id],
        )
        .unwrap();

    assert_eq!(
        confirm_authoritative(
            &mut connection,
            captured_request(request_id, 2_500, 0, None),
        ),
        Err(ConfirmSaleError::PersistedDataInvalid),
    );
}


#[test]
fn explicit_final_price_uses_final_for_totals_and_persists_independent_snapshots() {
    let mut connection = repuestos_autos::infrastructure::sqlite::open_seeded_catalog().unwrap();
    let request = ApplicationConfirmSaleRequest {
        request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440150").unwrap(),
        lines: vec![ApplicationRequestedLine {
            product_id: 1,
            quantity: Quantity::new(2).unwrap(),
            captured_unit_price: money(2_500),
            captured_revision: 0,
            final_unit_price: Some(money(2_750)),
            acknowledged_price: None,
            acknowledged_revision: None,
        }],
        payment: PaymentInput {
            amount_tendered: None,
            qr_applied: Some(money(5_500)),
        },
    };

    let sale = ConfirmSaleUseCase::new(&mut connection, &SqliteSaleRepository)
        .confirm(request)
        .unwrap();

    assert_eq!(sale.total, money(5_500));
    assert_eq!(sale.lines[0].negotiated_unit_price, money(2_750));
    assert_eq!(sale.lines[0].minimum_unit_price_snapshot, money(2_500));
    assert_eq!(sale.lines[0].list_price_snapshot, Some(money(2_500)));
    assert_eq!(
        connection
            .query_row(
                "SELECT negotiated_unit_price_centavos, list_price_snapshot_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos FROM sale_lines WHERE sale_id = ?1",
                [sale.sale_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap(),
        (2_750, Some(2_500), 2_500, 5_500)
    );
    assert!(connection
        .execute(
            "UPDATE sale_lines SET list_price_snapshot_centavos = 3_100 WHERE sale_id = ?1",
            [sale.sale_id],
        )
        .is_err());
}

#[test]
fn explicit_final_price_below_reloaded_minimum_is_bounded_and_atomic() {
    let mut connection = repuestos_autos::infrastructure::sqlite::open_seeded_catalog().unwrap();
    connection
        .execute(
            "UPDATE products SET list_price_centavos = 2_700, minimum_unit_price_centavos = 2_700 WHERE id = 1",
            [],
        )
        .unwrap();
    let request = ApplicationConfirmSaleRequest {
        request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440151").unwrap(),
        lines: vec![ApplicationRequestedLine {
            product_id: 1,
            quantity: Quantity::new(1).unwrap(),
            captured_unit_price: money(2_500),
            captured_revision: 0,
            final_unit_price: Some(money(2_600)),
            acknowledged_price: None,
            acknowledged_revision: None,
        }],
        payment: PaymentInput {
            amount_tendered: None,
            qr_applied: Some(money(2_600)),
        },
    };

    assert_eq!(
        ConfirmSaleUseCase::new(&mut connection, &SqliteSaleRepository).confirm(request),
        Err(ConfirmSaleError::FinalPriceBelowMinimum {
            product_id: 1,
            current_minimum_unit_price: money(2_700),
        })
    );
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM sales", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
    assert_eq!(connection.query_row("SELECT quantity FROM stock_balances WHERE product_id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 8);
}


#[test]
fn minimum_above_list_price_is_rejected_as_invalid_persisted_data() {
    let mut connection = repuestos_autos::infrastructure::sqlite::open_seeded_catalog().unwrap();
    connection
        .execute_batch("DROP TRIGGER products_validate_price_update;")
        .unwrap();
    connection
        .execute(
            "UPDATE products SET minimum_unit_price_centavos = 2_600 WHERE id = 1",
            [],
        )
        .unwrap();
    let request = ApplicationConfirmSaleRequest {
        request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440154").unwrap(),
        lines: vec![ApplicationRequestedLine {
            product_id: 1,
            quantity: Quantity::new(1).unwrap(),
            captured_unit_price: money(2_600),
            captured_revision: 0,
            final_unit_price: Some(money(2_600)),
            acknowledged_price: None,
            acknowledged_revision: None,
        }],
        payment: PaymentInput {
            amount_tendered: None,
            qr_applied: Some(money(2_600)),
        },
    };

    assert_eq!(
        ConfirmSaleUseCase::new(&mut connection, &SqliteSaleRepository).confirm(request),
        Err(ConfirmSaleError::PersistedDataInvalid)
    );
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM sales", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
}

#[test]
fn changed_explicit_final_price_conflicts_on_request_id_reuse() {
    let mut connection = repuestos_autos::infrastructure::sqlite::open_seeded_catalog().unwrap();
    let request = |final_price| ApplicationConfirmSaleRequest {
        request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440152").unwrap(),
        lines: vec![ApplicationRequestedLine {
            product_id: 1,
            quantity: Quantity::new(1).unwrap(),
            captured_unit_price: money(2_500),
            captured_revision: 0,
            final_unit_price: Some(money(final_price)),
            acknowledged_price: None,
            acknowledged_revision: None,
        }],
        payment: PaymentInput {
            amount_tendered: None,
            qr_applied: Some(money(final_price)),
        },
    };
    ConfirmSaleUseCase::new(&mut connection, &SqliteSaleRepository)
        .confirm(request(2_750))
        .unwrap();
    assert_eq!(
        ConfirmSaleUseCase::new(&mut connection, &SqliteSaleRepository).confirm(request(2_800)),
        Err(ConfirmSaleError::RequestConflict)
    );
}
