use repuestos_autos::application::catalog::{replace_product_image, ProductImage};
use repuestos_autos::commands::catalog::{
    catalog_product_image_thumbnail, parse_product_image_request, ProductImageRequest,
    ProductImageThumbnailResponse,
    catalog_metadata_detail, edit_catalog, edit_category_schema, list_catalog_categories, list_catalog_maintenance, maintain_catalog,
    map_command_state_error, CatalogMaintenanceListResponse, CatalogMaintenanceRecord,
    CatalogMaintenanceResponse, CatalogMetadataDetailRequest, CatalogMetadataDetailResponse,
    EditCatalogRequest, EditCategorySchemaRequest, EditCategorySchemaFieldRequest, MaintainCatalogRequest,
};
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;

#[test]
fn sku_conflicts_are_global_typed_and_leave_metadata_search_and_audit_unchanged() {
    let mut connection = open_seeded_catalog().unwrap();
    connection.execute_batch("INSERT INTO products (id, sku, name, category_id, active, list_price_centavos, minimum_unit_price_centavos) VALUES (91, '  OtHeR  ', 'Shared name', 2, 0, 2500, 2500);
        INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (91, 1, 'Brand', 'text', 1);
        INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 91, 'Original', 'Original');").unwrap();
    let request = |sku: &str, name: &str, revision| serde_json::from_value::<EditCatalogRequest>(serde_json::json!({
        "target": "product", "entity_id": 1, "expected_revision": revision, "expected_category_revision": 0,
        "sku": sku, "name": name, "purchase_price_centavos": 2000, "sale_price_centavos": 2500,
        "minimum_sale_price_centavos": 2500, "attribute_values": [{"definition_id":91,"value":"Changed"}]
    })).unwrap();
    let before: String = connection.query_row("SELECT content FROM catalog_product_search WHERE rowid = 1", [], |r| r.get(0)).unwrap();
    for sku in ["OTHER", " other ", "OtHeR"] {
        let response = edit_catalog(&mut connection, request(sku, "Shared name", 0)).unwrap();
        assert_eq!(serde_json::to_value(response).unwrap()["code"], "duplicate_sku");
    }
    assert_eq!(serde_json::to_value(edit_catalog(&mut connection, request("FLT-001", "   ", 0)).unwrap()).unwrap()["code"], "validation_error");
    assert_eq!(serde_json::to_value(edit_catalog(&mut connection, request("OTHER", "Shared name", 1)).unwrap()).unwrap()["code"], "stale_catalog_record");
    assert_eq!(connection.query_row("SELECT sku, revision FROM products WHERE id = 1", [], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))).unwrap(), ("FLT-001".into(), 0));
    assert_eq!(connection.query_row("SELECT text_value FROM product_attribute_values WHERE product_id = 1 AND definition_id = 91", [], |r| r.get::<_, String>(0)).unwrap(), "Original");
    assert_eq!(connection.query_row("SELECT content FROM catalog_product_search WHERE rowid = 1", [], |r| r.get::<_, String>(0)).unwrap(), before);
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM catalog_audit", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
}

#[test]
fn sales_original_returns_unchanged_large_original_without_catalog_metadata() {
    use repuestos_autos::commands::catalog::{sales_product_image_original, SalesProductOriginalRequest};
    let mut connection = open_seeded_catalog().unwrap();
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(640, 480, image::Rgb([20, 40, 60])))
        .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png).unwrap();
    let original = ProductImage::new("image/png", bytes.clone()).unwrap();
    replace_product_image(&mut connection, 1, 0, &original).unwrap();
    let response = serde_json::to_value(sales_product_image_original(&connection, SalesProductOriginalRequest { product_id: 1 })).unwrap();
    assert_eq!(response["mime_type"], "image/png");
    assert_eq!(response["encoding"], "base64");
    assert_eq!(response["product_id"], 1);
    assert_eq!(response.as_object().unwrap().len(), 5);
    // Decode transport bytes independently to protect byte identity, not just the MIME.
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut decoded = Vec::new();
    for chunk in response["bytes"].as_str().unwrap().as_bytes().chunks(4) {
        let digits: Vec<u32> = chunk.iter().map(|byte| alphabet.find(*byte as char).unwrap_or(0) as u32).collect();
        let value = digits[0] << 18 | digits[1] << 12 | digits[2] << 6 | digits[3];
        decoded.push((value >> 16) as u8);
        if chunk[2] != b'=' { decoded.push((value >> 8) as u8); }
        if chunk[3] != b'=' { decoded.push(value as u8); }
    }
    assert_eq!(decoded, bytes);
    let thumbnail = repuestos_autos::application::catalog::read_product_image_thumbnail(&connection, 1).unwrap().unwrap();
    let thumbnail_image = image::load_from_memory(&thumbnail.bytes).unwrap();
    assert_eq!((thumbnail_image.width(), thumbnail_image.height()), (256, 192));
    assert_eq!(thumbnail.mime_type, "image/jpeg");
    assert_ne!(thumbnail.bytes, decoded);
}

