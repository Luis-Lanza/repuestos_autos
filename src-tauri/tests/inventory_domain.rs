use repuestos_autos::domain::inventory::{
    AdjustmentReason, AlertClassification, InventoryAlert, InventoryError, InventoryOperation,
    OperationKind, PersistedInventoryOperation, PhysicalCount, StockEntryQuantity,
};
use repuestos_autos::domain::RequestId;

fn request_id() -> RequestId {
    RequestId::parse("550e8400-e29b-41d4-a716-446655440101").unwrap()
}

fn persisted(
    kind: OperationKind,
    old: i64,
    delta: i64,
    result: i64,
) -> Result<PersistedInventoryOperation, InventoryError> {
    PersistedInventoryOperation::new(kind, request_id(), 1, old, delta, result, "now", None)
}

#[test]
fn domain_value_objects_enforce_inputs_and_persisted_results() {
    assert_eq!(StockEntryQuantity::new(1).unwrap().value(), 1);
    assert_eq!(
        StockEntryQuantity::new(0),
        Err(InventoryError::INVALID_QUANTITY)
    );
    assert_eq!(InventoryError::INVALID_QUANTITY.code(), "invalid_quantity");
    assert_eq!(PhysicalCount::new(0).unwrap().value(), 0);
    assert_eq!(PhysicalCount::new(-1), Err(InventoryError::INVALID_COUNT));
    assert_eq!(
        AdjustmentReason::new("  "),
        Err(InventoryError::REASON_REQUIRED)
    );
    assert_eq!(
        persisted(OperationKind::PhysicalCount, 5, 0, 5),
        Err(InventoryError::UNCHANGED_COUNT)
    );
    assert_eq!(
        persisted(OperationKind::StockEntry, i64::MAX, 1, i64::MAX),
        Err(InventoryError::QUANTITY_OVERFLOW)
    );
}

#[test]
fn inventory_identity_is_stable_and_excludes_request_id() {
    let first = InventoryOperation::stock_entry(
        1,
        RequestId::parse("550e8400-e29b-41d4-a716-446655440101").unwrap(),
        2,
        Some("delivery".into()),
    )
    .unwrap()
    .identity();
    let retry = InventoryOperation::stock_entry(
        1,
        RequestId::parse("550e8400-e29b-41d4-a716-446655440102").unwrap(),
        2,
        Some("delivery".into()),
    )
    .unwrap()
    .identity();

    assert_eq!(first, retry);
    assert_eq!(first.operation_kind(), "stock_entry");
    assert_eq!(first.payload_version(), 1);
    assert_eq!(first.payload_sha256().len(), 64);
}

#[test]
fn inventory_identity_preserves_optional_note_state_and_normalizes_reason() {
    let request_id = request_id();
    let without_note = InventoryOperation::stock_entry(1, request_id.clone(), 2, None)
        .unwrap()
        .identity();
    let empty_note = InventoryOperation::stock_entry(1, request_id.clone(), 2, Some(String::new()))
        .unwrap()
        .identity();
    assert_ne!(without_note, empty_note);

    let padded_reason = InventoryOperation::physical_count(1, request_id.clone(), 2, " counted ")
        .unwrap()
        .identity();
    let normalized_reason = InventoryOperation::physical_count(1, request_id, 2, "counted")
        .unwrap()
        .identity();
    assert_eq!(padded_reason, normalized_reason);
}

#[test]
fn alerts_filter_inactive_and_sufficient_stock_then_order_out_before_low() {
    let mut alerts = [
        InventoryAlert::for_product(2, "Zeta", true, 1).unwrap(),
        InventoryAlert::for_product(1, "alpha", true, 0).unwrap(),
        InventoryAlert::for_product(3, "Beta", true, 0).unwrap(),
    ];
    alerts.sort_by_key(InventoryAlert::sort_key);
    assert_eq!(alerts[0].classification, AlertClassification::OutOfStock);
    assert_eq!(alerts[1].product_id, 3);
    assert_eq!(alerts[2].classification, AlertClassification::LowStock);
    assert!(InventoryAlert::for_product(3, "hidden", false, 0).is_none());
    assert!(InventoryAlert::for_product(4, "enough", true, 2).is_none());
}
