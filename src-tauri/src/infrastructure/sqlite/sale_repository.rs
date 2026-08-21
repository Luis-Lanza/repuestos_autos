use rusqlite::{OptionalExtension, Transaction};

use crate::application::sales::{
    PersistedLine, PersistedSaleSummary, RequestedLine, SaleRepository,
};
use crate::domain::sales::{Payment, SaleLine};
use crate::domain::{MoneyCentavos, Quantity, RequestId};

pub struct SqliteSaleRepository;

impl SaleRepository for SqliteSaleRepository {
    fn reserve_request_id(
        &self,
        transaction: &Transaction,
        request_id: &str,
    ) -> Result<bool, String> {
        transaction
            .execute(
                "INSERT INTO sales (request_id, status, total_centavos) VALUES (?1, 'pending', 0) ON CONFLICT(request_id) DO NOTHING",
                [request_id],
            )
            .map(|affected| affected == 1)
            .map_err(database_error)
    }

    fn current_line(
        &self,
        transaction: &Transaction,
        line: RequestedLine,
    ) -> Result<SaleLine, String> {
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

    fn load_summary(
        &self,
        transaction: &Transaction,
        request_id: &str,
    ) -> Result<PersistedSaleSummary, String> {
        let (sale_id, status, total) = transaction
            .query_row(
                "SELECT id, status, total_centavos FROM sales WHERE request_id = ?1 AND status = 'confirmed'",
                [request_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| integrity_error())?;
        let lines = transaction
            .prepare("SELECT l.product_id, p.sku, p.name, l.quantity, l.negotiated_unit_price_centavos, l.minimum_unit_price_snapshot_centavos, l.line_total_centavos FROM sale_lines l JOIN products p ON p.id = l.product_id WHERE l.sale_id = ?1 ORDER BY l.id")
            .map_err(database_error)?
            .query_map([sale_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)))
            .map_err(database_error)?
            .collect::<Result<Vec<(i64, String, String, i64, i64, i64, i64)>, _>>()
            .map_err(database_error)?
            .into_iter()
            .map(|(product_id, sku, product_name, quantity, negotiated, minimum, total)| Ok(PersistedLine {
                product_id, sku, product_name,
                quantity: Quantity::new(quantity).map_err(|_| integrity_error())?,
                negotiated_unit_price: MoneyCentavos::new(negotiated).map_err(|_| integrity_error())?,
                minimum_unit_price_snapshot: MoneyCentavos::new(minimum).map_err(|_| integrity_error())?,
                line_total: MoneyCentavos::new(total).map_err(|_| integrity_error())?,
            }))
            .collect::<Result<Vec<_>, String>>()?;
        let payments = transaction
            .prepare("SELECT method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos FROM sale_payments WHERE sale_id = ?1 ORDER BY id")
            .map_err(database_error)?
            .query_map([sale_id], |row| Ok((row.get::<_, String>(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
            .map_err(database_error)?
            .collect::<Result<Vec<(String, i64, Option<i64>, Option<i64>)>, _>>()
            .map_err(database_error)?
            .into_iter()
            .map(|(method, applied, tendered, change)| match method.as_str() {
                "cash" => Payment::cash(
                    MoneyCentavos::new(applied).map_err(|_| integrity_error())?,
                    MoneyCentavos::new(tendered.ok_or_else(integrity_error)?).map_err(|_| integrity_error())?,
                    MoneyCentavos::new(change.ok_or_else(integrity_error)?).map_err(|_| integrity_error())?,
                ).map_err(|_| integrity_error()),
                "qr" => Ok(Payment::qr(MoneyCentavos::new(applied).map_err(|_| integrity_error())?)),
                _ => Err(integrity_error()),
            })
            .collect::<Result<Vec<_>, String>>()?;
        if lines.is_empty() || payments.is_empty() {
            return Err(integrity_error());
        }
        Ok(PersistedSaleSummary {
            sale_id,
            request_id: RequestId::parse(request_id).map_err(|_| integrity_error())?,
            status,
            lines,
            payments,
            total: MoneyCentavos::new(total).map_err(|_| integrity_error())?,
        })
    }
}

fn database_error(error: rusqlite::Error) -> String {
    error.to_string()
}
fn integrity_error() -> String {
    "persistence integrity failure".into()
}
