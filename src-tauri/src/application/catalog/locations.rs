use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::Serialize;

use crate::domain::catalog::location::{
    generate_code, validate_schema, validate_values, LocationSchema, LocationSegment,
    LocationSegmentDraft, LocationValidationError,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LocationSegmentContract {
    pub id: i64,
    pub label: String,
    pub position: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LocationSchemaContract {
    pub revision: i64,
    pub segments: Vec<LocationSegmentContract>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProductLocationContract {
    pub location_id: i64,
    pub code: String,
    pub values: Vec<String>,
    pub active: bool,
    pub revision: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveLocationSchemaInput {
    pub expected_revision: i64,
    pub segments: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateProductLocationInput {
    pub values: Vec<String>,
}

pub fn read_location_schema(connection: &Connection) -> Result<LocationSchemaContract, LocationValidationError> {
    let revision = connection.query_row("SELECT revision FROM location_schema WHERE id = 1", [], |row| row.get(0))
        .map_err(|_| LocationValidationError::PersistenceFailure)?;
    let mut statement = connection.prepare("SELECT id, label, position FROM location_segments WHERE schema_id = 1 ORDER BY position")
        .map_err(|_| LocationValidationError::PersistenceFailure)?;
    let segments = statement.query_map([], |row| Ok(LocationSegmentContract { id: row.get(0)?, label: row.get(1)?, position: row.get(2)? }))
        .map_err(|_| LocationValidationError::PersistenceFailure)?
        .collect::<rusqlite::Result<Vec<_>>>().map_err(|_| LocationValidationError::PersistenceFailure)?;
    Ok(LocationSchemaContract { revision, segments })
}

pub fn save_location_schema(connection: &mut Connection, input: SaveLocationSchemaInput) -> Result<LocationSchemaContract, LocationValidationError> {
    if input.expected_revision < 0 { return Err(LocationValidationError::StaleLocation); }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| LocationValidationError::PersistenceFailure)?;
    let current = read_location_schema(&transaction)?;
    if current.revision != input.expected_revision { return Err(LocationValidationError::StaleLocation); }
    let existing = current.segments.iter().map(|segment| LocationSegment { id: segment.id, label: segment.label.clone(), position: segment.position }).collect::<Vec<_>>();
    let requested = input.segments.iter().map(|label| LocationSegmentDraft { label: label.clone() }).collect::<Vec<_>>();
    let locations_exist: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM product_locations)", [], |row| row.get(0))
        .map_err(|_| LocationValidationError::PersistenceFailure)?;
    let normalized = validate_schema(&existing, &requested, locations_exist)?;
    if !locations_exist {
        transaction.execute("DELETE FROM location_segments WHERE schema_id = 1", []).map_err(|_| LocationValidationError::PersistenceFailure)?;
        for (position, segment) in normalized.iter().enumerate() {
            transaction.execute("INSERT INTO location_segments (schema_id, position, label) VALUES (1, ?1, ?2)", params![position as i64, segment.label])
                .map_err(|_| LocationValidationError::PersistenceFailure)?;
        }
    }
    let changed = transaction.execute("UPDATE location_schema SET revision = revision + 1 WHERE id = 1 AND revision = ?1 AND revision < 9223372036854775807", [input.expected_revision])
        .map_err(|_| LocationValidationError::PersistenceFailure)?;
    if changed != 1 { return Err(LocationValidationError::StaleLocation); }
    let result = read_location_schema(&transaction)?;
    transaction.commit().map_err(|_| LocationValidationError::PersistenceFailure)?;
    Ok(result)
}

pub fn create_product_location(connection: &mut Connection, input: CreateProductLocationInput) -> Result<ProductLocationContract, LocationValidationError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| LocationValidationError::PersistenceFailure)?;
    let schema = read_location_schema(&transaction)?;
    let schema = LocationSchema { revision: schema.revision, segments: schema.segments.iter().map(|segment| LocationSegment { id: segment.id, label: segment.label.clone(), position: segment.position }).collect() };
    let code = validate_values(&schema, &input.values)?;
    transaction.execute("INSERT INTO product_locations (code) VALUES (?1)", [&code]).map_err(|error| {
        if matches!(error, rusqlite::Error::SqliteFailure(ref failure, _) if failure.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE) {
            LocationValidationError::DuplicateCode
        } else { LocationValidationError::PersistenceFailure }
    })?;
    let location_id = transaction.last_insert_rowid();
    for (segment, value) in schema.segments.iter().zip(&input.values) {
        transaction.execute("INSERT INTO product_location_values (location_id, segment_id, value) VALUES (?1, ?2, ?3)", params![location_id, segment.id, value.trim()])
            .map_err(|_| LocationValidationError::PersistenceFailure)?;
    }
    transaction.commit().map_err(|_| LocationValidationError::PersistenceFailure)?;
    Ok(ProductLocationContract { location_id, code, values: input.values.into_iter().map(|value| value.trim().to_owned()).collect(), active: true, revision: 0 })
}

pub fn list_product_locations(connection: &Connection, include_inactive: bool) -> Result<Vec<ProductLocationContract>, LocationValidationError> {
    let mut statement = connection.prepare("SELECT id, code, active, revision FROM product_locations WHERE (?1 = 1 OR active = 1) ORDER BY code, id")
        .map_err(|_| LocationValidationError::PersistenceFailure)?;
    let locations = statement.query_map([include_inactive], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, bool>(2)?, row.get::<_, i64>(3)?)))
        .map_err(|_| LocationValidationError::PersistenceFailure)?
        .collect::<rusqlite::Result<Vec<_>>>().map_err(|_| LocationValidationError::PersistenceFailure)?;
    locations.into_iter().map(|(id, code, active, revision)| {
        let mut values = connection.prepare("SELECT value FROM product_location_values WHERE location_id = ?1 ORDER BY (SELECT position FROM location_segments WHERE id = segment_id)")
            .map_err(|_| LocationValidationError::PersistenceFailure)?;
        let values = values.query_map([id], |row| row.get(0)).map_err(|_| LocationValidationError::PersistenceFailure)?
            .collect::<rusqlite::Result<Vec<String>>>().map_err(|_| LocationValidationError::PersistenceFailure)?;
        Ok(ProductLocationContract { location_id: id, code, values, active, revision })
    }).collect()
}

