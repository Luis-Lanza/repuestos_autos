use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

use crate::application::inventory::InventoryRepository;
use crate::domain::inventory::{
    InventoryAlert, InventoryError, InventoryOperation, OperationKind, PersistedInventoryOperation,
};
use crate::domain::RequestId;

pub struct SqliteInventoryRepository<'connection> {
    connection: &'connection mut Connection,
}

impl<'connection> SqliteInventoryRepository<'connection> {
    pub fn new(connection: &'connection mut Connection) -> Self {
        Self { connection }
    }
}

impl InventoryRepository for SqliteInventoryRepository<'_> {
    fn confirm(
        &mut self,
        operation: InventoryOperation,
    ) -> Result<PersistedInventoryOperation, InventoryError> {
        let request_id = match &operation {
            InventoryOperation::StockEntry { request_id, .. }
            | InventoryOperation::PhysicalCount { request_id, .. } => {
                request_id.as_uuid().to_string()
            }
        };
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| InventoryError::PERSISTENCE_FAILURE)?;
        if let Some(existing) = load_persisted(&transaction, &request_id)? {
            transaction
                .commit()
                .map_err(|_| InventoryError::PERSISTENCE_FAILURE)?;
            return Ok(existing);
        }

        let (product_id, kind, delta, counted_quantity, reason, note) = match operation {
            InventoryOperation::StockEntry {
                product_id,
                quantity,
                note,
                ..
            } => (
                product_id,
                OperationKind::StockEntry,
                quantity.value(),
                None,
                None,
                note,
            ),
            InventoryOperation::PhysicalCount {
                product_id,
                count,
                reason,
                ..
            } => (
                product_id,
                OperationKind::PhysicalCount,
                count.value(),
                Some(count.value()),
                Some(reason.as_str().to_owned()),
                None,
            ),
        };
        let product = transaction
            .query_row(
                "SELECT p.active, b.quantity FROM products p JOIN stock_balances b ON b.product_id = p.id WHERE p.id = ?1",
                [product_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()
            .map_err(|_| InventoryError::PERSISTENCE_FAILURE)?;
        let (active, previous_quantity) = product.ok_or(InventoryError::MISSING_PRODUCT)?;
        if active == 0 {
            return Err(InventoryError::INACTIVE_PRODUCT);
        }
        let (quantity_delta, resulting_quantity) = match kind {
            OperationKind::StockEntry => (
                delta,
                previous_quantity
                    .checked_add(delta)
                    .ok_or(InventoryError::QUANTITY_OVERFLOW)?,
            ),
            OperationKind::PhysicalCount => {
                let adjustment = delta
                    .checked_sub(previous_quantity)
                    .ok_or(InventoryError::QUANTITY_OVERFLOW)?;
                if adjustment == 0 {
                    return Err(InventoryError::UNCHANGED_COUNT);
                }
                (adjustment, delta)
            }
        };
        let movement_type = match kind {
            OperationKind::StockEntry => "stock_entry",
            OperationKind::PhysicalCount => "adjustment",
        };
        if transaction.execute(
            "INSERT INTO inventory_movements (product_id, movement_type, quantity_delta, reason, source_reference, request_id, counted_quantity, resulting_quantity) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![product_id, movement_type, quantity_delta, reason, note, request_id, counted_quantity, resulting_quantity],
        ).is_err() {
            let winner = load_persisted(&transaction, &request_id)?;
            transaction.commit().map_err(|_| InventoryError::PERSISTENCE_FAILURE)?;
            return winner.ok_or(InventoryError::PERSISTENCE_FAILURE);
        }
        if transaction
            .execute(
                "UPDATE stock_balances SET quantity = ?1 WHERE product_id = ?2 AND quantity = ?3",
                params![resulting_quantity, product_id, previous_quantity],
            )
            .map_err(|_| InventoryError::PERSISTENCE_FAILURE)?
            != 1
        {
            return Err(InventoryError::PERSISTENCE_FAILURE);
        }
        let persisted = load_persisted(&transaction, &request_id)?
            .ok_or(InventoryError::PERSISTED_DATA_INVALID)?;
        transaction
            .commit()
            .map_err(|_| InventoryError::PERSISTENCE_FAILURE)?;
        Ok(persisted)
    }

    fn list_alerts(&self) -> Result<Vec<InventoryAlert>, InventoryError> {
        let mut statement = self.connection.prepare(
            "SELECT p.id, p.name, p.active, b.quantity FROM products p JOIN stock_balances b ON b.product_id = p.id WHERE p.active = 1 AND b.quantity IN (0, 1) ORDER BY CASE b.quantity WHEN 0 THEN 0 ELSE 1 END, lower(p.name), p.id",
        ).map_err(|_| InventoryError::PERSISTENCE_FAILURE)?;
        let alerts = statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get(3)?,
                ))
            })
            .map_err(|_| InventoryError::PERSISTENCE_FAILURE)?
            .map(|row| {
                row.map_err(|_| InventoryError::PERSISTENCE_FAILURE)
                    .and_then(|(id, name, active, quantity)| {
                        InventoryAlert::for_product(id, &name, active != 0, quantity)
                            .ok_or(InventoryError::PERSISTED_DATA_INVALID)
                    })
            })
            .collect();
        alerts
    }
}

fn load_persisted(
    connection: &Connection,
    request_id: &str,
) -> Result<Option<PersistedInventoryOperation>, InventoryError> {
    connection.query_row(
        "SELECT movement_type, product_id, quantity_delta, resulting_quantity, occurred_at, source_reference FROM inventory_movements WHERE request_id = ?1",
        [request_id],
        |row| {
            Ok::<(String, i64, i64, i64, String, Option<String>), rusqlite::Error>((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    ).optional().map_err(|_| InventoryError::PERSISTENCE_FAILURE)?.map(|(movement_type, product_id, delta, resulting, occurred_at, note)| {
        let kind = match movement_type.as_str() { "stock_entry" => OperationKind::StockEntry, "adjustment" => OperationKind::PhysicalCount, _ => return Err(InventoryError::PERSISTED_DATA_INVALID) };
        let previous = resulting.checked_sub(delta).ok_or(InventoryError::QUANTITY_OVERFLOW)?;
        PersistedInventoryOperation::new(kind, RequestId::parse(request_id).map_err(|_| InventoryError::PERSISTED_DATA_INVALID)?, product_id, previous, delta, resulting, &occurred_at, note)
    }).transpose()
}
