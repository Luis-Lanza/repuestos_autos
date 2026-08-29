use rusqlite::{params, Connection, OptionalExtension};

use crate::application::sales::history::history_fetch_limit;
use crate::application::sales::{
    HistoricalLine, HistoricalPayment, HistoryError, HistoryRange, PaymentMethod,
    SaleHistoryDetail, SaleHistoryDetailReader, SaleHistoryPage, SaleHistorySummary,
    SaleHistorySummaryReader,
};
use crate::domain::{MoneyCentavos, Quantity};

type SummaryRow = (i64, String, String, i64, i64, i64, bool, bool);
type LineRow = (i64, Option<String>, Option<String>, i64, i64, i64);

pub struct SqliteSaleHistoryReader<'connection>(&'connection Connection);
impl<'connection> SqliteSaleHistoryReader<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self(connection)
    }
}
impl SaleHistorySummaryReader for SqliteSaleHistoryReader<'_> {
    fn list(&self, range: &HistoryRange) -> Result<SaleHistoryPage, HistoryError> {
        let (from, to) = range.bounds();
        let mut statement = self.0.prepare("SELECT s.id, s.confirmed_at, s.status, s.total_centavos, (SELECT COUNT(*) FROM sale_lines l WHERE l.sale_id = s.id), (SELECT COUNT(*) FROM sale_payments p WHERE p.sale_id = s.id), EXISTS(SELECT 1 FROM sale_payments p WHERE p.sale_id = s.id AND p.method = 'cash'), EXISTS(SELECT 1 FROM sale_payments p WHERE p.sale_id = s.id AND p.method = 'qr') FROM sales s WHERE s.status = 'confirmed' AND s.confirmed_at >= ?1 AND s.confirmed_at < ?2 ORDER BY s.confirmed_at DESC, s.id DESC LIMIT ?3").map_err(|_| HistoryError::Persistence)?;
        let rows = statement
            .query_map(params![from, to, history_fetch_limit() as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, bool>(6)?,
                    row.get::<_, bool>(7)?,
                ))
            })
            .map_err(|_| HistoryError::Persistence)?;
        let summaries = rows
            .map(|row| {
                row.map_err(|_| HistoryError::Persistence)
                    .and_then(summary_from_row)
            })
            .collect::<Result<Vec<_>, _>>()?;
        SaleHistoryPage::from_overfetch(summaries)
    }
}
fn summary_from_row(
    (sale_id, confirmed_at, status, total, lines, payments, cash, qr): SummaryRow,
) -> Result<SaleHistorySummary, HistoryError> {
    let mut payment_methods = Vec::with_capacity(2);
    if cash {
        payment_methods.push(PaymentMethod::Cash);
    }
    if qr {
        payment_methods.push(PaymentMethod::Qr);
    }
    Ok(SaleHistorySummary {
        sale_id,
        confirmed_at,
        status,
        total_centavos: money(total)?,
        line_count: count(lines)?,
        payment_count: count(payments)?,
        payment_methods,
    })
}
impl SaleHistoryDetailReader for SqliteSaleHistoryReader<'_> {
    fn detail(&self, sale_id: i64) -> Result<Option<SaleHistoryDetail>, HistoryError> {
        let sale = self.0.query_row("SELECT id, confirmed_at, status, total_centavos FROM sales WHERE id = ?1 AND status = 'confirmed'", [sale_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?))).optional().map_err(|_| HistoryError::Persistence)?;
        let Some((sale_id, confirmed_at, status, total)) = sale else {
            return Ok(None);
        };
        let mut lines_statement = self.0.prepare("SELECT product_id, sku_snapshot, product_name_snapshot, quantity, negotiated_unit_price_centavos, line_total_centavos FROM sale_lines WHERE sale_id = ?1 ORDER BY id").map_err(|_| HistoryError::Persistence)?;
        let lines = lines_statement
            .query_map([sale_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .map_err(|_| HistoryError::Persistence)?
            .map(|row| {
                row.map_err(|_| HistoryError::Persistence)
                    .and_then(line_from_row)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut payments_statement = self.0.prepare("SELECT method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos FROM sale_payments WHERE sale_id = ?1 ORDER BY id").map_err(|_| HistoryError::Persistence)?;
        let payments = payments_statement
            .query_map([sale_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                ))
            })
            .map_err(|_| HistoryError::Persistence)?
            .map(|row| {
                row.map_err(|_| HistoryError::Persistence)
                    .and_then(payment_from_row)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(SaleHistoryDetail {
            sale_id,
            confirmed_at,
            status,
            total_centavos: money(total)?,
            lines,
            payments,
        }))
    }
}
fn line_from_row(
    (product_id, sku, product_name, quantity, unit_price, line_total): LineRow,
) -> Result<HistoricalLine, HistoryError> {
    Ok(HistoricalLine {
        product_id,
        sku,
        product_name,
        quantity: Quantity::new(quantity).map_err(|_| HistoryError::PersistedDataInvalid)?,
        unit_price_centavos: money(unit_price)?,
        line_total_centavos: money(line_total)?,
    })
}
fn payment_from_row(
    (method, applied, tendered, change): (String, i64, Option<i64>, Option<i64>),
) -> Result<HistoricalPayment, HistoryError> {
    match method.as_str() {
        "cash" => Ok(HistoricalPayment::Cash {
            amount_applied_centavos: money(applied)?,
            amount_tendered_centavos: money(tendered.ok_or(HistoryError::PersistedDataInvalid)?)?,
            change_given_centavos: money(change.ok_or(HistoryError::PersistedDataInvalid)?)?,
        }),
        "qr" if tendered.is_none() && change.is_none() => Ok(HistoricalPayment::Qr {
            amount_applied_centavos: money(applied)?,
        }),
        _ => Err(HistoryError::PersistedDataInvalid),
    }
}
fn money(value: i64) -> Result<MoneyCentavos, HistoryError> {
    MoneyCentavos::new(value).map_err(|_| HistoryError::PersistedDataInvalid)
}
fn count(value: i64) -> Result<u32, HistoryError> {
    u32::try_from(value).map_err(|_| HistoryError::PersistedDataInvalid)
}
