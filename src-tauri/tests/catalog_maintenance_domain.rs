use repuestos_autos::domain::catalog::{
    has_normalized_collision, plan_transition, validate_maintenance_product, AttributeDefinition,
    AttributeValueDraft, CatalogActivity, CatalogIntent, CatalogSnapshot, CatalogTarget, FieldType,
    MaintenanceError, TransitionPlan,
};

fn product_snapshot(category_activity: CatalogActivity) -> CatalogSnapshot {
    CatalogSnapshot {
        target: CatalogTarget::Product,
        activity: CatalogActivity::Active,
        category_activity,
        active_products: 0,
        values_valid: true,
        revision: 4,
    }
}

#[test]
fn normalized_identity_collisions_are_rejected_before_persistence() {
    assert!(has_normalized_collision("  FLT-001 ", "flt-001"));
}

#[test]
fn maintenance_product_metadata_rejects_non_positive_centavos() {
    assert_eq!(
        validate_maintenance_product("FLT-001", "Oil filter", 0, &[], &[]),
        Err(MaintenanceError::InvalidCatalogPrice)
    );
}

#[test]
fn maintenance_product_metadata_rejects_mistyped_values() {
    assert_eq!(
        validate_maintenance_product(
            "FLT-001",
            "Oil filter",
            2_500,
            &[AttributeDefinition {
                id: 1,
                field_type: FieldType::Number,
                required: true,
                options: vec![],
            }],
            &[AttributeValueDraft {
                definition_id: 1,
                value: "wrong type".into(),
            }],
        ),
        Err(MaintenanceError::InvalidAttributeValue)
    );
}

#[test]
fn lifecycle_transitions_preserve_independent_category_and_product_state() {
    let category = CatalogSnapshot {
        target: CatalogTarget::Category,
        activity: CatalogActivity::Active,
        category_activity: CatalogActivity::Active,
        active_products: 1,
        values_valid: true,
        revision: 2,
    };
    assert_eq!(
        plan_transition(&category, CatalogIntent::Archive),
        Err(MaintenanceError::LifecycleBlocked)
    );
    assert_eq!(
        plan_transition(
            &product_snapshot(CatalogActivity::Archived),
            CatalogIntent::Reactivate,
        ),
        Err(MaintenanceError::LifecycleBlocked)
    );
    assert_eq!(
        plan_transition(
            &product_snapshot(CatalogActivity::Active),
            CatalogIntent::Archive
        ),
        Ok(TransitionPlan {
            activity: CatalogActivity::Archived,
            expected_revision: 4,
        })
    );
}
