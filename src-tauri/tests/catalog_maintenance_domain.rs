use repuestos_autos::domain::catalog::{
    has_normalized_collision, plan_category_schema_edit, plan_transition,
    validate_maintenance_category, validate_maintenance_product, AttributeDefinition,
    AttributeValueDraft, CatalogActivity, CatalogIntent, CatalogSnapshot, CatalogTarget,
    CategoryFieldLifecycle, CategorySchemaField, ExistingCategorySchemaField, FieldType,
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
fn maintenance_category_metadata_rejects_blank_names() {
    assert_eq!(
        validate_maintenance_category("  "),
        Err(MaintenanceError::InvalidProduct)
    );
}

#[test]
fn maintenance_product_metadata_rejects_non_positive_centavos() {
    assert_eq!(
        validate_maintenance_product("FLT-001", "Oil filter", 1, 0, 100, &[], &[]),
        Err(MaintenanceError::InvalidSalePrice)
    );
}

#[test]
fn maintenance_product_metadata_rejects_minimum_above_list() {
    assert_eq!(
        validate_maintenance_product("FLT-001", "Oil filter", 1_000, 2_500, 2_501, &[], &[]),
        Err(MaintenanceError::MinimumSalePriceExceedsSalePrice)
    );
}

#[test]
fn maintenance_product_metadata_rejects_non_positive_minimum() {
    assert_eq!(
        validate_maintenance_product("FLT-001", "Oil filter", 1_000, 2_500, 0, &[], &[]),
        Err(MaintenanceError::InvalidMinimumSalePrice)
    );
}

#[test]
fn maintenance_product_metadata_rejects_prices_above_the_safe_integer_cap() {
    const CAP: i64 = 9_007_199_254_740_991;

    assert_eq!(
        validate_maintenance_product("FLT-001", "Oil filter", CAP + 1, CAP, 1, &[], &[]),
        Err(MaintenanceError::InvalidPurchasePrice)
    );
    assert_eq!(
        validate_maintenance_product("FLT-001", "Oil filter", 1, CAP + 1, CAP, &[], &[]),
        Err(MaintenanceError::InvalidSalePrice)
    );
    assert!(validate_maintenance_product("FLT-001", "Oil filter", 1, CAP, CAP, &[], &[]).is_ok());
}

#[test]
fn maintenance_product_metadata_rejects_mistyped_values() {
    assert_eq!(
        validate_maintenance_product(
            "FLT-001",
            "Oil filter",
            1_000,
            2_500,
            2_500,
            &[AttributeDefinition {
                id: 1,
                field_type: FieldType::Number,
                required: true,
                options: vec![]
            }],
            &[AttributeValueDraft {
                definition_id: 1,
                value: "wrong type".into()
            }],
        ),
        Err(MaintenanceError::InvalidAttributeValue)
    );
}

#[test]
fn category_schema_plan_adds_fields_retires_omitted_ids_and_preserves_stable_definitions() {
    let existing = [ExistingCategorySchemaField {
        definition_id: 41,
        label: "Material".into(),
        field_type: FieldType::Option,
        required: false,
        options: vec!["Rubber".into(), "Steel".into()],
        lifecycle: CategoryFieldLifecycle::Active,
    }];
    let planned = plan_category_schema_edit(
        &existing,
        &[
            CategorySchemaField {
                definition_id: Some(41),
                label: "Material".into(),
                field_type: FieldType::Option,
                required: false,
                options: vec!["Rubber".into(), "Steel".into()],
            },
            CategorySchemaField {
                definition_id: None,
                label: "Length".into(),
                field_type: FieldType::Number,
                required: true,
                options: vec![],
            },
        ],
    )
    .unwrap();
    assert_eq!(planned.retire_definition_ids, vec![]);
    assert_eq!(planned.additions.len(), 1);
    assert_eq!(planned.additions[0].label, "Length");

    let retired = plan_category_schema_edit(&existing, &[]).unwrap();
    assert_eq!(retired.retire_definition_ids, vec![41]);
}

#[test]
fn category_schema_plan_allows_readding_a_retired_label() {
    let existing = [
        ExistingCategorySchemaField {
            definition_id: 41,
            label: "Material".into(),
            field_type: FieldType::Text,
            required: false,
            options: vec![],
            lifecycle: CategoryFieldLifecycle::Retired,
        },
        ExistingCategorySchemaField {
            definition_id: 42,
            label: "Length".into(),
            field_type: FieldType::Text,
            required: false,
            options: vec![],
            lifecycle: CategoryFieldLifecycle::Active,
        },
    ];
    let plan = plan_category_schema_edit(
        &existing,
        &[CategorySchemaField {
            definition_id: None,
            label: "Material".into(),
            field_type: FieldType::Text,
            required: false,
            options: vec![],
        }],
    )
    .unwrap();

    assert_eq!(plan.retire_definition_ids, vec![42]);
    assert_eq!(plan.additions[0].label, "Material");
}

#[test]
fn category_schema_plan_can_retire_and_replace_a_field_with_the_same_label() {
    let existing = [ExistingCategorySchemaField {
        definition_id: 41,
        label: "Material".into(),
        field_type: FieldType::Text,
        required: false,
        options: vec![],
        lifecycle: CategoryFieldLifecycle::Active,
    }];

    let plan = plan_category_schema_edit(
        &existing,
        &[CategorySchemaField {
            definition_id: None,
            label: "Material".into(),
            field_type: FieldType::Option,
            required: true,
            options: vec!["Steel".into()],
        }],
    )
    .unwrap();

    assert_eq!(plan.retire_definition_ids, vec![41]);
    assert_eq!(plan.additions.len(), 1);
    assert_eq!(plan.additions[0].label, "Material");
}

#[test]
fn category_schema_plan_rejects_mutating_existing_field_definitions() {
    let existing = [ExistingCategorySchemaField {
        definition_id: 41,
        label: "Material".into(),
        field_type: FieldType::Option,
        required: false,
        options: vec!["Rubber".into()],
        lifecycle: CategoryFieldLifecycle::Active,
    }];
    let changed = CategorySchemaField {
        definition_id: Some(41),
        label: "Material".into(),
        field_type: FieldType::Option,
        required: false,
        options: vec!["Rubber".into(), "Steel".into()],
    };
    assert_eq!(
        plan_category_schema_edit(&existing, &[changed]),
        Err(MaintenanceError::ImmutableCategoryField)
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
            expected_revision: 4
        })
    );
}
