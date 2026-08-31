use rusqlite::{params, OptionalExtension, Transaction};

use crate::application::sales::{
    CancellationResult, PersistedRequest, PostSaleError, PostSaleFacts, PostSaleOperation,
    PostSaleRepository, ReturnResult, SaleLifecycleStatus,
};
use crate::domain::sales::{
    CancellationPlan, OriginalSaleLine, PostSaleDomainError, ReturnPlan, ReturnPlanLine,
    SaleCorrectionState,
};

pub struct SqlitePostSaleRepository;

impl PostSaleRepository for SqlitePostSaleRepository {
    fn find_request(
        &self,
        transaction: &Transaction<'_>,
        request_id: &str,
    ) -> Result<Option<PersistedRequest>, PostSaleError> {
        let request = transaction
            .query_row(
                "SELECT operation_kind, sale_id, payload_version, canonical_payload, payload_sha256 FROM post_sale_requests WHERE request_id = ?1",
                [request_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| PostSaleError::PersistenceFailure)?;
        request
            .map(
                |(operation, sale_id, payload_version, canonical_payload, payload_sha256)| {
                    let operation = match operation.as_str() {
                        "return" => PostSaleOperation::Return,
                        "cancellation" => PostSaleOperation::Cancellation,
                        _ => return Err(PostSaleError::PersistenceFailure),
                    };
                    Ok(PersistedRequest {
                        request_id: request_id.into(),
                        operation,
                        sale_id,
                        payload_version,
                        canonical_payload,
                        payload_sha256,
                    })
                },
            )
            .transpose()
    }

    fn load_facts(
        &self,
        transaction: &Transaction<'_>,
        sale_id: i64,
    ) -> Result<PostSaleFacts, PostSaleError> {
        let (status, cancelled) = transaction
            .query_row(
                "SELECT s.status, EXISTS(SELECT 1 FROM sale_cancellations c WHERE c.sale_id = s.id) FROM sales s WHERE s.id = ?1",
                [sale_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?)),
            )
            .optional()
            .map_err(|_| PostSaleError::PersistenceFailure)?
            .ok_or(PostSaleError::Domain(PostSaleDomainError::SaleNotFound))?;
        let state = match (status.as_str(), cancelled) {
            (_, true) => SaleCorrectionState::Cancelled,
            ("confirmed", false) => SaleCorrectionState::Confirmed,
            _ => SaleCorrectionState::NotConfirmed,
        };
        let mut statement = transaction
            .prepare(
                "SELECT l.id, l.product_id, l.quantity, COALESCE(SUM(rl.quantity), 0) FROM sale_lines l LEFT JOIN sale_return_lines rl ON rl.sale_line_id = l.id WHERE l.sale_id = ?1 GROUP BY l.id, l.product_id, l.quantity ORDER BY l.id",
            )
            .map_err(|_| PostSaleError::PersistenceFailure)?;
        let original_lines = statement
            .query_map([sale_id], |row| {
                Ok(OriginalSaleLine {
                    sale_line_id: row.get(0)?,
                    product_id: row.get(1)?,
                    sold_quantity: row.get(2)?,
                    returned_quantity: row.get(3)?,
                })
            })
            .map_err(|_| PostSaleError::PersistenceFailure)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| PostSaleError::PersistenceFailure)?;
        Ok(PostSaleFacts {
            state,
            original_lines,
        })
    }

    fn load_return_result(
        &self,
        transaction: &Transaction<'_>,
        request: &PersistedRequest,
    ) -> Result<ReturnResult, PostSaleError> {
        load_return(transaction, request)
    }

    fn load_cancellation_result(
        &self,
        transaction: &Transaction<'_>,
        request: &PersistedRequest,
    ) -> Result<CancellationResult, PostSaleError> {
        load_cancellation(transaction, request)
    }

    fn persist_return(
        &self,
        transaction: &Transaction<'_>,
        request: &PersistedRequest,
        plan: &ReturnPlan,
    ) -> Result<ReturnResult, PostSaleError> {
        transaction
            .execute(
                "INSERT INTO post_sale_requests (request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (?1, 'return', ?2, ?3, ?4, ?5)",
                params![request.request_id, request.sale_id, request.payload_version, request.canonical_payload, request.payload_sha256],
            )
            .map_err(|_| PostSaleError::PersistenceFailure)?;
        let return_id = transaction.last_insert_rowid();
        transaction
            .execute(
                "INSERT INTO sale_returns (id, sale_id) VALUES (?1, ?2)",
                params![return_id, request.sale_id],
            )
            .map_err(|_| PostSaleError::PersistenceFailure)?;
        for line in &plan.lines {
            let stock = transaction
                .query_row(
                    "SELECT quantity FROM stock_balances WHERE product_id = ?1",
                    [line.product_id],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|_| PostSaleError::PersistenceFailure)?;
            let next = stock
                .checked_add(line.quantity)
                .ok_or(PostSaleError::PersistenceFailure)?;
            let updated = transaction
                .execute(
                    "UPDATE stock_balances SET quantity = ?1 WHERE product_id = ?2 AND quantity = ?3",
                    params![next, line.product_id, stock],
                )
                .map_err(|_| PostSaleError::PersistenceFailure)?;
            if updated != 1 {
                return Err(PostSaleError::PersistenceFailure);
            }
            transaction
                .execute(
                    "INSERT INTO inventory_movements (product_id, sale_id, sale_line_id, movement_type, quantity_delta, source_reference) VALUES (?1, ?2, ?3, 'return', ?4, ?5)",
                    params![line.product_id, request.sale_id, line.sale_line_id, line.quantity, format!("return:{return_id}:{}", line.sale_line_id)],
                )
                .map_err(|_| PostSaleError::PersistenceFailure)?;
            let movement_id = transaction.last_insert_rowid();
            transaction
                .execute(
                    "INSERT INTO sale_return_lines (return_id, sale_id, sale_line_id, product_id, quantity, movement_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![return_id, request.sale_id, line.sale_line_id, line.product_id, line.quantity, movement_id],
                )
                .map_err(|_| PostSaleError::PersistenceFailure)?;
        }
        load_return(transaction, request)
    }

    fn persist_cancellation(
        &self,
        transaction: &Transaction<'_>,
        request: &PersistedRequest,
        plan: &CancellationPlan,
    ) -> Result<CancellationResult, PostSaleError> {
        transaction
            .execute(
                "INSERT INTO post_sale_requests (request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (?1, 'cancellation', ?2, ?3, ?4, ?5)",
                params![request.request_id, request.sale_id, request.payload_version, request.canonical_payload, request.payload_sha256],
            )
            .map_err(|_| PostSaleError::PersistenceFailure)?;
        let cancellation_id = transaction.last_insert_rowid();
        transaction
            .execute(
                "INSERT INTO sale_cancellations (id, sale_id, reason) VALUES (?1, ?2, ?3)",
                params![cancellation_id, request.sale_id, plan.reason],
            )
            .map_err(|_| PostSaleError::PersistenceFailure)?;
        for line in &plan.lines {
            let movement_id = if line.restored_quantity == 0 {
                None
            } else {
                let stock = transaction
                    .query_row(
                        "SELECT quantity FROM stock_balances WHERE product_id = ?1",
                        [line.product_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|_| PostSaleError::PersistenceFailure)?;
                let next = stock
                    .checked_add(line.restored_quantity)
                    .ok_or(PostSaleError::PersistenceFailure)?;
                if transaction
                    .execute(
                        "UPDATE stock_balances SET quantity = ?1 WHERE product_id = ?2 AND quantity = ?3",
                        params![next, line.product_id, stock],
                    )
                    .map_err(|_| PostSaleError::PersistenceFailure)?
                    != 1
                {
                    return Err(PostSaleError::PersistenceFailure);
                }
                transaction
                    .execute(
                        "INSERT INTO inventory_movements (product_id, sale_id, sale_line_id, movement_type, quantity_delta, reason, source_reference) VALUES (?1, ?2, ?3, 'cancellation', ?4, ?5, ?6)",
                        params![line.product_id, request.sale_id, line.sale_line_id, line.restored_quantity, plan.reason, format!("cancellation:{cancellation_id}:{}", line.sale_line_id)],
                    )
                    .map_err(|_| PostSaleError::PersistenceFailure)?;
                Some(transaction.last_insert_rowid())
            };
            transaction
                .execute(
                    "INSERT INTO sale_cancellation_lines (cancellation_id, sale_id, sale_line_id, product_id, restored_quantity, movement_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![cancellation_id, request.sale_id, line.sale_line_id, line.product_id, line.restored_quantity, movement_id],
                )
                .map_err(|_| PostSaleError::PersistenceFailure)?;
        }
        load_cancellation(transaction, request)
    }
}

