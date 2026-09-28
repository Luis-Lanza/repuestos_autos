use repuestos_autos::application::catalog::locations::{
    assign_product_location, create_product_location, delete_product_location, list_product_locations,
    save_location_schema, set_location_activity, CreateProductLocationInput,
    SaveLocationSchemaInput,
};
use repuestos_autos::commands::catalog::{
    assign_product_primary_location, AssignProductLocationRequest, CreateProductLocationRequest,
    ProductLocationResponse, SaveLocationSchemaRequest,
};
use repuestos_autos::domain::catalog::location::LocationValidationError;
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;

fn configure(connection: &mut rusqlite::Connection, segments: &[&str]) {
    save_location_schema(connection, SaveLocationSchemaInput {
        expected_revision: 0,
        segments: segments.iter().map(|segment| (*segment).to_owned()).collect(),
    }).unwrap();
}

#[test]
fn location_nomenclature_is_global_ordered_and_locked_against_reordering_or_removal() {
    let mut connection = open_seeded_catalog().unwrap();
    configure(&mut connection, &["Zone", "Shelf"]);
    let created = create_product_location(&mut connection, CreateProductLocationInput {
        values: vec!["A-1".into(), "Shelf 2".into()],
    }).unwrap();
    assert_eq!(created.code, "A1-SHELF2");
    assert_eq!(created.values, ["A-1", "Shelf 2"]);

    let reorder = save_location_schema(&mut connection, SaveLocationSchemaInput {
        expected_revision: 1,
        segments: vec!["Shelf".into(), "Zone".into()],
    });
    assert_eq!(reorder, Err(LocationValidationError::UnsafeSchemaChange));
    let removal = save_location_schema(&mut connection, SaveLocationSchemaInput {
        expected_revision: 1,
        segments: vec!["Zone".into()],
    });
    assert_eq!(removal, Err(LocationValidationError::UnsafeSchemaChange));
    let append = save_location_schema(&mut connection, SaveLocationSchemaInput {
        expected_revision: 1,
        segments: vec!["Zone".into(), "Shelf".into(), "Drawer".into()],
    });
    assert_eq!(append, Err(LocationValidationError::UnsafeSchemaChange));
    assert_eq!(list_product_locations(&connection, true).unwrap()[0].code, "A1-SHELF2");
}

#[test]
fn generated_codes_are_unique_and_invalid_or_duplicate_segment_values_are_rejected() {
    let mut connection = open_seeded_catalog().unwrap();
    configure(&mut connection, &["Zone", "Shelf"]);
    let first = create_product_location(&mut connection, CreateProductLocationInput { values: vec!["A-1".into(), "Shelf 2".into()] }).unwrap();
    assert_eq!(first.code, "A1-SHELF2");
    let duplicate = create_product_location(&mut connection, CreateProductLocationInput { values: vec!["a1".into(), "shelf2".into()] });
    assert_eq!(duplicate, Err(LocationValidationError::DuplicateCode));
    let malformed = create_product_location(&mut connection, CreateProductLocationInput { values: vec![" ".into(), "Shelf".into()] });
    assert_eq!(malformed, Err(LocationValidationError::InvalidLocationValues));
    assert_eq!(list_product_locations(&connection, true).unwrap().len(), 1);
}

