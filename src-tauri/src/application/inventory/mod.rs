mod repository;

pub use repository::InventoryRepository;

use crate::domain::inventory::{
    InventoryAlert, InventoryError, InventoryOperation, PersistedInventoryOperation,
};
use crate::domain::RequestId;

pub fn confirm_stock_entry<R: InventoryRepository>(
    repository: &mut R,
    product_id: i64,
    request_id: RequestId,
    quantity: i64,
    note: Option<String>,
) -> Result<PersistedInventoryOperation, InventoryError> {
    repository.confirm(InventoryOperation::stock_entry(
        product_id, request_id, quantity, note,
    )?)
}

pub fn confirm_physical_count<R: InventoryRepository>(
    repository: &mut R,
    product_id: i64,
    request_id: RequestId,
    count: i64,
    reason: &str,
) -> Result<PersistedInventoryOperation, InventoryError> {
    repository.confirm(InventoryOperation::physical_count(
        product_id, request_id, count, reason,
    )?)
}

pub fn list_inventory_alerts<R: InventoryRepository>(
    repository: &R,
) -> Result<Vec<InventoryAlert>, InventoryError> {
    let mut alerts = repository.list_alerts()?;
    alerts.sort_by_key(InventoryAlert::sort_key);
    Ok(alerts)
}
