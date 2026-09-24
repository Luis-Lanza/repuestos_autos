use repuestos_autos::application::catalog::{replace_product_image, ProductImage};
use repuestos_autos::commands::catalog::{
    catalog_product_image_thumbnail, parse_product_image_request, ProductImageRequest,
    ProductImageThumbnailResponse,
    catalog_metadata_detail, edit_catalog, list_catalog_categories, list_catalog_maintenance, maintain_catalog,
    map_command_state_error, CatalogMaintenanceListResponse, CatalogMaintenanceRecord,
    CatalogMaintenanceResponse, CatalogMetadataDetailRequest, CatalogMetadataDetailResponse,
    EditCatalogRequest, MaintainCatalogRequest,
};
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;

#[test]
fn image_requests_are_strict_and_image_responses_never_expose_paths() {
    assert!(serde_json::from_str::<ProductImageRequest>(r#"{"product_id":1,"expected_revision":0,"path":"/secret"}"#).is_err());
    assert!(serde_json::from_str::<ProductImageRequest>(r#"{"product_id":1,"expected_revision":-1}"#).is_ok());
    assert!(parse_product_image_request(ProductImageRequest { product_id: 0, expected_revision: 0 }).is_err());

    let mut connection = open_seeded_catalog().unwrap();
    let image = ProductImage::new("image/png", generated_png()).unwrap();
    replace_product_image(&mut connection, 1, 0, &image).unwrap();
    let response = catalog_product_image_thumbnail(&connection, ProductImageRequest { product_id: 1, expected_revision: 1 });
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(!serialized.contains("path"));
    assert!(!serialized.contains("sqlite"));
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["kind"], "success");
    assert_eq!(json["product_id"], 1);
    assert_eq!(json["revision"], 1);
    assert_eq!(json["mime_type"], "image/jpeg");
    assert_eq!(json["encoding"], "base64");
    assert!(json["bytes"].as_str().unwrap().starts_with("/9j/"));
    assert!(matches!(response, ProductImageThumbnailResponse::Success { .. }));
    let stale = catalog_product_image_thumbnail(&connection, ProductImageRequest { product_id: 1, expected_revision: 0 });
    assert_eq!(serde_json::to_value(stale).unwrap()["code"], "stale_catalog_record");
}

#[test]
fn missing_product_thumbnail_is_distinct_from_an_unavailable_product() {
    let connection = open_seeded_catalog().unwrap();
    let no_image = catalog_product_image_thumbnail(&connection, ProductImageRequest { product_id: 1, expected_revision: 0 });
    assert_eq!(serde_json::to_value(no_image).unwrap()["code"], "image_unavailable");
    let stale_no_image = catalog_product_image_thumbnail(&connection, ProductImageRequest { product_id: 1, expected_revision: 99 });
    assert_eq!(serde_json::to_value(stale_no_image).unwrap()["code"], "stale_catalog_record");

    let missing_product = catalog_product_image_thumbnail(&connection, ProductImageRequest { product_id: 99, expected_revision: 0 });
    assert_eq!(serde_json::to_value(missing_product).unwrap()["code"], "catalog_unavailable");
}

fn generated_png() -> Vec<u8> {
    use image::ImageEncoder;
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(&[255; 16], 2, 2, image::ExtendedColorType::Rgba8)
        .unwrap();
    bytes
}

#[test]
fn category_list_command_serializes_the_authoritative_active_product_count() {
    let connection = open_seeded_catalog().unwrap();
    let response = list_catalog_categories(&connection).unwrap();
    let value = serde_json::to_value(response).unwrap();
    assert_eq!(value["kind"], "success");
    let records = value["records"].as_array().unwrap();
    let category = records.iter().find(|record| record["entity_id"] == 1).unwrap();
    assert_eq!(category["active_product_count"], 1);
    assert_eq!(records.iter().find(|record| record["entity_id"] == 2).unwrap()["active_product_count"], 0);

    connection.execute("UPDATE products SET active = 0 WHERE category_id = 1", []).unwrap();
    let response = list_catalog_categories(&connection).unwrap();
    let value = serde_json::to_value(response).unwrap();
    let category = value["records"].as_array().unwrap().iter().find(|record| record["entity_id"] == 1).unwrap();
    assert_eq!(category["active_product_count"], 0);
}

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
fn edit_requests_accept_legacy_list_price_alias_but_emit_sale_price_terminology() {
    let legacy = r#"{"target":"product","entity_id":1,"expected_revision":0,"sku":"FLT-001","name":"Filter","purchase_price_centavos":1500,"list_price_centavos":2500,"minimum_sale_price_centavos":2000,"attribute_values":[]}"#;
    let request: EditCatalogRequest = serde_json::from_str(legacy).unwrap();
    let mut connection = open_seeded_catalog().unwrap();
    let response = edit_catalog(&mut connection, request).unwrap();
    assert!(matches!(response, CatalogMaintenanceResponse::Success(_)));
    let detail = catalog_metadata_detail(&connection, CatalogMetadataDetailRequest { target: "product".into(), entity_id: 1 }).unwrap();
    let json = serde_json::to_value(detail).unwrap();
    assert_eq!(json["sale_price_centavos"], 2500);
    assert_eq!(json["purchase_price_centavos"], 1500);
    assert!(json.get("list_price_centavos").is_none());
}

#[test]
fn typed_metadata_commands_deny_unknown_fields_and_project_stable_outcomes() {
    let mut connection = open_seeded_catalog().unwrap();
    let invalid = r#"{"target":"product","entity_id":1,"expected_revision":0,"sku":"NEW-1","name":"New","purchase_price_centavos":2000,"sale_price_centavos":4000,"minimum_sale_price_centavos":3000,"attribute_values":[{"definition_id":1,"value":"x","sql":"details"}]}"#;
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
    assert_eq!(detail_json["purchase_price_centavos"], serde_json::Value::Null);
    assert_eq!(detail_json["sale_price_centavos"], 2_500);
    assert!(detail_json.get("list_price_centavos").is_none());
    assert_eq!(detail_json["minimum_sale_price_centavos"], 2_500);
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
