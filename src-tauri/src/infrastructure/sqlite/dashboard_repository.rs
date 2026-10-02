use rusqlite::{params, Connection};

use crate::application::reporting::{
    DashboardMetrics, DashboardPayment, DashboardPeriod, DashboardProduct, DashboardRange,
    DashboardReader, DashboardRecentSale, DashboardReport, DashboardStockAlert,
    GrossProfitReader, GrossProfitReport, GrossProfitCostState, GrossProfitOperationsPage,
    GrossProfitOperationsPagination, GrossProfitOperationsReader, GrossProfitOperationKind,
    GrossProfitOperationRow, RealizedGrossProfit, ReportingError, GrossProfitOperationsExportReadError,
};

pub const DASHBOARD_TOP_PRODUCTS_LIMIT: i64 = 5;
pub const DASHBOARD_RECENT_SALES_LIMIT: i64 = 8;
pub const DASHBOARD_STOCK_ALERTS_LIMIT: i64 = 8;

pub struct SqliteDashboardReader<'connection>(pub &'connection Connection);

impl<'connection> SqliteDashboardReader<'connection> {
    pub fn new(connection: &'connection Connection) -> Self { Self(connection) }

    fn metrics(connection: &Connection, range: &DashboardRange) -> Result<DashboardMetrics, ReportingError> {
        let (from, to) = range.bounds();
        let row = connection.query_row(
            "SELECT
                COUNT(CASE WHEN NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id) THEN 1 END),
                COALESCE(SUM(CASE WHEN NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id) THEN s.total_centavos ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id) THEN
                    (SELECT COALESCE(SUM(l.quantity - COALESCE((SELECT SUM(r.quantity) FROM sale_return_lines r WHERE r.sale_line_id = l.id), 0)), 0)
                     FROM sale_lines l WHERE l.sale_id = s.id) ELSE 0 END), 0),
                COUNT(CASE WHEN EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id) THEN 1 END)
             FROM sales s
             WHERE s.status = 'confirmed' AND s.confirmed_at >= ?1 AND s.confirmed_at < ?2",
            params![from, to],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?)),
        ).map_err(|_| ReportingError::Persistence)?;
        Ok(DashboardMetrics {
            effective_sale_count: non_negative(row.0)?,
            effective_total_centavos: non_negative(row.1)?,
            net_units_out: non_negative(row.2)?,
            cancelled_sale_count: non_negative(row.3)?,
            realized_gross_profit: Self::realized_gross_profit(connection, range)?,
        })
    }

    fn realized_gross_profit(
        connection: &Connection,
        range: &DashboardRange,
    ) -> Result<RealizedGrossProfit, ReportingError> {
        let (from, to) = range.bounds();
        let mut statement = connection
            .prepare(
                "SELECT l.quantity, l.negotiated_unit_price_centavos, l.unit_cost_snapshot_centavos,
                        COALESCE((SELECT SUM(r.quantity) FROM sale_return_lines r WHERE r.sale_line_id = l.id), 0)
                 FROM sale_lines l JOIN sales s ON s.id = l.sale_id
                 WHERE s.status = 'confirmed' AND s.confirmed_at >= ?1 AND s.confirmed_at < ?2
                   AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)",
            )
            .map_err(|_| ReportingError::Persistence)?;
        let rows = statement
            .query_map(params![from, to], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(|_| ReportingError::Persistence)?;
        let mut total = 0_i64;
        let mut missing_cost_line_count = 0_i64;
        for row in rows {
            let (quantity, unit_price, unit_cost, returned_quantity) =
                row.map_err(|_| ReportingError::Persistence)?;
            if quantity <= 0
                || unit_price < 0
                || returned_quantity < 0
                || returned_quantity > quantity
            {
                return Err(ReportingError::PersistedDataInvalid);
            }
            let net_units = quantity
                .checked_sub(returned_quantity)
                .ok_or(ReportingError::PersistedDataInvalid)?;
            if let Some(unit_cost) = unit_cost {
                if unit_cost < 1 {
                    return Err(ReportingError::PersistedDataInvalid);
                }
                let line_profit = unit_price
                    .checked_sub(unit_cost)
                    .and_then(|margin| margin.checked_mul(net_units))
                    .ok_or(ReportingError::PersistedDataInvalid)?;
                total = total
                    .checked_add(line_profit)
                    .ok_or(ReportingError::PersistedDataInvalid)?;
            } else {
                missing_cost_line_count = missing_cost_line_count
                    .checked_add(1)
                    .ok_or(ReportingError::PersistedDataInvalid)?;
            }
        }
        Ok(RealizedGrossProfit {
            amount_centavos: total,
            missing_cost_line_count,
        })
    }

    fn gross_profit(connection: &Connection, range: &DashboardRange) -> Result<GrossProfitReport, ReportingError> {
        let (from, to) = range.bounds();
        let mut statement = connection.prepare(
            "SELECT l.quantity, l.negotiated_unit_price_centavos, l.unit_cost_snapshot_centavos,
                    CASE WHEN s.confirmed_at >= ?1 AND s.confirmed_at < ?2 THEN 1 ELSE 0 END,
                    COALESCE((SELECT SUM(rl.quantity)
                              FROM sale_return_lines rl JOIN sale_returns r ON r.id = rl.return_id
                              WHERE rl.sale_line_id = l.id AND r.occurred_at >= ?1 AND r.occurred_at < ?2), 0)
             FROM sale_lines l JOIN sales s ON s.id = l.sale_id
             WHERE s.status = 'confirmed'
               AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)
               AND ((s.confirmed_at >= ?1 AND s.confirmed_at < ?2)
                    OR EXISTS (SELECT 1 FROM sale_return_lines rl JOIN sale_returns r ON r.id = rl.return_id
                               WHERE rl.sale_line_id = l.id AND r.occurred_at >= ?1 AND r.occurred_at < ?2))"
        ).map_err(|_| ReportingError::Persistence)?;
        let rows = statement.query_map(params![from, to], |row| Ok((
            row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, Option<i64>>(2)?,
            row.get::<_, i64>(3)?, row.get::<_, i64>(4)?,
        ))).map_err(|_| ReportingError::Persistence)?;
        let mut amount_centavos = 0_i64;
        let mut missing_cost_line_count = 0_i64;
        for row in rows {
            let (quantity, unit_price, unit_cost, sale_in_range, returned_in_range) = row.map_err(|_| ReportingError::Persistence)?;
            if quantity <= 0 || unit_price < 0 || returned_in_range < 0 || returned_in_range > quantity || !(0..=1).contains(&sale_in_range) {
                return Err(ReportingError::PersistedDataInvalid);
            }
            let Some(unit_cost) = unit_cost else {
                missing_cost_line_count = missing_cost_line_count.checked_add(1).ok_or(ReportingError::PersistedDataInvalid)?;
                continue;
            };
            if unit_cost < 1 { return Err(ReportingError::PersistedDataInvalid); }
            let sold_quantity = if sale_in_range == 1 { quantity } else { 0 };
            let net_quantity = sold_quantity.checked_sub(returned_in_range).ok_or(ReportingError::PersistedDataInvalid)?;
            let line_amount = unit_price.checked_sub(unit_cost)
                .and_then(|margin| margin.checked_mul(net_quantity))
                .ok_or(ReportingError::PersistedDataInvalid)?;
            amount_centavos = amount_centavos.checked_add(line_amount).ok_or(ReportingError::PersistedDataInvalid)?;
        }
        let activity_count = connection.query_row(
            "SELECT COUNT(*) FROM (
                SELECT 'sale' AS activity_kind, s.id AS activity_id
                FROM sales s
                WHERE s.status = 'confirmed' AND s.confirmed_at >= ?1 AND s.confirmed_at < ?2
                  AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)
                UNION
                SELECT 'return' AS activity_kind, r.id AS activity_id
                FROM sale_returns r
                JOIN sales s ON s.id = r.sale_id
                WHERE s.status = 'confirmed'
                  AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)
                  AND r.occurred_at >= ?1 AND r.occurred_at < ?2
            )",
            params![from, to],
            |row| row.get::<_, i64>(0),
        ).map_err(|_| ReportingError::Persistence)?;
        Ok(GrossProfitReport {
            amount_centavos,
            missing_cost_line_count,
            activity_count: non_negative(activity_count)?,
        })
    }

    fn top_products(connection: &Connection, range: &DashboardRange) -> Result<Vec<DashboardProduct>, ReportingError> {
        let (from, to) = range.bounds();
        let mut statement = connection.prepare(
            "WITH product_totals AS (
                SELECT l.product_id,
                       SUM(l.quantity - COALESCE((SELECT SUM(r.quantity) FROM sale_return_lines r WHERE r.sale_line_id = l.id), 0)) AS net_units
                FROM sale_lines l JOIN sales s ON s.id = l.sale_id
                WHERE s.status = 'confirmed' AND s.confirmed_at >= ?1 AND s.confirmed_at < ?2
                  AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)
                GROUP BY l.product_id
                HAVING net_units > 0
             )
             SELECT totals.product_id,
                    COALESCE(
                        (SELECT l.sku_snapshot
                         FROM sale_lines l JOIN sales s ON s.id = l.sale_id
                         WHERE l.product_id = totals.product_id
                           AND l.sku_snapshot IS NOT NULL
                           AND s.status = 'confirmed' AND s.confirmed_at >= ?1 AND s.confirmed_at < ?2
                           AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)
                         ORDER BY s.confirmed_at DESC, l.id DESC LIMIT 1),
                        p.sku
                    ),
                    COALESCE(
                        (SELECT l.product_name_snapshot
                         FROM sale_lines l JOIN sales s ON s.id = l.sale_id
                         WHERE l.product_id = totals.product_id
                           AND l.product_name_snapshot IS NOT NULL
                           AND s.status = 'confirmed' AND s.confirmed_at >= ?1 AND s.confirmed_at < ?2
                           AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)
                         ORDER BY s.confirmed_at DESC, l.id DESC LIMIT 1),
                        p.name
                    ),
                    totals.net_units
             FROM product_totals totals JOIN products p ON p.id = totals.product_id
             ORDER BY totals.net_units DESC, lower(COALESCE(
                 (SELECT l.product_name_snapshot
                  FROM sale_lines l JOIN sales s ON s.id = l.sale_id
                  WHERE l.product_id = totals.product_id
                    AND l.product_name_snapshot IS NOT NULL
                    AND s.status = 'confirmed' AND s.confirmed_at >= ?1 AND s.confirmed_at < ?2
                    AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)
                  ORDER BY s.confirmed_at DESC, l.id DESC LIMIT 1),
                 p.name
             )), totals.product_id
             LIMIT ?3",
        ).map_err(|_| ReportingError::Persistence)?;
        let rows = statement.query_map(params![from, to, DASHBOARD_TOP_PRODUCTS_LIMIT], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?))
        }).map_err(|_| ReportingError::Persistence)?;
        rows.map(|row| {
            let (product_id, sku, product_name, units) = row.map_err(|_| ReportingError::Persistence)?;
            Ok(DashboardProduct { product_id: positive(product_id)?, sku, product_name, net_units_out: positive(units)? })
        }).collect()
    }

    fn payment_distribution(connection: &Connection, range: &DashboardRange) -> Result<Vec<DashboardPayment>, ReportingError> {
        let (from, to) = range.bounds();
        let mut statement = connection.prepare(
            "SELECT p.method, SUM(p.amount_applied_centavos)
             FROM sale_payments p JOIN sales s ON s.id = p.sale_id
             WHERE s.status = 'confirmed' AND s.confirmed_at >= ?1 AND s.confirmed_at < ?2
               AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)
             GROUP BY p.method ORDER BY p.method",
        ).map_err(|_| ReportingError::Persistence)?;
        let rows = statement.query_map(params![from, to], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))).map_err(|_| ReportingError::Persistence)?;
        rows.map(|row| {
            let (method, amount) = row.map_err(|_| ReportingError::Persistence)?;
            if method != "cash" && method != "qr" { return Err(ReportingError::PersistedDataInvalid); }
            Ok(DashboardPayment { method, amount_applied_centavos: non_negative(amount)? })
        }).collect()
    }

    fn recent_sales(connection: &Connection) -> Result<Vec<DashboardRecentSale>, ReportingError> {
        let mut statement = connection.prepare(
            "SELECT s.id, s.confirmed_at,
                    CASE WHEN EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id) THEN 'cancelled' ELSE 'confirmed' END,
                    s.total_centavos
             FROM sales s WHERE s.status = 'confirmed'
             ORDER BY s.confirmed_at DESC, s.id DESC LIMIT ?1",
        ).map_err(|_| ReportingError::Persistence)?;
        let rows = statement.query_map([DASHBOARD_RECENT_SALES_LIMIT], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?))).map_err(|_| ReportingError::Persistence)?;
        rows.map(|row| {
            let (sale_id, confirmed_at, status, total) = row.map_err(|_| ReportingError::Persistence)?;
            Ok(DashboardRecentSale { sale_id: positive(sale_id)?, confirmed_at, status, total_centavos: non_negative(total)? })
        }).collect()
    }

    fn stock_alerts(connection: &Connection) -> Result<Vec<DashboardStockAlert>, ReportingError> {
        let mut statement = connection.prepare(
            "SELECT p.id, p.sku, p.name, s.quantity,
                    CASE WHEN s.quantity = 0 THEN 'out_of_stock' ELSE 'low_stock' END
             FROM products p JOIN categories c ON c.id = p.category_id
             JOIN stock_balances s ON s.product_id = p.id
             WHERE p.active = 1 AND c.active = 1 AND s.quantity <= p.low_stock_threshold
             ORDER BY s.quantity, lower(p.name), p.id LIMIT ?1",
        ).map_err(|_| ReportingError::Persistence)?;
        let rows = statement.query_map([DASHBOARD_STOCK_ALERTS_LIMIT], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?, row.get::<_, String>(4)?))).map_err(|_| ReportingError::Persistence)?;
        rows.map(|row| {
            let (product_id, sku, product_name, quantity, classification) = row.map_err(|_| ReportingError::Persistence)?;
            if classification != "out_of_stock" && classification != "low_stock" { return Err(ReportingError::PersistedDataInvalid); }
            Ok(DashboardStockAlert { product_id: positive(product_id)?, sku, product_name, quantity: non_negative(quantity)?, classification })
        }).collect()
    }
}

