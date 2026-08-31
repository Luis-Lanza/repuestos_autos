use rusqlite::{Connection, DropBehavior, Transaction, TransactionBehavior};

use crate::application::sales::{PostSaleError, PostSaleTransaction, PostSaleTransactionFactory};

pub struct SqlitePostSaleTransactionFactory<'connection> {
    connection: &'connection mut Connection,
}

impl<'connection> SqlitePostSaleTransactionFactory<'connection> {
    pub fn new(connection: &'connection mut Connection) -> Self {
        Self { connection }
    }
}

pub struct SqlitePostSaleTransaction<'connection> {
    transaction: Transaction<'connection>,
}

impl PostSaleTransaction for SqlitePostSaleTransaction<'_> {
    fn repository_transaction(&self) -> &Transaction<'_> {
        &self.transaction
    }

    fn commit(&mut self) -> Result<(), PostSaleError> {
        self.transaction
            .execute_batch("COMMIT")
            .map_err(|_| PostSaleError::PersistenceFailure)?;
        self.transaction.set_drop_behavior(DropBehavior::Ignore);
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), PostSaleError> {
        self.transaction
            .execute_batch("ROLLBACK")
            .map_err(|_| PostSaleError::PersistenceFailure)?;
        self.transaction.set_drop_behavior(DropBehavior::Ignore);
        Ok(())
    }
}

impl PostSaleTransactionFactory for SqlitePostSaleTransactionFactory<'_> {
    type Transaction<'transaction>
        = SqlitePostSaleTransaction<'transaction>
    where
        Self: 'transaction;

    fn begin_immediate(&mut self) -> Result<Self::Transaction<'_>, PostSaleError> {
        self.connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map(|transaction| SqlitePostSaleTransaction { transaction })
            .map_err(|_| PostSaleError::PersistenceFailure)
    }
}
