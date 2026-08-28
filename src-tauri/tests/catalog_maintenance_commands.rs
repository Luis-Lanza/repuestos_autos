use repuestos_autos::commands::catalog::{
    list_catalog_maintenance, maintain_catalog, CatalogMaintenanceListResponse,
    CatalogMaintenanceResponse, MaintainCatalogRequest,
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
