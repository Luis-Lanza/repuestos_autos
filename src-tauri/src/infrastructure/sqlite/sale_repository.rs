use rusqlite::{params, OptionalExtension, Transaction};

use crate::application::sales::{
    ApplicationRequestedLine, ConfirmSaleError, ConfirmSaleRepository, PersistedLine,
    PersistedSaleSummary, RequestedLine, Reservation, SaleRepository,
};
use crate::domain::sales::{Payment, Sale, SaleLine};
use crate::domain::{MoneyCentavos, Quantity, RequestId};

pub struct SqliteSaleRepository;

impl ConfirmSaleRepository for SqliteSaleRepository {
    fn reserve_or_load(
        &self,
        transaction: &Transaction<'_>,
        request_id: &RequestId,
    ) -> Result<Reservation, ConfirmSaleError> {
        let request_id = request_id.as_uuid().to_string();
        let reserved = transaction
            .execute(
                "INSERT INTO sales (request_id, status, total_centavos) VALUES (?1, 'pending', 0) ON CONFLICT(request_id) DO NOTHING",
                [&request_id],
            )
            .map_err(|_| ConfirmSaleError::Persistence)?;
        if reserved == 1 {
            return Ok(Reservation::Reserved);
        }

        match transaction
            .query_row(
                "SELECT status FROM sales WHERE request_id = ?1",
                [&request_id],
                |row| row.get::<_, String>(0),
            )
            .map_err(|_| ConfirmSaleError::Persistence)?
            .as_str()
        {
            "confirmed" => self
                .load_summary(transaction, &request_id)
                .map(Reservation::ExistingConfirmed)
                .map_err(|_| ConfirmSaleError::Persistence),
            "pending" => Ok(Reservation::ExistingIncomplete),
            _ => Ok(Reservation::ExistingCorrupt),
        }
    }

    fn resolve_lines(
        &self,
        transaction: &Transaction<'_>,
        requested: &[ApplicationRequestedLine],
    ) -> Result<Vec<SaleLine>, ConfirmSaleError> {
        requested
            .iter()
            .map(|line| {
                let product = transaction
                    .query_row(
                        "SELECT active, minimum_unit_price_centavos FROM products WHERE id = ?1",
                        [line.product_id],
                        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
                    )
                    .optional()
                    .map_err(|_| ConfirmSaleError::Persistence)?;
                match product {
                    Some((1, catalog_price)) => SaleLine::priced(
                        line.product_id,
                        line.quantity,
                        MoneyCentavos::new(catalog_price)
                            .map_err(|_| ConfirmSaleError::PersistedDataInvalid)?,
                    )
                    .map_err(|_| ConfirmSaleError::MoneyOverflow),
                    Some(_) => Err(ConfirmSaleError::ProductInactive),
                    None => Err(ConfirmSaleError::ProductMissing),
                }
            })
            .collect()
    }

    fn persist_confirmed(
        &self,
        transaction: &Transaction<'_>,
        request_id: &RequestId,
        sale: &Sale,
    ) -> Result<PersistedSaleSummary, ConfirmSaleError> {
        let request_id = request_id.as_uuid().to_string();
        let sale_id = transaction
            .query_row(
                "SELECT id FROM sales WHERE request_id = ?1 AND status = 'pending'",
                [&request_id],
                |row| row.get(0),
            )
            .map_err(|_| ConfirmSaleError::PersistedDataInvalid)?;
        let mut line_ids = Vec::with_capacity(sale.lines().len());
        for line in sale.lines() {
            transaction
                .execute(
                    "INSERT INTO sale_lines (sale_id, product_id, sku_snapshot, product_name_snapshot, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos) SELECT ?1, ?2, sku, name, ?3, ?4, ?5, ?6 FROM products WHERE id = ?2",
                    params![sale_id, line.product_id(), line.quantity().value(), line.unit_price().value(), line.unit_price().value(), line.total().value()],
                )
                .map_err(|_| ConfirmSaleError::Persistence)?;
            line_ids.push(transaction.last_insert_rowid());
        }
        for payment in sale.payments() {
            insert_payment(transaction, sale_id, *payment)?;
        }
        for (line, line_id) in sale.lines().iter().zip(line_ids) {
            if transaction
                .execute(
                    "UPDATE stock_balances SET quantity = quantity - ?1 WHERE product_id = ?2 AND quantity >= ?1",
                    params![line.quantity().value(), line.product_id()],
                )
                .map_err(|_| ConfirmSaleError::Persistence)?
                != 1
            {
                return Err(ConfirmSaleError::InsufficientStock);
            }
            transaction
                .execute(
                    "INSERT INTO inventory_movements (product_id, sale_id, sale_line_id, movement_type, quantity_delta) VALUES (?1, ?2, ?3, 'sale', ?4)",
                    params![line.product_id(), sale_id, line_id, -line.quantity().value()],
                )
                .map_err(|_| ConfirmSaleError::Persistence)?;
        }
        transaction
            .execute(
                "UPDATE sales SET status = 'confirmed', total_centavos = ?1, confirmed_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![sale.total().value(), sale_id],
            )
            .map_err(|_| ConfirmSaleError::Persistence)?;
        self.load_summary(transaction, &request_id)
            .map_err(|_| ConfirmSaleError::PersistedDataInvalid)
    }
}

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
        let (sale_id, status, confirmed_at, total) = transaction
            .query_row(
                "SELECT id, status, confirmed_at, total_centavos FROM sales WHERE request_id = ?1 AND status = 'confirmed'",
                [request_id],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get::<_, Option<String>>(2)?,
                            row.get(3)?,
                        ))
                    },
            )
            .map_err(|_| integrity_error())?;
        let lines = transaction
            .prepare("SELECT l.product_id, COALESCE(l.sku_snapshot, p.sku), COALESCE(l.product_name_snapshot, p.name), l.quantity, l.negotiated_unit_price_centavos, l.minimum_unit_price_snapshot_centavos, l.line_total_centavos FROM sale_lines l JOIN products p ON p.id = l.product_id WHERE l.sale_id = ?1 ORDER BY l.id")
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
            confirmed_at: confirmed_at.ok_or_else(integrity_error)?,
            lines,
            payments,
            total: MoneyCentavos::new(total).map_err(|_| integrity_error())?,
        })
    }
}

fn insert_payment(
    transaction: &Transaction<'_>,
    sale_id: i64,
    payment: Payment,
) -> Result<(), ConfirmSaleError> {
    match payment {
        Payment::Cash {
            amount_applied,
            amount_tendered,
            change_given,
        } => transaction.execute(
            "INSERT INTO sale_payments (sale_id, method, amount_applied_centavos, amount_tendered_centavos, change_given_centavos) VALUES (?1, 'cash', ?2, ?3, ?4)",
            params![sale_id, amount_applied.value(), amount_tendered.value(), change_given.value()],
        ),
        Payment::Qr { amount_applied } => transaction.execute(
            "INSERT INTO sale_payments (sale_id, method, amount_applied_centavos) VALUES (?1, 'qr', ?2)",
            params![sale_id, amount_applied.value()],
        ),
    }
    .map(|_| ())
    .map_err(|_| ConfirmSaleError::Persistence)
}

fn database_error(error: rusqlite::Error) -> String {
    error.to_string()
}
fn integrity_error() -> String {
    "persistence integrity failure".into()
}
