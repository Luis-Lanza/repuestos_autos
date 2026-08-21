use rusqlite::{params, Connection, Transaction};

use crate::domain::sales::{Payment, Sale};
use crate::domain::{MoneyCentavos, Quantity, RequestId};
use crate::infrastructure::sqlite::sale_repository::SqliteSaleRepository;

use super::SaleRepository;

pub struct RequestedLine {
    pub product_id: i64,
    pub quantity: Quantity,
    pub negotiated_unit_price: MoneyCentavos,
}

pub struct ConfirmSaleRequest {
    pub request_id: RequestId,
    pub lines: Vec<RequestedLine>,
    pub payments: Vec<Payment>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PersistedLine {
    pub product_id: i64,
    pub sku: String,
    pub product_name: String,
    pub quantity: Quantity,
    pub negotiated_unit_price: MoneyCentavos,
    pub minimum_unit_price_snapshot: MoneyCentavos,
    pub line_total: MoneyCentavos,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PersistedSaleSummary {
    pub sale_id: i64,
    pub request_id: RequestId,
    pub status: String,
    pub confirmed_at: String,
    pub lines: Vec<PersistedLine>,
    pub payments: Vec<Payment>,
    pub total: MoneyCentavos,
}

pub fn confirm_sale(
    connection: &mut Connection,
    request: ConfirmSaleRequest,
) -> Result<PersistedSaleSummary, String> {
    in_transaction(connection, |transaction| {
        let repository = SqliteSaleRepository;
        let request_id = request.request_id.as_uuid().to_string();
        if !repository.reserve_request_id(transaction, &request_id)? {
            return repository.load_summary(transaction, &request_id);
        }

        let lines = request
            .lines
            .into_iter()
            .map(|line| repository.current_line(transaction, line))
            .collect::<Result<Vec<_>, _>>()?;
        let sale = Sale::new(lines, request.payments).map_err(str::to_owned)?;
        let sale_id = transaction
            .query_row(
                "SELECT id FROM sales WHERE request_id = ?1",
                [&request_id],
                |row| row.get(0),
            )
            .map_err(database_error)?;

        let mut line_ids = Vec::new();
        for line in sale.lines() {
            transaction.execute(
                    "INSERT INTO sale_lines (sale_id, product_id, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![sale_id, line.product_id(), line.quantity().value(), line.negotiated_unit_price().value(), line.minimum_unit_price_snapshot().value(), line.total().value()],
                ).map_err(database_error)?;
            line_ids.push(transaction.last_insert_rowid());
        }
        for payment in sale.payments() {
            insert_payment(transaction, sale_id, *payment)?;
        }
        for (line, line_id) in sale.lines().iter().zip(line_ids) {
            if transaction.execute("UPDATE stock_balances SET quantity = quantity - ?1 WHERE product_id = ?2 AND quantity >= ?1", params![line.quantity().value(), line.product_id()]).map_err(database_error)? != 1 { return Err("insufficient stock".into()); }
            transaction.execute("INSERT INTO inventory_movements (product_id, sale_id, sale_line_id, quantity_delta) VALUES (?1, ?2, ?3, ?4)", params![line.product_id(), sale_id, line_id, -line.quantity().value()]).map_err(database_error)?;
        }
        transaction.execute("UPDATE sales SET status = 'confirmed', total_centavos = ?1, confirmed_at = CURRENT_TIMESTAMP WHERE id = ?2", params![sale.total().value(), sale_id]).map_err(database_error)?;
        repository.load_summary(transaction, &request_id)
    })
}

fn in_transaction<T>(
    connection: &mut Connection,
    operation: impl FnOnce(&Transaction) -> Result<T, String>,
) -> Result<T, String> {
    let transaction = connection.transaction().map_err(database_error)?;
    match operation(&transaction) {
        Ok(result) => transaction.commit().map(|_| result).map_err(database_error),
        Err(error) => transaction
            .rollback()
            .map(|_| Err(error))
            .map_err(database_error)?,
    }
}

fn insert_payment(transaction: &Transaction, sale_id: i64, payment: Payment) -> Result<(), String> {
    match payment {
        Payment::Cash { amount_applied, amount_tendered, change_given } => transaction.execute("INSERT INTO sale_payments (sale_id, method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos) VALUES (?1, 'cash', ?2, ?3, ?4)", params![sale_id, amount_applied.value(), amount_tendered.value(), change_given.value()]),
        Payment::Qr { amount_applied } => transaction.execute("INSERT INTO sale_payments (sale_id, method, amount_applied_centavos) VALUES (?1, 'qr', ?2)", params![sale_id, amount_applied.value()]),
    }.map(|_| ()).map_err(database_error)
}

fn database_error(error: rusqlite::Error) -> String {
    error.to_string()
}
