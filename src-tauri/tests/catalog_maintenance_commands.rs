use repuestos_autos::commands::catalog::{
    catalog_metadata_detail, edit_catalog, list_catalog_maintenance, maintain_catalog,
    map_command_state_error, CatalogMaintenanceListResponse, CatalogMaintenanceRecord,
    CatalogMaintenanceResponse, CatalogMetadataDetailRequest, CatalogMetadataDetailResponse,
    EditCatalogRequest, MaintainCatalogRequest,
};
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;

#[test]
fn maintenance_command_returns_tagged_outcomes_without_sql_details() {
    let mut connection = open_seeded_catalog().unwrap();
    let stale = maintain_catalog(
        &mut connection,
        MaintainCatalogRequest {
            target: "product".into(),
            entity_id: 1,
            intent: "archive".into(),
            expected_revision: 1,
        },
    )
    .unwrap();
    assert!(matches!(stale, CatalogMaintenanceResponse::Error(_)));
    let success = maintain_catalog(
        &mut connection,
        MaintainCatalogRequest {
            target: "product".into(),
            entity_id: 1,
            intent: "archive".into(),
            expected_revision: 0,
        },
    )
    .unwrap();
    assert!(
        matches!(success, CatalogMaintenanceResponse::Success(record) if record.activity == "archived")
    );
    let listed = list_catalog_maintenance(&connection).unwrap();
    assert!(
        matches!(listed, CatalogMaintenanceListResponse::Success { ref records } if records.iter().any(|record| record.activity == "archived"))
    );
    assert!(!serde_json::to_string(&listed).unwrap().contains("sqlite"));
}

#[test]
fn maintenance_request_rejects_unknown_fields() {
    assert!(serde_json::from_str::<MaintainCatalogRequest>(r#"{"target":"product","entity_id":1,"intent":"archive","expected_revision":0,"sql":"details"}"#).is_err());
}

#[test]
fn typed_metadata_commands_deny_unknown_fields_and_project_stable_outcomes() {
    let mut connection = open_seeded_catalog().unwrap();
    let invalid = r#"{"target":"product","entity_id":1,"expected_revision":0,"sku":"NEW-1","name":"New","catalog_unit_price_centavos":3000,"attribute_values":[{"definition_id":1,"value":"x","sql":"details"}]}"#;
    assert!(serde_json::from_str::<EditCatalogRequest>(invalid).is_err());
    assert!(serde_json::from_str::<EditCatalogRequest>(r#"{"target":"category","entity_id":1,"expected_revision":0,"name":"Filters","unexpected":true}"#).is_err());

    let edited = edit_catalog(
        &mut connection,
        EditCatalogRequest::Category {
            entity_id: 1,
            expected_revision: 0,
            name: "Filters and oils".into(),
        },
    )
    .unwrap();
    assert!(matches!(
        edited,
        CatalogMaintenanceResponse::Success(CatalogMaintenanceRecord { revision: 1, .. })
    ));
    let stale = edit_catalog(
        &mut connection,
        EditCatalogRequest::Category {
            entity_id: 1,
            expected_revision: 0,
            name: "Other".into(),
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(stale).unwrap()["code"],
        "stale_catalog_record"
    );
    let category = catalog_metadata_detail(
        &connection,
        CatalogMetadataDetailRequest {
            target: "category".into(),
            entity_id: 1,
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(category).unwrap()["name"],
        "Filters and oils"
    );
    connection.execute_batch("INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (1, 1, 'Material', 'text', 1); INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 1, 'Paper', 'Paper');").unwrap();
    maintain_catalog(
        &mut connection,
        MaintainCatalogRequest {
            target: "product".into(),
            entity_id: 1,
            intent: "archive".into(),
            expected_revision: 0,
        },
    )
    .unwrap();

    let detail = catalog_metadata_detail(
        &connection,
        CatalogMetadataDetailRequest {
            target: "product".into(),
            entity_id: 1,
        },
    )
    .unwrap();
    let detail_json = serde_json::to_value(&detail).unwrap();
    assert!(matches!(detail, CatalogMetadataDetailResponse::Success(_)));
    assert_eq!(detail_json["target"], "product");
    assert_eq!(detail_json["sku"], "FLT-001");
    assert_eq!(detail_json["catalog_unit_price_centavos"], 2_500);
    assert_eq!(detail_json["revision"], 1);
    assert_eq!(detail_json["activity"], "archived");
    assert_eq!(detail_json["attribute_definitions"][0]["label"], "Material");
    assert_eq!(detail_json["attribute_values"][0]["value"], "Paper");
    let missing = catalog_metadata_detail(
        &connection,
        CatalogMetadataDetailRequest {
            target: "product".into(),
            entity_id: 99,
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(missing).unwrap()["code"],
        "catalog_unavailable"
    );
    assert_eq!(
        map_command_state_error("database_unavailable").code,
        "catalog_unavailable"
    );
    let failure = catalog_metadata_detail(
        &rusqlite::Connection::open_in_memory().unwrap(),
        CatalogMetadataDetailRequest {
            target: "product".into(),
            entity_id: 1,
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&failure).unwrap()["code"],
        "persistence_failure"
    );
    assert!(!serde_json::to_string(&failure)
        .unwrap()
        .contains("no such table"));
    assert!(!serde_json::to_string(&detail).unwrap().contains("sqlite"));
}
