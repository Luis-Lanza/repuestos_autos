use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Number,
    Option,
}

impl FieldType {
    pub fn parse(value: &str) -> Result<Self, CatalogValidationError> {
        match value {
            "text" => Ok(Self::Text),
            "number" => Ok(Self::Number),
            "option" => Ok(Self::Option),
            _ => Err(CatalogValidationError::InvalidFieldDefinition),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Option => "option",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CategoryFieldDraft {
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct AttributeDefinition {
    pub id: i64,
    pub field_type: FieldType,
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct AttributeValueDraft {
    pub definition_id: i64,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValidatedAttributeValue {
    Text {
        definition_id: i64,
        value: String,
    },
    Number {
        definition_id: i64,
        value: f64,
        searchable: String,
    },
    Option {
        definition_id: i64,
        value: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogValidationError {
    InvalidCategory,
    InvalidFieldDefinition,
    InvalidProduct,
    InvalidCatalogPrice,
    InvalidOpeningQuantity,
    MissingRequiredField,
    InvalidAttributeValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogActivity {
    Active,
    Archived,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogTarget {
    Category,
    Product,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogIntent {
    Archive,
    Reactivate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CatalogSnapshot {
    pub target: CatalogTarget,
    pub activity: CatalogActivity,
    pub category_activity: CatalogActivity,
    pub active_products: i64,
    pub values_valid: bool,
    pub revision: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransitionPlan {
    pub activity: CatalogActivity,
    pub expected_revision: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaintenanceError {
    InvalidProduct,
    InvalidCatalogPrice,
    InvalidAttributeValue,
    LifecycleBlocked,
}

pub fn has_normalized_collision(left: &str, right: &str) -> bool {
    normalize_identity(left) == normalize_identity(right)
}

pub fn normalize_identity(value: &str) -> String {
    value.trim().to_lowercase()
}

pub fn validate_maintenance_product(
    sku: &str,
    name: &str,
    catalog_unit_price_centavos: i64,
    definitions: &[AttributeDefinition],
    values: &[AttributeValueDraft],
) -> Result<Vec<ValidatedAttributeValue>, MaintenanceError> {
    if sku.trim().is_empty() || name.trim().is_empty() {
        return Err(MaintenanceError::InvalidProduct);
    }
    if catalog_unit_price_centavos <= 0 {
        return Err(MaintenanceError::InvalidCatalogPrice);
    }
    validate_attribute_values(definitions, values)
        .map_err(|_| MaintenanceError::InvalidAttributeValue)
}

pub fn plan_transition(
    snapshot: &CatalogSnapshot,
    intent: CatalogIntent,
) -> Result<TransitionPlan, MaintenanceError> {
    match (snapshot.target, intent) {
        (CatalogTarget::Category, CatalogIntent::Archive) if snapshot.active_products > 0 => {
            Err(MaintenanceError::LifecycleBlocked)
        }
        (CatalogTarget::Product, CatalogIntent::Reactivate)
            if snapshot.category_activity != CatalogActivity::Active || !snapshot.values_valid =>
        {
            Err(MaintenanceError::LifecycleBlocked)
        }
        (_, CatalogIntent::Archive) => Ok(TransitionPlan {
            activity: CatalogActivity::Archived,
            expected_revision: snapshot.revision,
        }),
        (_, CatalogIntent::Reactivate) => Ok(TransitionPlan {
            activity: CatalogActivity::Active,
            expected_revision: snapshot.revision,
        }),
    }
}

pub fn validate_category(
    name: &str,
    fields: &[CategoryFieldDraft],
) -> Result<(), CatalogValidationError> {
    if name.trim().is_empty() {
        return Err(CatalogValidationError::InvalidCategory);
    }
    let mut labels = HashSet::new();
    for field in fields {
        let label = field.label.trim().to_lowercase();
        let unique_options = field
            .options
            .iter()
            .map(|option| option.trim())
            .filter(|option| !option.is_empty())
            .collect::<HashSet<_>>();
        if label.is_empty()
            || !labels.insert(label)
            || (field.field_type == FieldType::Option
                && unique_options.len() != field.options.len())
            || (field.field_type == FieldType::Option && field.options.is_empty())
            || (field.field_type != FieldType::Option && !field.options.is_empty())
        {
            return Err(CatalogValidationError::InvalidFieldDefinition);
        }
    }
    Ok(())
}

pub fn validate_product(
    sku: &str,
    name: &str,
    catalog_unit_price_centavos: i64,
    opening_quantity: i64,
    definitions: &[AttributeDefinition],
    values: &[AttributeValueDraft],
) -> Result<Vec<ValidatedAttributeValue>, CatalogValidationError> {
    if sku.trim().is_empty() || name.trim().is_empty() {
        return Err(CatalogValidationError::InvalidProduct);
    }
    if catalog_unit_price_centavos <= 0 {
        return Err(CatalogValidationError::InvalidCatalogPrice);
    }
    if opening_quantity <= 0 {
        return Err(CatalogValidationError::InvalidOpeningQuantity);
    }

    validate_attribute_values(definitions, values)
}

fn validate_attribute_values(
    definitions: &[AttributeDefinition],
    values: &[AttributeValueDraft],
) -> Result<Vec<ValidatedAttributeValue>, CatalogValidationError> {
    let mut supplied = HashMap::new();
    for value in values {
        if supplied
            .insert(value.definition_id, value.value.trim())
            .is_some()
        {
            return Err(CatalogValidationError::InvalidAttributeValue);
        }
    }
    if supplied
        .keys()
        .any(|id| !definitions.iter().any(|definition| definition.id == *id))
    {
        return Err(CatalogValidationError::InvalidAttributeValue);
    }

    definitions
        .iter()
        .filter_map(|definition| match supplied.get(&definition.id) {
            Some(value) if !value.is_empty() => Some(validate_attribute(definition, value)),
            _ if definition.required => Some(Err(CatalogValidationError::MissingRequiredField)),
            _ => None,
        })
        .collect()
}

fn validate_attribute(
    definition: &AttributeDefinition,
    value: &str,
) -> Result<ValidatedAttributeValue, CatalogValidationError> {
    match definition.field_type {
        FieldType::Text => Ok(ValidatedAttributeValue::Text {
            definition_id: definition.id,
            value: value.into(),
        }),
        FieldType::Number => {
            let number = value
                .parse::<f64>()
                .map_err(|_| CatalogValidationError::InvalidAttributeValue)?;
            if !number.is_finite() {
                return Err(CatalogValidationError::InvalidAttributeValue);
            }
            Ok(ValidatedAttributeValue::Number {
                definition_id: definition.id,
                value: number,
                searchable: value.into(),
            })
        }
        FieldType::Option if definition.options.iter().any(|option| option == value) => {
            Ok(ValidatedAttributeValue::Option {
                definition_id: definition.id,
                value: value.into(),
            })
        }
        FieldType::Option => Err(CatalogValidationError::InvalidAttributeValue),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_product_rejects_missing_required_value() {
        let definitions = [AttributeDefinition {
            id: 1,
            field_type: FieldType::Text,
            required: true,
            options: vec![],
        }];

        let result = validate_product("SKU", "Product", 100, 1, &definitions, &[]);

        assert_eq!(result, Err(CatalogValidationError::MissingRequiredField));
    }

    #[test]
    fn validate_product_rejects_value_outside_predefined_options() {
        let definitions = [AttributeDefinition {
            id: 1,
            field_type: FieldType::Option,
            required: true,
            options: vec!["Rubber".into()],
        }];
        let values = [AttributeValueDraft {
            definition_id: 1,
            value: "Leather".into(),
        }];

        let result = validate_product("SKU", "Product", 100, 1, &definitions, &values);

        assert_eq!(result, Err(CatalogValidationError::InvalidAttributeValue));
    }
}
