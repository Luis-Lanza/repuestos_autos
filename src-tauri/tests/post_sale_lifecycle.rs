use repuestos_autos::application::sales::{
    confirm_sale, CancelSaleRequest, ConfirmSaleRequest, CreateReturnRequest, PostSaleError,
    PostSaleLifecycleUseCase, PostSaleTransaction, PostSaleTransactionFactory, PostSaleUseCase,
    RequestedLine,
};
use repuestos_autos::catalog::open_seeded_catalog;
use repuestos_autos::domain::sales::{Payment, PostSaleDomainError, RequestedReturnLine};
use repuestos_autos::domain::{MoneyCentavos, Quantity, RequestId};
use repuestos_autos::infrastructure::sqlite::{
    open_database, production_database_config, SqlitePostSaleRepository,
    SqlitePostSaleTransactionFactory,
};
use rusqlite::{types::Value, Connection};
use std::{
    path::PathBuf,
    sync::{Arc, Barrier},
    thread,
    time::Duration,
};

fn request_id(value: &str) -> RequestId {
    RequestId::parse(value).unwrap()
}

fn scalar<P: rusqlite::Params>(connection: &Connection, sql: &str, params: P) -> i64 {
    connection.query_row(sql, params, |row| row.get(0)).unwrap()
}

fn confirm_two_line_sale(connection: &mut Connection) -> i64 {
    connection.execute("INSERT INTO products (id, category_id, sku, name, active, minimum_unit_price_centavos) VALUES (3, 1, 'FLT-002', 'Filter two', 1, 3000)", []).unwrap();
    connection
        .execute(
            "INSERT INTO stock_balances (product_id, quantity) VALUES (3, 4)",
            [],
        )
        .unwrap();
    confirm_sale(
        connection,
        ConfirmSaleRequest {
            request_id: request_id("550e8400-e29b-41d4-a716-446655440031"),
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
    .unwrap()
    .sale_id
}

fn return_request(id: RequestId, sale_id: i64, first: i64, second: i64) -> CreateReturnRequest {
    CreateReturnRequest::new(
        id,
        sale_id,
        vec![
            RequestedReturnLine {
                sale_line_id: second,
                quantity: 1,
            },
            RequestedReturnLine {
                sale_line_id: first,
                quantity: 1,
            },
        ],
    )
    .unwrap()
}

type OriginalFacts = (
    (i64, String, String, i64, Option<String>),
    Vec<(
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        Option<String>,
        Option<String>,
    )>,
    Vec<(i64, i64, String, i64, Option<i64>, Option<i64>)>,
);

fn original_facts(connection: &Connection, sale_id: i64) -> OriginalFacts {
    let sale = connection
        .query_row(
            "SELECT id, request_id, status, total_centavos, confirmed_at FROM sales WHERE id = ?1",
            [sale_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .unwrap();
    let lines = connection
        .prepare("SELECT id, sale_id, product_id, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos, sku_snapshot, product_name_snapshot FROM sale_lines WHERE sale_id = ?1 ORDER BY id")
        .unwrap()
        .query_map([sale_id], |row| {
            Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?,
                row.get(6)?, row.get(7)?, row.get(8)?,
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let payments = connection
        .prepare("SELECT id, sale_id, method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos FROM sale_payments WHERE sale_id = ?1 ORDER BY id")
        .unwrap()
        .query_map([sale_id], |row| {
            Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?,
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    (sale, lines, payments)
}

fn return_effects(connection: &Connection) -> (i64, i64, i64, i64, i64) {
    (
        scalar(connection, "SELECT COUNT(*) FROM post_sale_requests", []),
        scalar(connection, "SELECT COUNT(*) FROM sale_returns", []),
        scalar(connection, "SELECT COUNT(*) FROM sale_return_lines", []),
        scalar(connection, "SELECT COUNT(*) FROM inventory_movements WHERE movement_type = 'return'", []),
        scalar(connection, "SELECT COUNT(*) FROM sale_return_lines r JOIN inventory_movements m ON m.id = r.movement_id WHERE m.sale_id = r.sale_id AND m.sale_line_id = r.sale_line_id AND m.product_id = r.product_id AND m.quantity_delta = r.quantity AND m.movement_type = 'return'", []),
    )
}

fn stocks(connection: &Connection) -> (i64, i64) {
    (
        scalar(
            connection,
            "SELECT quantity FROM stock_balances WHERE product_id = ?1",
            [1],
        ),
        scalar(
            connection,
            "SELECT quantity FROM stock_balances WHERE product_id = ?1",
            [3],
        ),
    )
}

fn temporary_directory(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "repuestos-autos-post-sale-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn sqlite_post_sale_lifecycle_commits_and_rolls_back_in_a_migrated_database() {
    let mut connection = open_seeded_catalog().unwrap();
    let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
    {
        let mut transaction = factory.begin_immediate().unwrap();
        assert_eq!(
            transaction
                .repository_transaction()
                .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            10
        );
        transaction
            .repository_transaction()
            .execute("CREATE TABLE lifecycle_effects (id INTEGER)", [])
            .unwrap();
        transaction.commit().unwrap();
    }
    {
        let mut transaction = factory.begin_immediate().unwrap();
        transaction
            .repository_transaction()
            .execute("INSERT INTO lifecycle_effects DEFAULT VALUES", [])
            .unwrap();
        transaction.rollback().unwrap();
    }
    assert_eq!(
        factory
            .begin_immediate()
            .unwrap()
            .repository_transaction()
            .query_row("SELECT COUNT(*) FROM lifecycle_effects", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn return_persists_by_original_line_and_replays_canonical_requests() {
    let mut connection = open_seeded_catalog().unwrap();
    let sale_id = confirm_two_line_sale(&mut connection);
    let lines = connection
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY product_id")
        .unwrap()
        .query_map([sale_id], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let original = original_facts(&connection, sale_id);
    let id = request_id("550e8400-e29b-41d4-a716-446655440032");
    let repository = SqlitePostSaleRepository;
    let first = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .create_return(return_request(id.clone(), sale_id, lines[0], lines[1]))
            .unwrap()
    };

    assert_eq!(
        first
            .lines
            .iter()
            .map(|line| line.sale_line_id)
            .collect::<Vec<_>>(),
        lines
    );
    assert_eq!(return_effects(&connection), (1, 1, 2, 2, 2));
    assert_eq!(stocks(&connection), (7, 4));
    assert_eq!(original_facts(&connection, sale_id), original);

    let replay = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .create_return(return_request(id, sale_id, lines[1], lines[0]))
            .unwrap()
    };
    assert_eq!(replay, first);
    assert_eq!(return_effects(&connection), (1, 1, 2, 2, 2));
    assert_eq!(stocks(&connection), (7, 4));
    assert_eq!(original_facts(&connection, sale_id), original);
}

#[test]
fn cancellation_restores_residuals_and_replays_without_extra_effects() {
    let mut connection = open_seeded_catalog().unwrap();
    let sale_id = confirm_two_line_sale(&mut connection);
    let lines = connection
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY product_id")
        .unwrap()
        .query_map([sale_id], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let original = original_facts(&connection, sale_id);
    let repository = SqlitePostSaleRepository;
    {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .create_return(return_request(
                request_id("550e8400-e29b-41d4-a716-446655440081"),
                sale_id,
                lines[0],
                lines[1],
            ))
            .unwrap();
    }
    let id = request_id("550e8400-e29b-41d4-a716-446655440082");
    let first = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .cancel_sale(
                CancelSaleRequest::new(id.clone(), sale_id, " customer return ".into()).unwrap(),
            )
            .unwrap()
    };

    assert_eq!(
        first.status,
        repuestos_autos::application::sales::SaleLifecycleStatus::Cancelled
    );
    assert_eq!(first.reason, "customer return");
    assert_eq!(
        first
            .lines
            .iter()
            .map(|line| (line.sale_line_id, line.restored_quantity))
            .collect::<Vec<_>>(),
        vec![(lines[0], 1), (lines[1], 0)]
    );
    assert_eq!(stocks(&connection), (8, 4));
    assert_eq!(original_facts(&connection, sale_id), original);
    assert_eq!(
        scalar(&connection, "SELECT COUNT(*) FROM sale_cancellations", []),
        1
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM sale_cancellation_lines",
            []
        ),
        2
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM inventory_movements WHERE movement_type = 'cancellation'",
            []
        ),
        1
    );
    assert_eq!(scalar(&connection, "SELECT COUNT(*) FROM sale_cancellation_lines WHERE restored_quantity = 0 AND movement_id IS NULL", []), 1);
    assert_eq!(scalar(&connection, "SELECT COUNT(*) FROM sale_cancellation_lines c JOIN inventory_movements m ON m.id = c.movement_id WHERE m.movement_type = 'cancellation' AND m.sale_line_id = c.sale_line_id AND m.quantity_delta = c.restored_quantity", []), 1);
    assert_eq!(
        connection
            .query_row(
                "SELECT reason FROM inventory_movements WHERE movement_type = 'cancellation'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "customer return"
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM inventory_movements WHERE movement_type = 'return' AND reason IS NOT NULL",
            [],
        ),
        0
    );

    let replay = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .cancel_sale(CancelSaleRequest::new(id, sale_id, "customer return".into()).unwrap())
            .unwrap()
    };
    assert_eq!(replay, first);
    assert_eq!(stocks(&connection), (8, 4));
    assert_eq!(original_facts(&connection, sale_id), original);
    assert_eq!(
        scalar(&connection, "SELECT COUNT(*) FROM post_sale_requests", []),
        2
    );
    assert_eq!(
        scalar(&connection, "SELECT COUNT(*) FROM sale_cancellations", []),
        1
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM sale_cancellation_lines",
            []
        ),
        2
    );
    assert_eq!(
        scalar(
            &connection,
            "SELECT COUNT(*) FROM inventory_movements WHERE movement_type = 'cancellation'",
            []
        ),
        1
    );
}

fn cancellation_request(id: RequestId, sale_id: i64, reason: &str) -> CancelSaleRequest {
    CancelSaleRequest::new(id, sale_id, reason.into()).unwrap()
}

fn cancellation_effects(connection: &Connection) -> (i64, i64, i64, i64) {
    (
        scalar(connection, "SELECT COUNT(*) FROM post_sale_requests", []),
        scalar(connection, "SELECT COUNT(*) FROM sale_cancellations", []),
        scalar(
            connection,
            "SELECT COUNT(*) FROM sale_cancellation_lines",
            [],
        ),
        scalar(
            connection,
            "SELECT COUNT(*) FROM inventory_movements WHERE movement_type = 'cancellation'",
            [],
        ),
    )
}

fn correction_rows(connection: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut statement = connection.prepare(sql).unwrap();
    let columns = statement.column_count();
    statement
        .query_map([], |row| {
            (0..columns).map(|column| row.get(column)).collect()
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn correction_snapshot(connection: &Connection) -> [Vec<Vec<Value>>; 7] {
    [
        correction_rows(connection, "SELECT id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256, created_at FROM post_sale_requests ORDER BY id"),
        correction_rows(connection, "SELECT id, sale_id, operation_kind, occurred_at FROM sale_returns ORDER BY id"),
        correction_rows(connection, "SELECT return_id, sale_id, sale_line_id, product_id, quantity, movement_id FROM sale_return_lines ORDER BY return_id, sale_line_id"),
        correction_rows(connection, "SELECT id, sale_id, operation_kind, reason, occurred_at FROM sale_cancellations ORDER BY id"),
        correction_rows(connection, "SELECT cancellation_id, sale_id, sale_line_id, product_id, restored_quantity, movement_id FROM sale_cancellation_lines ORDER BY cancellation_id, sale_line_id"),
        correction_rows(connection, "SELECT id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, occurred_at, reason, operator_id, source_reference, request_id, counted_quantity, resulting_quantity FROM inventory_movements ORDER BY id"),
        correction_rows(connection, "SELECT product_id, quantity FROM stock_balances ORDER BY product_id"),
    ]
}

#[test]
fn fully_returned_sale_cancellation_records_zero_lines_and_replays() {
    let mut connection = open_seeded_catalog().unwrap();
    let sale_id = confirm_two_line_sale(&mut connection);
    let lines = connection
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY product_id")
        .unwrap()
        .query_map([sale_id], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let original = original_facts(&connection, sale_id);
    let repository = SqlitePostSaleRepository;
    {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .create_return(
                CreateReturnRequest::new(
                    request_id("550e8400-e29b-41d4-a716-446655440091"),
                    sale_id,
                    vec![
                        RequestedReturnLine {
                            sale_line_id: lines[0],
                            quantity: 2,
                        },
                        RequestedReturnLine {
                            sale_line_id: lines[1],
                            quantity: 1,
                        },
                    ],
                )
                .unwrap(),
            )
            .unwrap();
    }
    let stock_before = stocks(&connection);
    let id = request_id("550e8400-e29b-41d4-a716-446655440092");
    let first = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .cancel_sale(cancellation_request(id.clone(), sale_id, "fully returned"))
            .unwrap()
    };
    assert_eq!(
        first.status,
        repuestos_autos::application::sales::SaleLifecycleStatus::Cancelled
    );
    assert_eq!(
        first
            .lines
            .iter()
            .map(|line| line.restored_quantity)
            .collect::<Vec<_>>(),
        vec![0, 0]
    );
    assert_eq!(cancellation_effects(&connection), (2, 1, 2, 0));
    assert_eq!(scalar(&connection, "SELECT COUNT(*) FROM sale_cancellation_lines WHERE restored_quantity = 0 AND movement_id IS NULL", []), 2);
    assert_eq!(stocks(&connection), stock_before);
    assert_eq!(original_facts(&connection, sale_id), original);
    let replay = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .cancel_sale(cancellation_request(id, sale_id, "fully returned"))
            .unwrap()
    };
    assert_eq!(replay, first);
    assert_eq!(cancellation_effects(&connection), (2, 1, 2, 0));
    assert_eq!(stocks(&connection), stock_before);
    assert_eq!(original_facts(&connection, sale_id), original);
}

#[test]
fn cancellation_rejections_and_conflicts_leave_no_effects() {
    let mut connection = open_seeded_catalog().unwrap();
    let sale_id = confirm_two_line_sale(&mut connection);
    let lines = connection
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY product_id")
        .unwrap()
        .query_map([sale_id], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let original = original_facts(&connection, sale_id);
    let before = (cancellation_effects(&connection), stocks(&connection));
    let repository = SqlitePostSaleRepository;
    assert_eq!(
        CancelSaleRequest::new(
            request_id("550e8400-e29b-41d4-a716-446655440083"),
            sale_id,
            "   ".into(),
        ),
        Err(PostSaleError::Domain(
            PostSaleDomainError::CancellationReasonRequired
        ))
    );
    assert_eq!(
        (cancellation_effects(&connection), stocks(&connection)),
        before
    );

    for (id, target_sale, expected) in [
        (
            "550e8400-e29b-41d4-a716-446655440084",
            sale_id + 999,
            PostSaleError::Domain(PostSaleDomainError::SaleNotFound),
        ),
        (
            "550e8400-e29b-41d4-a716-446655440085",
            {
                connection
                    .execute(
                        "INSERT INTO sales (request_id, status, total_centavos) VALUES ('pending-cancellation', 'pending', 0)",
                        [],
                    )
                    .unwrap();
                connection.last_insert_rowid()
            },
            PostSaleError::Domain(PostSaleDomainError::SaleNotConfirmed),
        ),
    ] {
        let error = {
            let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
            PostSaleUseCase::new(&mut factory, &repository)
                .cancel_sale(cancellation_request(request_id(id), target_sale, "reason"))
                .unwrap_err()
        };
        assert_eq!(error, expected);
        assert_eq!(
            (cancellation_effects(&connection), stocks(&connection)),
            before
        );
    }

    let id = request_id("550e8400-e29b-41d4-a716-446655440086");
    {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .cancel_sale(cancellation_request(id.clone(), sale_id, "first reason"))
            .unwrap();
    }
    let committed = (correction_snapshot(&connection), stocks(&connection));
    assert_eq!(committed.1, (8, 4));
    let cross_operation = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .create_return(return_request(id.clone(), sale_id, lines[0], lines[1]))
            .unwrap_err()
    };
    assert_eq!(cross_operation, PostSaleError::RequestConflict);
    assert_eq!(
        (correction_snapshot(&connection), stocks(&connection)),
        committed
    );
    for (request, expected) in [
        (
            cancellation_request(id, sale_id, "changed reason"),
            PostSaleError::RequestConflict,
        ),
        (
            cancellation_request(
                request_id("550e8400-e29b-41d4-a716-446655440087"),
                sale_id,
                "another reason",
            ),
            PostSaleError::Domain(PostSaleDomainError::CancellationAlreadyRecorded),
        ),
    ] {
        let error = {
            let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
            PostSaleUseCase::new(&mut factory, &repository)
                .cancel_sale(request)
                .unwrap_err()
        };
        assert_eq!(error, expected);
        assert_eq!(
            (correction_snapshot(&connection), stocks(&connection)),
            committed
        );
    }
    assert_eq!(original_facts(&connection, sale_id), original);
}

#[test]
fn injected_cancellation_failure_rolls_back_all_effects() {
    let mut connection = open_seeded_catalog().unwrap();
    let sale_id = confirm_two_line_sale(&mut connection);
    let original = original_facts(&connection, sale_id);
    connection
        .execute_batch(
            "CREATE TRIGGER reject_cancellation_line BEFORE INSERT ON sale_cancellation_lines \
             BEGIN SELECT RAISE(ABORT, 'injected cancellation failure'); END;",
        )
        .unwrap();
    let before = (cancellation_effects(&connection), stocks(&connection));
    let repository = SqlitePostSaleRepository;
    let error = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .cancel_sale(cancellation_request(
                request_id("550e8400-e29b-41d4-a716-446655440088"),
                sale_id,
                "storage fault",
            ))
            .unwrap_err()
    };
    assert_eq!(error, PostSaleError::PersistenceFailure);
    assert_eq!(
        (cancellation_effects(&connection), stocks(&connection)),
        before
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT CASE WHEN EXISTS(SELECT 1 FROM sale_cancellations WHERE sale_id = ?1) THEN 'cancelled' ELSE 'confirmed' END",
                [sale_id],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "confirmed"
    );
    assert_eq!(original_facts(&connection, sale_id), original);
}

#[test]
fn overlapping_return_and_cancellation_serialize_without_double_restoration() {
    let directory = temporary_directory("overlapping-cancellation");
    let config = production_database_config(&directory);
    let mut verifier = open_database(&config).unwrap();
    let sale_id = confirm_two_line_sale(&mut verifier);
    let lines = verifier
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY product_id")
        .unwrap()
        .query_map([sale_id], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let original = original_facts(&verifier, sale_id);
    let start = Arc::new(Barrier::new(3));
    let return_start = Arc::clone(&start);
    let return_config = config.clone();
    let return_lines = lines.clone();
    let returner = thread::spawn(move || {
        let mut connection = open_database(&return_config).unwrap();
        connection.busy_timeout(Duration::from_secs(1)).unwrap();
        let repository = SqlitePostSaleRepository;
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        return_start.wait();
        PostSaleUseCase::new(&mut factory, &repository).create_return(return_request(
            request_id("550e8400-e29b-41d4-a716-446655440089"),
            sale_id,
            return_lines[0],
            return_lines[1],
        ))
    });
    let cancellation_start = Arc::clone(&start);
    let cancellation_config = config.clone();
    let canceller = thread::spawn(move || {
        let mut connection = open_database(&cancellation_config).unwrap();
        connection.busy_timeout(Duration::from_secs(1)).unwrap();
        let repository = SqlitePostSaleRepository;
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        cancellation_start.wait();
        PostSaleUseCase::new(&mut factory, &repository).cancel_sale(cancellation_request(
            request_id("550e8400-e29b-41d4-a716-446655440090"),
            sale_id,
            "overlap",
        ))
    });
    start.wait();
    let return_outcome = returner.join().unwrap();
    let cancellation_outcome = canceller.join().unwrap();

    assert!(cancellation_outcome.is_ok());
    assert!(
        return_outcome.is_ok()
            || matches!(
                return_outcome,
                Err(PostSaleError::Domain(PostSaleDomainError::SaleCancelled))
            )
    );
    assert_eq!(
        cancellation_effects(&verifier),
        (
            1 + i64::from(return_outcome.is_ok()),
            1,
            2,
            2 - i64::from(return_outcome.is_ok())
        )
    );
    assert_eq!(
        scalar(
            &verifier,
            "SELECT COUNT(*) FROM inventory_movements WHERE movement_type IN ('return', 'cancellation')",
            [],
        ),
        2 + i64::from(return_outcome.is_ok())
    );
    assert_eq!(stocks(&verifier), (8, 4));
    assert_eq!(original_facts(&verifier, sale_id), original);
    for (line, sold) in lines.into_iter().zip([2, 1]) {
        assert_eq!(
            scalar(&verifier, "SELECT COALESCE((SELECT SUM(quantity) FROM sale_return_lines WHERE sale_line_id = ?1), 0) + COALESCE((SELECT restored_quantity FROM sale_cancellation_lines WHERE sale_line_id = ?1), 0)", [line]),
            sold,
        );
    }
    drop(verifier);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn conflicting_return_retry_leaves_persisted_effects_unchanged() {
    let mut connection = open_seeded_catalog().unwrap();
    let sale_id = confirm_two_line_sale(&mut connection);
    let lines = connection
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY product_id")
        .unwrap()
        .query_map([sale_id], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let id = request_id("550e8400-e29b-41d4-a716-446655440033");
    let repository = SqlitePostSaleRepository;
    {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .create_return(return_request(id.clone(), sale_id, lines[0], lines[1]))
            .unwrap();
    }
    let before = (return_effects(&connection), stocks(&connection));

    let conflict = CreateReturnRequest::new(
        id,
        sale_id,
        vec![RequestedReturnLine {
            sale_line_id: lines[0],
            quantity: 2,
        }],
    )
    .unwrap();
    let error = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .create_return(conflict)
            .unwrap_err()
    };
    assert_eq!(error, PostSaleError::RequestConflict);
    assert_eq!((return_effects(&connection), stocks(&connection)), before);
}

#[test]
fn injected_return_line_failure_rolls_back_every_effect() {
    let mut connection = open_seeded_catalog().unwrap();
    let sale_id = confirm_two_line_sale(&mut connection);
    let lines = connection
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY product_id")
        .unwrap()
        .query_map([sale_id], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER reject_return_line BEFORE INSERT ON sale_return_lines \
             BEGIN SELECT RAISE(ABORT, 'injected return failure'); END;",
        )
        .unwrap();
    let before = (return_effects(&connection), stocks(&connection));
    let repository = SqlitePostSaleRepository;
    let error = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .create_return(return_request(
                request_id("550e8400-e29b-41d4-a716-446655440034"),
                sale_id,
                lines[0],
                lines[1],
            ))
            .unwrap_err()
    };
    assert_eq!(error, PostSaleError::PersistenceFailure);
    assert_eq!((return_effects(&connection), stocks(&connection)), before);
}

#[test]
fn immediate_writer_busy_failure_has_no_partial_return_and_can_retry() {
    let directory = temporary_directory("busy-return");
    let config = production_database_config(&directory);
    let mut writer = open_database(&config).unwrap();
    let sale_id = confirm_two_line_sale(&mut writer);
    let lines = writer
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY product_id")
        .unwrap()
        .query_map([sale_id], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut contender = open_database(&config).unwrap();
    contender.busy_timeout(Duration::from_millis(1)).unwrap();
    let stock_before_contention = stocks(&contender);
    let repository = SqlitePostSaleRepository;
    let mut writer_factory = SqlitePostSaleTransactionFactory::new(&mut writer);
    let lock = writer_factory.begin_immediate().unwrap();
    let error = {
        let mut contender_factory = SqlitePostSaleTransactionFactory::new(&mut contender);
        PostSaleUseCase::new(&mut contender_factory, &repository)
            .create_return(return_request(
                request_id("550e8400-e29b-41d4-a716-446655440035"),
                sale_id,
                lines[0],
                lines[1],
            ))
            .unwrap_err()
    };
    assert_eq!(error, PostSaleError::PersistenceFailure);
    assert_eq!(return_effects(&contender), (0, 0, 0, 0, 0));
    assert_eq!(stocks(&contender), stock_before_contention);
    drop(lock);
    drop(writer_factory);

    let result = {
        let mut contender_factory = SqlitePostSaleTransactionFactory::new(&mut contender);
        PostSaleUseCase::new(&mut contender_factory, &repository).create_return(return_request(
            request_id("550e8400-e29b-41d4-a716-446655440035"),
            sale_id,
            lines[0],
            lines[1],
        ))
    };
    assert!(result.is_ok());
    assert_eq!(return_effects(&contender), (1, 1, 2, 2, 2));
    drop(contender);
    drop(writer);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn return_edge_rejections_leave_no_partial_effects() {
    let mut connection = open_seeded_catalog().unwrap();
    let sale_id = confirm_two_line_sale(&mut connection);
    let lines = connection
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY product_id")
        .unwrap()
        .query_map([sale_id], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let other_sale_id = confirm_sale(
        &mut connection,
        ConfirmSaleRequest {
            request_id: request_id("550e8400-e29b-41d4-a716-446655440042"),
            lines: vec![RequestedLine {
                product_id: 1,
                quantity: Quantity::new(1).unwrap(),
                negotiated_unit_price: MoneyCentavos::new(2500).unwrap(),
            }],
            payments: vec![Payment::cash(
                MoneyCentavos::new(2500).unwrap(),
                MoneyCentavos::new(2500).unwrap(),
                MoneyCentavos::new(0).unwrap(),
            )
            .unwrap()],
        },
    )
    .unwrap()
    .sale_id;
    let other_line = scalar(
        &connection,
        "SELECT id FROM sale_lines WHERE sale_id = ?1",
        [other_sale_id],
    );
    let before = (return_effects(&connection), stocks(&connection));

    for lines in [
        vec![RequestedReturnLine {
            sale_line_id: lines[0],
            quantity: 0,
        }],
        vec![RequestedReturnLine {
            sale_line_id: lines[0],
            quantity: -1,
        }],
    ] {
        assert_eq!(
            CreateReturnRequest::new(
                request_id("550e8400-e29b-41d4-a716-446655440036"),
                sale_id,
                lines
            ),
            Err(PostSaleError::InvalidRequest)
        );
    }
    assert_eq!(
        CreateReturnRequest::new(
            request_id("550e8400-e29b-41d4-a716-446655440037"),
            sale_id,
            vec![
                RequestedReturnLine {
                    sale_line_id: lines[0],
                    quantity: 1,
                },
                RequestedReturnLine {
                    sale_line_id: lines[0],
                    quantity: 1,
                },
            ],
        ),
        Err(PostSaleError::Domain(
            PostSaleDomainError::DuplicateSaleLine
        ))
    );
    assert_eq!((return_effects(&connection), stocks(&connection)), before);

    let repository = SqlitePostSaleRepository;
    for (id, target_sale, target_line, quantity, expected) in [
        (
            "550e8400-e29b-41d4-a716-446655440038",
            sale_id,
            lines[0],
            3,
            PostSaleError::Domain(PostSaleDomainError::QuantityExceedsRemaining),
        ),
        (
            "550e8400-e29b-41d4-a716-446655440039",
            sale_id,
            999_999,
            1,
            PostSaleError::Domain(PostSaleDomainError::SaleLineNotFound),
        ),
        (
            "550e8400-e29b-41d4-a716-446655440043",
            sale_id,
            other_line,
            1,
            PostSaleError::Domain(PostSaleDomainError::SaleLineNotFound),
        ),
        (
            "550e8400-e29b-41d4-a716-446655440040",
            sale_id + 999,
            lines[0],
            1,
            PostSaleError::Domain(PostSaleDomainError::SaleNotFound),
        ),
    ] {
        let request = CreateReturnRequest::new(
            request_id(id),
            target_sale,
            vec![RequestedReturnLine {
                sale_line_id: target_line,
                quantity,
            }],
        )
        .unwrap();
        let error = {
            let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
            PostSaleUseCase::new(&mut factory, &repository)
                .create_return(request)
                .unwrap_err()
        };
        assert_eq!(error, expected);
        assert_eq!((return_effects(&connection), stocks(&connection)), before);
    }
}

#[test]
fn missing_stock_row_rolls_back_prior_return_line_effects() {
    let mut connection = open_seeded_catalog().unwrap();
    let sale_id = confirm_two_line_sale(&mut connection);
    let lines = connection
        .prepare("SELECT id FROM sale_lines WHERE sale_id = ?1 ORDER BY product_id")
        .unwrap()
        .query_map([sale_id], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    connection
        .execute("DELETE FROM stock_balances WHERE product_id = 3", [])
        .unwrap();
    let before = (
        return_effects(&connection),
        scalar(
            &connection,
            "SELECT quantity FROM stock_balances WHERE product_id = 1",
            [],
        ),
        scalar(
            &connection,
            "SELECT COUNT(*) FROM stock_balances WHERE product_id = 3",
            [],
        ),
    );
    let repository = SqlitePostSaleRepository;
    let error = {
        let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
        PostSaleUseCase::new(&mut factory, &repository)
            .create_return(return_request(
                request_id("550e8400-e29b-41d4-a716-446655440041"),
                sale_id,
                lines[0],
                lines[1],
            ))
            .unwrap_err()
    };
    assert_eq!(error, PostSaleError::PersistenceFailure);
    assert_eq!(
        (
            return_effects(&connection),
            scalar(
                &connection,
                "SELECT quantity FROM stock_balances WHERE product_id = 1",
                [],
            ),
            scalar(
                &connection,
                "SELECT COUNT(*) FROM stock_balances WHERE product_id = 3",
                [],
            ),
        ),
        before
    );
}

#[test]
fn overlapping_returns_serialize_without_double_restoration() {
    let directory = temporary_directory("overlapping-returns");
    let config = production_database_config(&directory);
    let mut verifier = open_database(&config).unwrap();
    let sale_id = confirm_two_line_sale(&mut verifier);
    let sale_line_id = scalar(
        &verifier,
        "SELECT id FROM sale_lines WHERE sale_id = ?1 AND product_id = 1",
        [sale_id],
    );
    let stock_before = stocks(&verifier);
    let start = Arc::new(Barrier::new(3));
    let outcomes = [
        "550e8400-e29b-41d4-a716-446655440044",
        "550e8400-e29b-41d4-a716-446655440045",
    ]
    .map(|id| {
        let config = config.clone();
        let start = Arc::clone(&start);
        thread::spawn(move || {
            let mut connection = open_database(&config).unwrap();
            connection.busy_timeout(Duration::from_secs(1)).unwrap();
            let repository = SqlitePostSaleRepository;
            let request = CreateReturnRequest::new(
                request_id(id),
                sale_id,
                vec![RequestedReturnLine {
                    sale_line_id,
                    quantity: 2,
                }],
            )
            .unwrap();
            let mut factory = SqlitePostSaleTransactionFactory::new(&mut connection);
            start.wait();
            PostSaleUseCase::new(&mut factory, &repository).create_return(request)
        })
    });
    start.wait();
    let outcomes = outcomes.map(|handle| handle.join().unwrap());

    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert!(outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            Err(PostSaleError::Domain(
                PostSaleDomainError::QuantityExceedsRemaining
            ))
        )
    }));
    assert_eq!(return_effects(&verifier), (1, 1, 1, 1, 1));
    assert_eq!(
        scalar(
            &verifier,
            "SELECT quantity FROM sale_return_lines WHERE sale_line_id = ?1",
            [sale_line_id],
        ),
        2
    );
    assert_eq!(stocks(&verifier), (stock_before.0 + 2, stock_before.1));
    drop(verifier);
    std::fs::remove_dir_all(directory).unwrap();
}
