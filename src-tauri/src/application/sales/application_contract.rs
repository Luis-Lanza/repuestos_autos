use std::collections::HashSet;

use rusqlite::{Connection, Transaction};

use crate::domain::sales::{PaymentBreakdown, PaymentInput, Sale};
use crate::domain::{MoneyCentavos, Quantity, RequestId};

use super::{ConfirmSaleError, ConfirmSaleRepository, PersistedSaleSummary, Reservation};

pub struct ApplicationRequestedLine {
    pub product_id: i64,
    pub quantity: Quantity,
}

pub struct ApplicationConfirmSaleRequest {
    pub request_id: RequestId,
    pub lines: Vec<ApplicationRequestedLine>,
    pub payment: PaymentInput,
}

pub struct ConfirmSaleUseCase<'connection, 'repository, R> {
    connection: &'connection mut Connection,
    repository: &'repository R,
}

impl<'connection, 'repository, R: ConfirmSaleRepository>
    ConfirmSaleUseCase<'connection, 'repository, R>
{
    pub fn new(connection: &'connection mut Connection, repository: &'repository R) -> Self {
        Self {
            connection,
            repository,
        }
    }

    pub fn confirm(
        self,
        request: ApplicationConfirmSaleRequest,
    ) -> Result<PersistedSaleSummary, ConfirmSaleError> {
        let repository = self.repository;
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| ConfirmSaleError::Persistence)?;
        match Self::confirm_in_transaction(repository, &transaction, request) {
            Ok(summary) => transaction
                .commit()
                .map(|_| summary)
                .map_err(|_| ConfirmSaleError::Persistence),
            Err(error) => transaction
                .rollback()
                .map(|_| Err(error))
                .map_err(|_| ConfirmSaleError::Persistence)?,
        }
    }

    fn confirm_in_transaction(
        repository: &R,
        transaction: &Transaction<'_>,
        request: ApplicationConfirmSaleRequest,
    ) -> Result<PersistedSaleSummary, ConfirmSaleError> {
        match repository.reserve_or_load(transaction, &request.request_id)? {
            Reservation::ExistingConfirmed(summary) => return Ok(summary),
            Reservation::ExistingIncomplete | Reservation::ExistingCorrupt => {
                return Err(ConfirmSaleError::Persistence);
            }
            Reservation::Reserved => (),
        }
        reject_duplicate_products(&request.lines)?;
        let lines = repository.resolve_lines(transaction, &request.lines)?;
        let total = lines.iter().try_fold(
            MoneyCentavos::new(0).map_err(|_| ConfirmSaleError::PersistedDataInvalid)?,
            |sum, line| {
                sum.checked_add(line.total())
                    .map_err(|_| ConfirmSaleError::MoneyOverflow)
            },
        )?;
        let payments = PaymentBreakdown::derive(total, request.payment)?;
        let sale = Sale::new(lines, payments.payments().to_vec())
            .map_err(|_| ConfirmSaleError::PersistedDataInvalid)?;
        repository.persist_confirmed(transaction, &request.request_id, &sale)
    }
}

fn reject_duplicate_products(lines: &[ApplicationRequestedLine]) -> Result<(), ConfirmSaleError> {
    let mut product_ids = HashSet::with_capacity(lines.len());
    if lines
        .iter()
        .any(|line| !product_ids.insert(line.product_id))
    {
        return Err(ConfirmSaleError::DuplicateProduct);
    }
    Ok(())
}
