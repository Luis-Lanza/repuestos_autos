use rusqlite::{params, Connection};

use crate::application::inventory::movement_ledger::{
    MovementLedgerError, MovementLedgerPage, MovementLedgerQuery, MovementLedgerReader,
    MovementLedgerProductOption, MovementLedgerProductOptionPage, MovementLedgerProductOptionsQuery, MovementLedgerRow,
};

pub struct SqliteMovementLedgerReader<'connection>(&'connection Connection);

impl<'connection> SqliteMovementLedgerReader<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self(connection)
    }
}

impl MovementLedgerReader for SqliteMovementLedgerReader<'_> {
    fn product_options(&self, query: &MovementLedgerProductOptionsQuery) -> Result<MovementLedgerProductOptionPage, MovementLedgerError> {
        let (pattern, offset, limit, page, page_size) = query.sql_parameters();
        let mut statement = self.0.prepare(
            "SELECT id, name, sku, active FROM products
             WHERE lower(name) LIKE lower(?1) ESCAPE '\\'
                OR lower(sku) LIKE lower(?1) ESCAPE '\\'
             ORDER BY lower(name), id LIMIT ?2 OFFSET ?3",
        ).map_err(|_| MovementLedgerError::Persistence)?;
        let rows = statement.query_map(params![pattern, limit, offset], |row| Ok(MovementLedgerProductOption {
            product_id: row.get(0)?,
            product_name: row.get(1)?,
            product_sku: row.get(2)?,
            active: row.get(3)?,
        })).map_err(|_| MovementLedgerError::Persistence)?;
        let mut products = rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| MovementLedgerError::PersistedDataInvalid)?;
        let has_more = products.len() > page_size as usize;
        if has_more { products.pop(); }
        Ok(MovementLedgerProductOptionPage { products, page, page_size, has_more })
    }

    fn list(&self, query: &MovementLedgerQuery) -> Result<MovementLedgerPage, MovementLedgerError> {
        let (from, to, product_id, movement_type, limit, offset) = query.sql_parameters();
        let mut statement = self.0.prepare(
            "SELECT m.id, m.occurred_at, m.product_id, p.name, p.sku, m.movement_type,
                    m.quantity_delta, m.resulting_quantity, m.reason,
                    CASE WHEN m.movement_type = 'stock_entry' THEN m.source_reference ELSE NULL END,
                    m.sale_id, m.sale_line_id
             FROM inventory_movements m
             JOIN products p ON p.id = m.product_id
             WHERE m.occurred_at >= ?1 AND m.occurred_at < ?2
               AND (?3 IS NULL OR m.product_id = ?3)
               AND (?4 IS NULL OR m.movement_type = ?4)
             ORDER BY m.occurred_at DESC, m.id DESC
             LIMIT ?5 OFFSET ?6",
        ).map_err(|_| MovementLedgerError::Persistence)?;
        let rows = statement
            .query_map(params![from, to, product_id, movement_type, limit, offset], |row| {
                Ok(MovementLedgerRow {
                    movement_id: row.get(0)?,
                    occurred_at: row.get(1)?,
                    product_id: row.get(2)?,
                    product_name: row.get(3)?,
                    product_sku: row.get(4)?,
                    movement_type: row.get(5)?,
                    quantity_delta: row.get(6)?,
                    resulting_quantity: row.get(7)?,
                    reason: row.get(8)?,
                    note: row.get(9)?,
                    sale_id: row.get(10)?,
                    sale_line_id: row.get(11)?,
                })
            })
            .map_err(|_| MovementLedgerError::Persistence)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| MovementLedgerError::PersistedDataInvalid)?;
        let has_more = rows.len() > query.page_size() as usize;
        let mut rows = rows;
        if has_more {
            rows.pop();
        }
        Ok(MovementLedgerPage {
            rows,
            page: query.page(),
            page_size: query.page_size(),
            has_more,
        })
    }
}
