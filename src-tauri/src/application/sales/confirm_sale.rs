use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::domain::sales::{Payment, Sale, SaleLine};
use crate::domain::{MoneyCentavos, Quantity, RequestId};

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
    pub lines: Vec<PersistedLine>,
    pub payments: Vec<Payment>,
    pub total: MoneyCentavos,
}

pub fn confirm_sale(
    connection: &mut Connection,
    request: ConfirmSaleRequest,
) -> Result<PersistedSaleSummary, String> {
    let transaction = connection.transaction().map_err(database_error)?;
    let request_id = request.request_id.as_uuid().to_string();
    if transaction
        .execute(
            "INSERT INTO sales (request_id, status, total_centavos) VALUES (?1, 'pending', 0) ON CONFLICT(request_id) DO NOTHING",
            [&request_id],
        )
        .map_err(database_error)?
        == 0
    {
        return load_summary(&transaction, &request_id);
    }

    let lines = request
        .lines
        .into_iter()
        .map(|line| current_line(&transaction, line))
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
        insert_payment(&transaction, sale_id, *payment)?;
    }
    for (line, line_id) in sale.lines().iter().zip(line_ids) {
        if transaction.execute("UPDATE stock_balances SET quantity = quantity - ?1 WHERE product_id = ?2 AND quantity >= ?1", params![line.quantity().value(), line.product_id()]).map_err(database_error)? != 1 { return Err("insufficient stock".into()); }
        transaction.execute("INSERT INTO inventory_movements (product_id, sale_id, sale_line_id, quantity_delta) VALUES (?1, ?2, ?3, ?4)", params![line.product_id(), sale_id, line_id, -line.quantity().value()]).map_err(database_error)?;
    }
    transaction.execute("UPDATE sales SET status = 'confirmed', total_centavos = ?1, confirmed_at = CURRENT_TIMESTAMP WHERE id = ?2", params![sale.total().value(), sale_id]).map_err(database_error)?;
    let summary = load_summary(&transaction, &request_id)?;
    transaction.commit().map_err(database_error)?;
    Ok(summary)
}

fn current_line(transaction: &Transaction, line: RequestedLine) -> Result<SaleLine, String> {
    let product = transaction
        .query_row(
            "SELECT active, minimum_unit_price_centavos FROM products WHERE id = ?1",
            [line.product_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()
        .map_err(database_error)?;
    match product {
        Some((1, minimum)) => SaleLine::new(
            line.product_id,
            line.quantity,
            line.negotiated_unit_price,
            MoneyCentavos::new(minimum).map_err(str::to_owned)?,
        )
        .map_err(str::to_owned),
        Some(_) => Err("product is inactive".into()),
        None => Err("product is missing".into()),
    }
}

fn insert_payment(transaction: &Transaction, sale_id: i64, payment: Payment) -> Result<(), String> {
    match payment {
        Payment::Cash { amount_applied, amount_tendered, change_given } => transaction.execute("INSERT INTO sale_payments (sale_id, method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos) VALUES (?1, 'cash', ?2, ?3, ?4)", params![sale_id, amount_applied.value(), amount_tendered.value(), change_given.value()]),
        Payment::Qr { amount_applied } => transaction.execute("INSERT INTO sale_payments (sale_id, method, amount_applied_centavos) VALUES (?1, 'qr', ?2)", params![sale_id, amount_applied.value()]),
    }.map(|_| ()).map_err(database_error)
}

fn load_summary(
    transaction: &Transaction,
    request_id: &str,
) -> Result<PersistedSaleSummary, String> {
    let (sale_id, status, total) = transaction.query_row("SELECT id, status, total_centavos FROM sales WHERE request_id = ?1 AND status = 'confirmed'", [request_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).map_err(database_error)?;
    let lines = transaction.prepare("SELECT l.product_id, p.sku, p.name, l.quantity, l.negotiated_unit_price_centavos, l.minimum_unit_price_snapshot_centavos, l.line_total_centavos FROM sale_lines l JOIN products p ON p.id = l.product_id WHERE l.sale_id = ?1 ORDER BY l.id").map_err(database_error)?.query_map([sale_id], |row| Ok(PersistedLine { product_id: row.get(0)?, sku: row.get(1)?, product_name: row.get(2)?, quantity: Quantity::new(row.get(3)?).unwrap(), negotiated_unit_price: MoneyCentavos::new(row.get(4)?).unwrap(), minimum_unit_price_snapshot: MoneyCentavos::new(row.get(5)?).unwrap(), line_total: MoneyCentavos::new(row.get(6)?).unwrap() })).map_err(database_error)?.collect::<Result<Vec<_>, _>>().map_err(database_error)?;
    let payments = transaction.prepare("SELECT method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos FROM sale_payments WHERE sale_id = ?1 ORDER BY id").map_err(database_error)?.query_map([sale_id], |row| { let applied = MoneyCentavos::new(row.get(1)?).unwrap(); Ok(if row.get::<_, String>(0)? == "cash" { Payment::cash(applied, MoneyCentavos::new(row.get(2)?).unwrap(), MoneyCentavos::new(row.get(3)?).unwrap()).unwrap() } else { Payment::qr(applied) }) }).map_err(database_error)?.collect::<Result<Vec<_>, _>>().map_err(database_error)?;
    if lines.is_empty() || payments.is_empty() {
        return Err("persistence integrity failure".into());
    }
    Ok(PersistedSaleSummary {
        sale_id,
        request_id: RequestId::parse(request_id).map_err(|_| "persistence integrity failure")?,
        status,
        lines,
        payments,
        total: MoneyCentavos::new(total).map_err(str::to_owned)?,
    })
}

fn database_error(error: rusqlite::Error) -> String {
    error.to_string()
}
