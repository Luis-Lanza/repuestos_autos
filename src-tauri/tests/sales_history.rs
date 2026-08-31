use repuestos_autos::application::sales::{
    HistoricalPayment, HistoryError, HistoryRange, PaymentMethod, SaleHistoryDetailReader,
    SaleHistorySummaryReader,
};
use repuestos_autos::infrastructure::sqlite::{
    open_seeded_catalog, sale_history_repository::SqliteSaleHistoryReader,
};
use rusqlite::{params, Connection};
fn insert_sale(connection: &Connection, id: i64, status: &str, confirmed_at: Option<&str>) {
    connection.execute("INSERT INTO sales (id, request_id, status, total_centavos, confirmed_at) VALUES (?1, ?2, ?3, 100, ?4)", params![id, format!("history-{id}"), status, confirmed_at]).unwrap();
}
fn insert_detail(connection: &Connection, sale_id: i64) {
    connection.execute("INSERT INTO sale_lines (sale_id, product_id, sku_snapshot, product_name_snapshot, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos) VALUES (?1, 1, NULL, NULL, 1, 100, 100, 100)", [sale_id]).unwrap();
    connection.execute("INSERT INTO sale_payments (sale_id, method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos) VALUES (?1, 'cash', 100, 150, 50), (?1, 'qr', 0, NULL, NULL)", [sale_id]).unwrap();
}
#[test]
#[rustfmt::skip]
fn sales_history_migrates_and_reads_a_fixed_immutable_newest_first_page() {
    let connection = open_seeded_catalog().unwrap(); for id in 1..=102 { insert_sale(&connection, id, "confirmed", Some("2025-01-02 07:00:00")); }
    insert_sale(&connection, 103, "pending", None); insert_sale(&connection, 104, "confirmed", Some("2025-01-03 07:00:00"));
    let before: i64 = connection.query_row("SELECT COUNT(*) FROM sales", [], |row| row.get(0)).unwrap();
    let reader = SqliteSaleHistoryReader::new(&connection); let range = HistoryRange::parse("2025-01-02T03:00:00-04:00", "2025-01-03T03:00:00-04:00").unwrap(); let page = reader.list(&range).unwrap();
    assert_eq!(HistoryRange::parse("2025-01-02T00:00:00Z", "2025-01-01T00:00:00Z"), Err(HistoryError::InvalidRange));
    assert_eq!(page.sales().len(), 100); assert!(page.has_more()); assert_eq!(page.sales()[0].sale_id, 102);
    assert_eq!(page.sales(), reader.list(&range).unwrap().sales()); assert_eq!(connection.query_row("SELECT COUNT(*) FROM sales", [], |row| row.get::<_, i64>(0)).unwrap(), before);
}

#[test]
#[rustfmt::skip]
fn sales_history_detail_keeps_snapshots_unavailable_and_rejects_invalid_facts() {
    let connection = open_seeded_catalog().unwrap(); insert_sale(&connection, 1, "confirmed", Some("2025-01-01 00:00:00")); insert_detail(&connection, 1);
    let reader = SqliteSaleHistoryReader::new(&connection); let detail = reader.detail(1).unwrap().unwrap();
    assert_eq!((detail.lines[0].sku.as_deref(), detail.lines[0].product_name.as_deref()), (None, None));
    assert!(matches!(detail.payments.as_slice(), [HistoricalPayment::Cash { amount_applied_centavos, amount_tendered_centavos, change_given_centavos }, HistoricalPayment::Qr { amount_applied_centavos: qr }] if amount_applied_centavos.value() == 100 && amount_tendered_centavos.value() == 150 && change_given_centavos.value() == 50 && qr.value() == 0));
    assert_eq!(reader.list(&HistoryRange::parse("2025-01-01T00:00:00Z", "2025-01-02T00:00:00Z").unwrap()).unwrap().sales()[0].payment_methods, vec![PaymentMethod::Cash, PaymentMethod::Qr]);
    connection.execute_batch("DROP TRIGGER confirmed_sale_lines_immutable_price; PRAGMA ignore_check_constraints = ON; UPDATE sale_lines SET quantity = 0 WHERE sale_id = 1; PRAGMA ignore_check_constraints = OFF;").unwrap(); assert_eq!(reader.detail(1), Err(HistoryError::PersistedDataInvalid));
    connection.execute_batch("PRAGMA ignore_check_constraints = ON; UPDATE sale_lines SET quantity = 1, negotiated_unit_price_centavos = -1 WHERE sale_id = 1; PRAGMA ignore_check_constraints = OFF;").unwrap(); assert_eq!(reader.detail(1), Err(HistoryError::PersistedDataInvalid));
}

