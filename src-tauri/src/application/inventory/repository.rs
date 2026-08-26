use crate::domain::inventory::{
    InventoryAlert, InventoryError, InventoryOperation, PersistedInventoryOperation,
};

pub trait InventoryRepository {
    fn confirm(
        &mut self,
        operation: InventoryOperation,
    ) -> Result<PersistedInventoryOperation, InventoryError>;
    fn list_alerts(&self) -> Result<Vec<InventoryAlert>, InventoryError>;
}
