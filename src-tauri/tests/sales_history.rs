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
