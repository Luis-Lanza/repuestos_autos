use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocationSegment {
    pub id: i64,
    pub label: String,
    pub position: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocationSegmentDraft {
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocationSchema {
    pub revision: i64,
    pub segments: Vec<LocationSegment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalLocation {
    pub id: i64,
    pub code: String,
    pub values: Vec<String>,
    pub active: bool,
    pub revision: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationValidationError {
    InvalidSchema,
    UnsafeSchemaChange,
    InvalidLocationValues,
    DuplicateCode,
    MissingLocation,
    InactiveLocation,
    LocationInUse,
    StaleLocation,
    PersistenceFailure,
}

/// Validate the ordered global schema and preserve existing segment identity once locations exist.
pub fn validate_schema(
    existing: &[LocationSegment],
    requested: &[LocationSegmentDraft],
    locations_exist: bool,
) -> Result<Vec<LocationSegmentDraft>, LocationValidationError> {
    if requested.is_empty() || requested.len() > 8 {
        return Err(LocationValidationError::InvalidSchema);
    }
    let mut labels = HashSet::new();
    let mut normalized = Vec::with_capacity(requested.len());
    for segment in requested {
        let label = segment.label.trim();
        if label.is_empty()
            || label.chars().count() > 40
            || !labels.insert(label.to_lowercase())
        {
            return Err(LocationValidationError::InvalidSchema);
        }
        normalized.push(LocationSegmentDraft { label: label.to_owned() });
    }
    if locations_exist
        && (requested.len() != existing.len()
            || existing.iter().zip(&normalized).any(|(old, new)| old.label != new.label))
    {
        return Err(LocationValidationError::UnsafeSchemaChange);
    }
    Ok(normalized)
}

/// Codes are canonical, generated values; callers never supply a code directly.
pub fn generate_code(values: &[String]) -> Result<String, LocationValidationError> {
    if values.is_empty() || values.len() > 8 {
        return Err(LocationValidationError::InvalidLocationValues);
    }
    let mut code = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() || value.chars().count() > 40 {
            return Err(LocationValidationError::InvalidLocationValues);
        }
        let part = value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .map(|character| character.to_ascii_uppercase())
            .collect::<String>();
        if part.is_empty() {
            return Err(LocationValidationError::InvalidLocationValues);
        }
        code.push(part);
    }
    Ok(code.join("-"))
}

pub fn validate_values(
    schema: &LocationSchema,
    values: &[String],
) -> Result<String, LocationValidationError> {
    if values.len() != schema.segments.len() {
        return Err(LocationValidationError::InvalidLocationValues);
    }
    generate_code(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_code_is_canonical_and_not_caller_controlled() {
        assert_eq!(generate_code(&[" A-1 ".into(), " shelf 2 ".into()]), Ok("A1-SHELF2".into()));
    }

    #[test]
    fn schema_with_locations_cannot_change_segment_identity_or_order() {
        let existing = [LocationSegment { id: 1, label: "Zone".into(), position: 0 }];
        assert_eq!(validate_schema(&existing, &[LocationSegmentDraft { label: "Aisle".into() }], true), Err(LocationValidationError::UnsafeSchemaChange));
        assert_eq!(validate_schema(&existing, &[LocationSegmentDraft { label: "Zone".into() }, LocationSegmentDraft { label: "Aisle".into() }], true), Err(LocationValidationError::UnsafeSchemaChange));
    }
}
