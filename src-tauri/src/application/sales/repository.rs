use rusqlite::Transaction;

use super::{PersistedSaleSummary, RequestedLine};
use crate::domain::sales::SaleLine;

pub trait SaleRepository {
    fn reserve_request_id(
        &self,
        transaction: &Transaction,
        request_id: &str,
    ) -> Result<bool, String>;

    fn current_line(
        &self,
        transaction: &Transaction,
        line: RequestedLine,
    ) -> Result<SaleLine, String>;

    fn load_summary(
        &self,
        transaction: &Transaction,
        request_id: &str,
    ) -> Result<PersistedSaleSummary, String>;
}