pub fn set_location_activity(connection: &mut Connection, location_id: i64, expected_revision: i64, active: bool) -> Result<ProductLocationContract, LocationValidationError> {
    if location_id <= 0 || expected_revision < 0 { return Err(LocationValidationError::StaleLocation); }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|_| LocationValidationError::PersistenceFailure)?;
    let changed = transaction.execute("UPDATE product_locations SET active = ?1, revision = revision + 1 WHERE id = ?2 AND revision = ?3 AND revision < 9223372036854775807", params![active, location_id, expected_revision]);
    match changed {
        Ok(1) => (),
        Ok(_) => return Err(LocationValidationError::StaleLocation),
        Err(rusqlite::Error::SqliteFailure(ref failure, _)) if failure.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_TRIGGER => return Err(LocationValidationError::LocationInUse),
        Err(_) => return Err(LocationValidationError::PersistenceFailure),
    }
    let result = read_location(&transaction, location_id)?.ok_or(LocationValidationError::MissingLocation)?;
    transaction.commit().map_err(|_| LocationValidationError::PersistenceFailure)?;
    Ok(result)
}

pub fn delete_product_location(connection: &mut Connection, location_id: i64, expected_revision: i64) -> Result<(), LocationValidationError> {
    if location_id <= 0 || expected_revision < 0 { return Err(LocationValidationError::MissingLocation); }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|_| LocationValidationError::PersistenceFailure)?;
    let revision: Option<i64> = transaction.query_row("SELECT revision FROM product_locations WHERE id = ?1", [location_id], |row| row.get(0)).optional().map_err(|_| LocationValidationError::PersistenceFailure)?;
    match revision {
        None => return Err(LocationValidationError::MissingLocation),
        Some(revision) if revision != expected_revision => return Err(LocationValidationError::StaleLocation),
        Some(_) => (),
    }
    transaction.execute("DELETE FROM product_locations WHERE id = ?1", [location_id]).map_err(|error| {
        if matches!(error, rusqlite::Error::SqliteFailure(ref failure, _) if failure.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_TRIGGER || failure.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY) { LocationValidationError::LocationInUse } else { LocationValidationError::PersistenceFailure }
    })?;
    transaction.commit().map_err(|_| LocationValidationError::PersistenceFailure)
}

pub fn assign_product_location(connection: &mut Connection, product_id: i64, expected_revision: i64, location_id: Option<i64>) -> Result<i64, LocationValidationError> {
    if product_id <= 0 || expected_revision < 0 || location_id.is_some_and(|id| id <= 0) { return Err(LocationValidationError::MissingLocation); }
    if let Some(id) = location_id {
        let active: Option<bool> = connection.query_row("SELECT active FROM product_locations WHERE id = ?1", [id], |row| row.get(0)).optional().map_err(|_| LocationValidationError::PersistenceFailure)?;
        match active { None => return Err(LocationValidationError::MissingLocation), Some(false) => return Err(LocationValidationError::InactiveLocation), Some(true) => () }
    }
    let changed = connection.execute("UPDATE products SET primary_location_id = ?1, revision = revision + 1 WHERE id = ?2 AND revision = ?3 AND revision < 9223372036854775807", params![location_id, product_id, expected_revision]).map_err(|_| LocationValidationError::PersistenceFailure)?;
    if changed == 0 {
        let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM products WHERE id = ?1)", [product_id], |row| row.get(0)).map_err(|_| LocationValidationError::PersistenceFailure)?;
        return Err(if exists { LocationValidationError::StaleLocation } else { LocationValidationError::MissingLocation });
    }
    Ok(expected_revision + 1)
}

fn read_location(connection: &Connection, id: i64) -> Result<Option<ProductLocationContract>, LocationValidationError> {
    let Some((code, active, revision)) = connection.query_row("SELECT code, active, revision FROM product_locations WHERE id = ?1", [id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?, row.get::<_, i64>(2)?))).optional().map_err(|_| LocationValidationError::PersistenceFailure)? else { return Ok(None); };
    let values = connection.prepare("SELECT value FROM product_location_values WHERE location_id = ?1 ORDER BY (SELECT position FROM location_segments WHERE id = segment_id)").map_err(|_| LocationValidationError::PersistenceFailure)?
        .query_map([id], |row| row.get(0)).map_err(|_| LocationValidationError::PersistenceFailure)?
        .collect::<rusqlite::Result<Vec<String>>>().map_err(|_| LocationValidationError::PersistenceFailure)?;
    Ok(Some(ProductLocationContract { location_id: id, code, values, active, revision }))
}

pub fn generated_code(values: &[String]) -> Result<String, LocationValidationError> { generate_code(values) }