#[test]
fn product_assignment_is_optional_active_only_and_used_locations_are_protected() {
    let mut connection = open_seeded_catalog().unwrap();
    configure(&mut connection, &["Zone"]);
    let location = create_product_location(&mut connection, CreateProductLocationInput { values: vec!["A".into()] }).unwrap();
    assert_eq!(connection.query_row("SELECT primary_location_id FROM products WHERE id = 1", [], |row| row.get::<_, Option<i64>>(0)).unwrap(), None);

    assert_eq!(assign_product_location(&mut connection, 1, 0, Some(location.location_id)), Ok(1));
    assert_eq!(assign_product_location(&mut connection, 1, 1, None), Ok(2));
    assert_eq!(connection.query_row("SELECT primary_location_id FROM products WHERE id = 1", [], |row| row.get::<_, Option<i64>>(0)).unwrap(), None);
    assert_eq!(assign_product_location(&mut connection, 1, 2, Some(location.location_id)), Ok(3));
    assert_eq!(set_location_activity(&mut connection, location.location_id, 0, false), Err(LocationValidationError::LocationInUse));
    assert_eq!(delete_product_location(&mut connection, location.location_id, 0), Err(LocationValidationError::LocationInUse));

    connection.execute("UPDATE products SET primary_location_id = NULL WHERE id = 1", []).unwrap();
    let inactive = set_location_activity(&mut connection, location.location_id, 0, false).unwrap();
    assert!(!inactive.active);
    assert_eq!(assign_product_location(&mut connection, 1, 3, Some(location.location_id)), Err(LocationValidationError::InactiveLocation));
    assert!(connection.execute("UPDATE products SET primary_location_id = ?1 WHERE id = 1", [location.location_id]).is_err());
    assert_eq!(connection.query_row("SELECT quantity FROM stock_balances WHERE product_id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 8);
}

#[test]
fn command_requests_are_strict_and_success_contracts_are_stable() {
    assert!(serde_json::from_value::<SaveLocationSchemaRequest>(serde_json::json!({ "expected_revision": 0, "segments": ["Zone"], "extra": true })).is_err());
    assert!(serde_json::from_value::<CreateProductLocationRequest>(serde_json::json!({ "values": ["A"], "code": "OVERRIDE" })).is_err());
    assert!(serde_json::from_value::<AssignProductLocationRequest>(serde_json::json!({ "product_id": 1, "expected_revision": 0, "location_id": null })).is_ok());
    assert!(matches!(
        serde_json::from_value::<AssignProductLocationRequest>(serde_json::json!({ "product_id": 1, "expected_revision": 0 })).unwrap().location_id,
        repuestos_autos::commands::catalog::ExplicitNullableLocationId::Missing,
    ));
    let value = serde_json::to_value(ProductLocationResponse::Deleted).unwrap();
    assert_eq!(value, serde_json::json!({ "kind": "deleted" }));
}

#[test]
fn assignment_request_requires_location_id_but_accepts_explicit_null_for_revision_checked_unassignment() {
    let mut connection = open_seeded_catalog().unwrap();
    configure(&mut connection, &["Zone"]);
    let location = create_product_location(&mut connection, CreateProductLocationInput { values: vec!["A".into()] }).unwrap();
    assert_eq!(assign_product_location(&mut connection, 1, 0, Some(location.location_id)), Ok(1));

    let omitted = serde_json::from_value::<AssignProductLocationRequest>(serde_json::json!({
        "product_id": 1,
        "expected_revision": 1,
    })).unwrap();
    assert_eq!(assign_product_primary_location(&mut connection, omitted), ProductLocationResponse::Error(repuestos_autos::commands::catalog::CatalogLocationError {
        code: "validation_error",
        message: "Review the location values and try again.",
    }));
    assert_eq!(connection.query_row("SELECT primary_location_id FROM products WHERE id = 1", [], |row| row.get::<_, Option<i64>>(0)).unwrap(), Some(location.location_id));
    assert_eq!(connection.query_row("SELECT revision FROM products WHERE id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 1);

    let explicit_null = serde_json::from_value::<AssignProductLocationRequest>(serde_json::json!({
        "product_id": 1,
        "expected_revision": 1,
        "location_id": null,
    })).unwrap();
    assert_eq!(assign_product_primary_location(&mut connection, explicit_null), ProductLocationResponse::AssignmentSuccess { product_id: 1, location_id: None, revision: 2 });
    assert_eq!(connection.query_row("SELECT primary_location_id FROM products WHERE id = 1", [], |row| row.get::<_, Option<i64>>(0)).unwrap(), None);
    assert_eq!(connection.query_row("SELECT revision FROM products WHERE id = 1", [], |row| row.get::<_, i64>(0)).unwrap(), 2);
}