fn request_id(value: &str) -> repuestos_autos::domain::RequestId {
    repuestos_autos::domain::RequestId::parse(value).unwrap()
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
    let lines = connection.prepare("SELECT id, sale_id, product_id, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos, sku_snapshot, product_name_snapshot FROM sale_lines WHERE sale_id = ?1 ORDER BY id").unwrap().query_map([sale_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    let payments = connection.prepare("SELECT id, sale_id, method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos FROM sale_payments WHERE sale_id = ?1 ORDER BY id").unwrap().query_map([sale_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    (sale, lines, payments)
}

fn corrected_sale(connection: &mut Connection) -> (i64, i64, OriginalFacts) {
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
            request_id: request_id("550e8400-e29b-41d4-a716-446655440101"),
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
    let before = original_facts(connection, sale_id);
    let repository = SqlitePostSaleRepository;
    let mut transactions = SqlitePostSaleTransactionFactory::new(connection);
    let mut use_case = PostSaleUseCase::new(&mut transactions, &repository);
    use_case
        .create_return(
            CreateReturnRequest::new(
                request_id("550e8400-e29b-41d4-a716-446655440102"),
                sale_id,
                vec![RequestedReturnLine {
                    sale_line_id: line_id,
                    quantity: 1,
                }],
            )
            .unwrap(),
        )
        .unwrap();
    use_case
        .create_return(
            CreateReturnRequest::new(
                request_id("550e8400-e29b-41d4-a716-446655440103"),
                sale_id,
                vec![RequestedReturnLine {
                    sale_line_id: line_id,
                    quantity: 1,
                }],
            )
            .unwrap(),
        )
        .unwrap();
    use_case
        .cancel_sale(
            CancelSaleRequest::new(
                request_id("550e8400-e29b-41d4-a716-446655440104"),
                sale_id,
                "inventory count".into(),
            )
            .unwrap(),
        )
        .unwrap();
    (sale_id, line_id, before)
}

#[test]
fn sales_history_lists_cancelled_corrections_and_line_aggregates() {
    let mut connection = open_seeded_catalog().unwrap();
    let (sale_id, line_id, before) = corrected_sale(&mut connection);
    let reader = SqliteSaleHistoryReader::new(&connection);
    let range = HistoryRange::parse("2000-01-01T00:00:00Z", "2100-01-01T00:00:00Z").unwrap();
    let summary = reader.list(&range).unwrap().sales()[0].clone();
    let detail = reader.detail(sale_id).unwrap().unwrap();
    let stored_status: String = connection
        .query_row("SELECT status FROM sales WHERE id = ?1", [sale_id], |row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(
        (
            summary.sale_id,
            summary.status.as_str(),
            summary.has_corrections
        ),
        (sale_id, "cancelled", true)
    );
    assert_eq!(stored_status, "confirmed");
    assert_eq!(detail.status, "cancelled");
    assert_eq!(before, original_facts(&connection, sale_id));
    assert_eq!(detail.lines.len(), 1);
    assert_eq!(
        (
            detail.lines[0].sale_line_id,
            detail.lines[0].returned_quantity,
            detail.lines[0].cancellation_restored_quantity,
            detail.lines[0].remaining_returnable_quantity
        ),
        (line_id, 2, 0, 0),
    );
}

#[test]
fn sales_history_reads_ordered_correction_details() {
    let mut connection = open_seeded_catalog().unwrap();
    let (sale_id, line_id, before) = corrected_sale(&mut connection);
    let detail = SqliteSaleHistoryReader::new(&connection)
        .detail(sale_id)
        .unwrap()
        .unwrap();

    assert_eq!(
        detail
            .returns
            .iter()
            .map(|returned| returned.request_id.as_str())
            .collect::<Vec<_>>(),
        [
            "550e8400-e29b-41d4-a716-446655440102",
            "550e8400-e29b-41d4-a716-446655440103"
        ]
    );
    assert!(detail.returns[0].return_id < detail.returns[1].return_id);
    assert!(!detail
        .returns
        .iter()
        .any(|returned| returned.occurred_at.is_empty()));
    assert_eq!(detail.returns[0].lines[0].sale_line_id, line_id);
    assert_eq!(detail.returns[1].lines[0].quantity, 1);
    let cancellation = detail.cancellation.unwrap();
    assert!(cancellation.cancellation_id > 0 && !cancellation.occurred_at.is_empty());
    assert_eq!(
        (
            cancellation.request_id.as_str(),
            cancellation.reason.as_str(),
            cancellation.lines[0].sale_line_id,
            cancellation.lines[0].restored_quantity
        ),
        (
            "550e8400-e29b-41d4-a716-446655440104",
            "inventory count",
            line_id,
            0
        )
    );
    assert_eq!(before, original_facts(&connection, sale_id));
}