#[test]
fn sales_original_requests_and_failures_are_bounded() {
    use repuestos_autos::commands::catalog::{sales_product_image_original, SalesProductOriginalRequest};
    for json in [r#"{}"#, r#"{"product_id":1,"revision":0}"#, r#"{"product_id":1,"path":"private"}"#, r#"{"product_id":1.5}"#] {
        assert!(serde_json::from_str::<SalesProductOriginalRequest>(json).is_err());
    }
    let connection = open_seeded_catalog().unwrap();
    for product_id in [-1, 0, 1, 99] {
        assert_eq!(serde_json::to_value(sales_product_image_original(&connection, SalesProductOriginalRequest { product_id })).unwrap(), serde_json::json!({"kind":"unavailable"}));
    }
    let unavailable = rusqlite::Connection::open_in_memory().unwrap();
    assert_eq!(serde_json::to_value(sales_product_image_original(&unavailable, SalesProductOriginalRequest { product_id: 1 })).unwrap(), serde_json::json!({"kind":"error","code":"persistence_failure","message":"The product image could not be loaded."}));
}

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
    let legacy = r#"{"target":"product","entity_id":1,"expected_revision":0,"expected_category_revision":0,"sku":"FLT-001","name":"Filter","purchase_price_centavos":1500,"list_price_centavos":2500,"minimum_sale_price_centavos":2000,"attribute_values":[]}"#;
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

#[test]
fn schema_edit_is_typed_registered_use_case_and_stale_product_saves_are_recoverable() {
    let mut connection = open_seeded_catalog().unwrap();
    let invalid = r#"{"category_id":1,"expected_revision":0,"fields":[{"definition_id":null,"label":"Material","field_type":"text","required":false,"options":[],"sql":"hidden"}]}"#;
    assert!(serde_json::from_str::<EditCategorySchemaRequest>(invalid).is_err());

    let response = edit_category_schema(
        &mut connection,
        EditCategorySchemaRequest {
            category_id: 1,
            expected_revision: 0,
            fields: vec![EditCategorySchemaFieldRequest {
                definition_id: None,
                label: "Material".into(),
                field_type: "text".into(),
                required: false,
                options: vec![],
            }],
        },
    );
    assert!(matches!(response, CatalogMaintenanceResponse::Success(CatalogMaintenanceRecord { revision: 1, .. })));

    let stale_save = edit_catalog(
        &mut connection,
        EditCatalogRequest::Product {
            entity_id: 1,
            expected_revision: 0,
            sku: "FLT-001".into(),
            name: "Oil filter".into(),
            purchase_price_centavos: 2_000,
            sale_price_centavos: 2_500,
            minimum_sale_price_centavos: 2_500,
            low_stock_threshold: None,
            expected_category_revision: 0,
            attribute_values: vec![],
        },
    ).unwrap();
    let error = serde_json::to_value(stale_save).unwrap();
    assert_eq!(error["code"], "stale_category_schema");
    assert!(error["message"].as_str().unwrap().contains("Reload"));
    assert_eq!(connection.query_row("SELECT sku, revision FROM products WHERE id = 1", [], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))).unwrap(), ("FLT-001".into(), 0));
}