const GROSS_PROFIT_OPERATION_EVENTS: &str = "
    SELECT s.confirmed_at AS occurred_at, 'sale', s.id, NULL, l.product_name_snapshot AS product_name, l.sku_snapshot AS sku,
           l.quantity, l.negotiated_unit_price_centavos, l.unit_cost_snapshot_centavos, l.id AS sale_line_id
    FROM sale_lines l JOIN sales s ON s.id = l.sale_id
    WHERE s.status = 'confirmed' AND s.confirmed_at >= ?1 AND s.confirmed_at < ?2
      AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)
    UNION ALL
    SELECT r.occurred_at, 'return', s.id, r.id, l.product_name_snapshot, l.sku_snapshot,
           -rl.quantity, l.negotiated_unit_price_centavos, l.unit_cost_snapshot_centavos, l.id
    FROM sale_return_lines rl
    JOIN sale_returns r ON r.id = rl.return_id
    JOIN sales s ON s.id = rl.sale_id
    JOIN sale_lines l ON l.id = rl.sale_line_id AND l.sale_id = rl.sale_id
    WHERE s.status = 'confirmed' AND r.occurred_at >= ?1 AND r.occurred_at < ?2
      AND NOT EXISTS (SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id)";

impl GrossProfitOperationsReader for SqliteDashboardReader<'_> {
    fn read_gross_profit_operations(
        &self,
        range: &DashboardRange,
        pagination: &GrossProfitOperationsPagination,
    ) -> Result<GrossProfitOperationsPage, ReportingError> {
        let snapshot = self.0.unchecked_transaction().map_err(|_| ReportingError::Persistence)?;
        let (from, to) = range.bounds();
        let total: i64 = snapshot.query_row(
            &format!("SELECT COUNT(*) FROM ({GROSS_PROFIT_OPERATION_EVENTS})"),
            params![from, to],
            |row| row.get(0),
        ).map_err(|_| ReportingError::Persistence)?;
        let total = non_negative(total)?;
        let total_pages = if total == 0 { 0 } else {
            total.checked_add(pagination.page_size - 1)
                .ok_or(ReportingError::PersistedDataInvalid)? / pagination.page_size
        };
        let mut statement = snapshot.prepare(&format!(
            "SELECT * FROM ({GROSS_PROFIT_OPERATION_EVENTS})
             ORDER BY 1 DESC, 3 DESC, 2 DESC, 4 DESC, 10 DESC LIMIT ?3 OFFSET ?4"
        )).map_err(|_| ReportingError::Persistence)?;
        let rows = statement.query_map(
            params![from, to, pagination.page_size, pagination.offset()?],
            |row| Ok((
                row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?,
                row.get::<_, Option<i64>>(3)?, row.get::<_, String>(4)?, row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?, row.get::<_, i64>(7)?, row.get::<_, Option<i64>>(8)?,
            )),
        ).map_err(|_| ReportingError::Persistence)?;
        let mut result = Vec::new();
        for row in rows {
            let (occurred_at, kind, sale_id, return_id, product_name, sku, quantity, price, cost) =
                row.map_err(|_| ReportingError::Persistence)?;
            let operation_kind = match kind.as_str() {
                "sale" if return_id.is_none() && quantity > 0 => GrossProfitOperationKind::Sale,
                "return" if return_id.is_some() && quantity < 0 => GrossProfitOperationKind::Return,
                _ => return Err(ReportingError::PersistedDataInvalid),
            };
            positive(sale_id)?;
            if let Some(id) = return_id { positive(id)?; }
            if price < 0 { return Err(ReportingError::PersistedDataInvalid); }
            let (cost_state, profit) = match cost {
                Some(cost) if cost > 0 => (
                    GrossProfitCostState::Known,
                    Some(price.checked_sub(cost)
                        .and_then(|margin| margin.checked_mul(quantity))
                        .ok_or(ReportingError::PersistedDataInvalid)?),
                ),
                Some(_) => return Err(ReportingError::PersistedDataInvalid),
                None => (GrossProfitCostState::Unknown, None),
            };
            result.push(GrossProfitOperationRow {
                occurred_at, operation_kind, sale_id, return_id, product_name, sku,
                signed_quantity: quantity,
                negotiated_unit_price_centavos: price,
                unit_cost_snapshot_centavos: cost,
                cost_state,
                signed_gross_profit_centavos: profit,
            });
        }
        drop(statement);
        snapshot.commit().map_err(|_| ReportingError::Persistence)?;
        Ok(GrossProfitOperationsPage {
            rows: result, page: pagination.page, page_size: pagination.page_size,
            total, total_pages,
        })
    }

    fn read_all_gross_profit_operations(
        &self,
        range: &DashboardRange,
    ) -> Result<Vec<GrossProfitOperationRow>, GrossProfitOperationsExportReadError> {
        const MAX_EXPORT_OPERATIONS: i64 = 10_000;
        const MAX_PRODUCT_NAME_CHARS: i64 = 512;
        const MAX_SKU_CHARS: i64 = 128;
        const MAX_OCCURRED_AT_CHARS: i64 = 64;
        let snapshot = self.0.unchecked_transaction().map_err(|_| ReportingError::Persistence)?;
        let (from, to) = range.bounds();
        let count: i64 = snapshot.query_row(
            &format!("SELECT COUNT(*) FROM ({GROSS_PROFIT_OPERATION_EVENTS})"),
            params![from, to],
            |row| row.get(0),
        ).map_err(|_| ReportingError::Persistence)?;
        if count < 0 { return Err(ReportingError::PersistedDataInvalid.into()); }
        if count > MAX_EXPORT_OPERATIONS { return Err(GrossProfitOperationsExportReadError::LimitExceeded); }
        let invalid_cumulative_returns: i64 = snapshot.query_row(
            &format!("SELECT COUNT(*) FROM sale_lines l
                     WHERE l.id IN (SELECT sale_line_id FROM ({GROSS_PROFIT_OPERATION_EVENTS}))
                       AND COALESCE((SELECT SUM(rl.quantity) FROM sale_return_lines rl
                                     WHERE rl.sale_line_id = l.id), 0) > l.quantity"),
            params![from, to],
            |row| row.get(0),
        ).map_err(|_| ReportingError::Persistence)?;
        if invalid_cumulative_returns > 0 { return Err(ReportingError::PersistedDataInvalid.into()); }
        let oversized_text: i64 = snapshot.query_row(
            &format!("SELECT COUNT(*) FROM ({GROSS_PROFIT_OPERATION_EVENTS}) WHERE length(COALESCE(product_name, '')) > ?3 OR length(COALESCE(sku, '')) > ?4 OR length(occurred_at) > ?5"),
            params![from, to, MAX_PRODUCT_NAME_CHARS, MAX_SKU_CHARS, MAX_OCCURRED_AT_CHARS],
            |row| row.get(0),
        ).map_err(|_| ReportingError::Persistence)?;
        if oversized_text > 0 { return Err(GrossProfitOperationsExportReadError::LimitExceeded); }
        let mut statement = snapshot.prepare(&format!(
            "SELECT * FROM ({GROSS_PROFIT_OPERATION_EVENTS})
             ORDER BY 1 DESC, 3 DESC, 2 DESC, 4 DESC, 10 DESC"
        )).map_err(|_| ReportingError::Persistence)?;
        let rows = statement.query_map(params![from, to], |row| Ok((
            row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?,
            row.get::<_, Option<i64>>(3)?, row.get::<_, String>(4)?, row.get::<_, String>(5)?,
            row.get::<_, i64>(6)?, row.get::<_, i64>(7)?, row.get::<_, Option<i64>>(8)?,
        ))).map_err(|_| ReportingError::Persistence)?;
        let mut result = Vec::with_capacity(count as usize);
        for row in rows {
            let (occurred_at, kind, sale_id, return_id, product_name, sku, quantity, price, cost) =
                row.map_err(|_| ReportingError::Persistence)?;
            let operation_kind = match kind.as_str() {
                "sale" if return_id.is_none() && quantity > 0 => GrossProfitOperationKind::Sale,
                "return" if return_id.is_some() && quantity < 0 => GrossProfitOperationKind::Return,
                _ => return Err(ReportingError::PersistedDataInvalid.into()),
            };
            positive(sale_id)?;
            if let Some(id) = return_id { positive(id)?; }
            if price < 0 { return Err(ReportingError::PersistedDataInvalid.into()); }
            let (cost_state, profit) = match cost {
                Some(value) if value > 0 => (
                    GrossProfitCostState::Known,
                    Some(price.checked_sub(value).and_then(|margin| margin.checked_mul(quantity))
                        .ok_or(ReportingError::PersistedDataInvalid)?),
                ),
                Some(_) => return Err(ReportingError::PersistedDataInvalid.into()),
                None => (GrossProfitCostState::Unknown, None),
            };
            result.push(GrossProfitOperationRow {
                occurred_at, operation_kind, sale_id, return_id, product_name, sku,
                signed_quantity: quantity,
                negotiated_unit_price_centavos: price,
                unit_cost_snapshot_centavos: cost,
                cost_state,
                signed_gross_profit_centavos: profit,
            });
        }
        drop(statement);
        snapshot.commit().map_err(|_| ReportingError::Persistence)?;
        if result.len() != count as usize { return Err(ReportingError::Persistence.into()); }
        Ok(result)
    }
}