fn load_return(
    transaction: &Transaction<'_>,
    request: &PersistedRequest,
) -> Result<ReturnResult, PostSaleError> {
    let (return_id, sale_id, occurred_at) = transaction
        .query_row(
            "SELECT r.id, r.sale_id, r.occurred_at FROM sale_returns r JOIN post_sale_requests p ON p.id = r.id WHERE p.request_id = ?1",
            [&request.request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| PostSaleError::PersistenceFailure)?;
    let mut statement = transaction
        .prepare(
            "SELECT sale_line_id, product_id, quantity FROM sale_return_lines WHERE return_id = ?1 ORDER BY sale_line_id",
        )
        .map_err(|_| PostSaleError::PersistenceFailure)?;
    let lines = statement
        .query_map([return_id], |row| {
            Ok(ReturnPlanLine {
                sale_line_id: row.get(0)?,
                product_id: row.get(1)?,
                quantity: row.get(2)?,
            })
        })
        .map_err(|_| PostSaleError::PersistenceFailure)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| PostSaleError::PersistenceFailure)?;
    Ok(ReturnResult {
        request_id: request.request_id.clone(),
        return_id,
        sale_id,
        status: SaleLifecycleStatus::Confirmed,
        occurred_at,
        lines,
    })
}

fn load_cancellation(
    transaction: &Transaction<'_>,
    request: &PersistedRequest,
) -> Result<CancellationResult, PostSaleError> {
    let (cancellation_id, sale_id, occurred_at, reason) = transaction
        .query_row(
            "SELECT c.id, c.sale_id, c.occurred_at, c.reason FROM sale_cancellations c JOIN post_sale_requests p ON p.id = c.id WHERE p.request_id = ?1",
            [&request.request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|_| PostSaleError::PersistenceFailure)?;
    let mut statement = transaction
        .prepare(
            "SELECT sale_line_id, product_id, restored_quantity FROM sale_cancellation_lines WHERE cancellation_id = ?1 ORDER BY sale_line_id",
        )
        .map_err(|_| PostSaleError::PersistenceFailure)?;
    let lines = statement
        .query_map([cancellation_id], |row| {
            Ok(crate::domain::sales::CancellationPlanLine {
                sale_line_id: row.get(0)?,
                product_id: row.get(1)?,
                restored_quantity: row.get(2)?,
            })
        })
        .map_err(|_| PostSaleError::PersistenceFailure)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| PostSaleError::PersistenceFailure)?;
    Ok(CancellationResult {
        request_id: request.request_id.clone(),
        cancellation_id,
        sale_id,
        status: SaleLifecycleStatus::Cancelled,
        occurred_at,
        reason,
        lines,
    })
}
