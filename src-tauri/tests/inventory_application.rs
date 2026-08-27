use repuestos_autos::application::inventory::{
    confirm_physical_count, list_inventory_alerts, InventoryRepository,
};
use repuestos_autos::domain::inventory::{
    AlertClassification, InventoryAlert, InventoryError, InventoryOperation, OperationKind,
    PersistedInventoryOperation,
};
use repuestos_autos::domain::RequestId;

struct FakeRepository {
    stored: Option<PersistedInventoryOperation>,
    writes: usize,
    alerts: Vec<InventoryAlert>,
}

impl InventoryRepository for FakeRepository {
    fn confirm(
        &mut self,
        operation: InventoryOperation,
    ) -> Result<PersistedInventoryOperation, InventoryError> {
        if let Some(result) = self.stored.clone() {
            return Ok(result);
        }
        self.writes += 1;
        let (kind, request_id) = match operation {
            InventoryOperation::StockEntry { request_id, .. } => {
                (OperationKind::StockEntry, request_id)
            }
            InventoryOperation::PhysicalCount { request_id, .. } => {
                (OperationKind::PhysicalCount, request_id)
            }
        };
        let result =
            PersistedInventoryOperation::new(kind, request_id, 1, 9, -5, 4, "database-time", None)?;
        self.stored = Some(result.clone());
        Ok(result)
    }

    fn list_alerts(&self) -> Result<Vec<InventoryAlert>, InventoryError> {
        Ok(self.alerts.clone())
    }
}

fn request_id() -> RequestId {
    RequestId::parse("550e8400-e29b-41d4-a716-446655440102").unwrap()
}

fn repository() -> FakeRepository {
    FakeRepository {
        stored: None,
        writes: 0,
        alerts: vec![],
    }
}

#[test]
fn physical_count_uses_authoritative_persisted_balance_and_retries_idempotently() {
    let mut repository = repository();
    let first = confirm_physical_count(&mut repository, 1, request_id(), 4, "counted").unwrap();
    let retry = confirm_physical_count(&mut repository, 1, request_id(), 4, "counted").unwrap();
    assert_eq!(first, retry);
    assert_eq!(first.kind, OperationKind::PhysicalCount);
    assert_eq!(first.previous_quantity, 9);
    assert_eq!(repository.writes, 1);
}

#[test]
fn alert_listing_returns_deterministically_ordered_tagged_alerts() {
    let mut repository = repository();
    repository.alerts = vec![
        InventoryAlert::for_product(2, "Zeta", true, 1).unwrap(),
        InventoryAlert::for_product(1, "alpha", true, 0).unwrap(),
    ];
    let alerts = list_inventory_alerts(&repository).unwrap();
    assert_eq!(alerts[0].classification, AlertClassification::OutOfStock);
}