impl GrossProfitReader for SqliteDashboardReader<'_> {
    fn read_gross_profit(&self, range: &DashboardRange) -> Result<GrossProfitReport, ReportingError> {
        let snapshot = self.0.unchecked_transaction().map_err(|_| ReportingError::Persistence)?;
        let report = Self::gross_profit(&snapshot, range)?;
        snapshot.commit().map_err(|_| ReportingError::Persistence)?;
        Ok(report)
    }
}

impl DashboardReader for SqliteDashboardReader<'_> {
    fn read(&self, today: &DashboardRange, month: &DashboardRange) -> Result<DashboardReport, ReportingError> {
        let snapshot = self.0.unchecked_transaction().map_err(|_| ReportingError::Persistence)?;
        let report = DashboardReport {
            today: DashboardPeriod { metrics: Self::metrics(&snapshot, today)? },
            month: DashboardPeriod { metrics: Self::metrics(&snapshot, month)? },
            top_products: Self::top_products(&snapshot, month)?,
            payment_distribution: Self::payment_distribution(&snapshot, month)?,
            recent_sales: Self::recent_sales(&snapshot)?,
            stock_alerts: Self::stock_alerts(&snapshot)?,
        };
        snapshot.commit().map_err(|_| ReportingError::Persistence)?;
        Ok(report)
    }
}

fn non_negative(value: i64) -> Result<i64, ReportingError> { (value >= 0).then_some(value).ok_or(ReportingError::PersistedDataInvalid) }
fn positive(value: i64) -> Result<i64, ReportingError> { (value > 0).then_some(value).ok_or(ReportingError::PersistedDataInvalid) }
