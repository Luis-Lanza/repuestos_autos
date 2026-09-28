use std::collections::{HashMap, HashSet};

#[path = "location.rs"]
pub mod location;

pub const MAX_CATALOG_PRICE_CENTAVOS: i64 = 9_007_199_254_740_991;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CategoryFieldDraft {
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CategoryFieldLifecycle {
    Active,
    Retired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CategorySchemaField {
    pub definition_id: Option<i64>,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistingCategorySchemaField {
    pub definition_id: i64,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub options: Vec<String>,
    pub lifecycle: CategoryFieldLifecycle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CategorySchemaPlan {
    pub additions: Vec<CategoryFieldDraft>,
    pub retire_definition_ids: Vec<i64>,
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
    InvalidPurchasePrice,
    InvalidSalePrice,
    InvalidMinimumSalePrice,
    MinimumSalePriceExceedsSalePrice,
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
    InvalidPurchasePrice,
    InvalidSalePrice,
    InvalidMinimumSalePrice,
    MinimumSalePriceExceedsSalePrice,
    InvalidAttributeValue,
    LifecycleBlocked,
    InvalidCategorySchema,
    ImmutableCategoryField,
}

pub fn plan_category_schema_edit(
    existing: &[ExistingCategorySchemaField],
    requested: &[CategorySchemaField],
) -> Result<CategorySchemaPlan, MaintenanceError> {
    let active = existing
        .iter()
        .filter(|field| field.lifecycle == CategoryFieldLifecycle::Active);
    let mut seen_ids = HashSet::new();
    let mut additions = Vec::new();
    let mut candidate = Vec::new();
    let mut retained = HashSet::new();
    for field in requested {
        if field.label.trim().is_empty() {
            return Err(MaintenanceError::InvalidCategorySchema);
        }
        match field.definition_id {
            Some(id) => {
                if !seen_ids.insert(id) {
                    return Err(MaintenanceError::InvalidCategorySchema);
                }
                let original = existing
                    .iter()
                    .find(|original| original.definition_id == id)
                    .ok_or(MaintenanceError::InvalidCategorySchema)?;
                if original.lifecycle != CategoryFieldLifecycle::Active {
                    return Err(MaintenanceError::InvalidCategorySchema);
                }
                if original.label != field.label.trim()
                    || original.field_type != field.field_type
                    || original.required != field.required
                    || original.options
                        != field
                            .options
                            .iter()
                            .map(|option| option.trim().to_owned())
                            .collect::<Vec<_>>()
                {
                    return Err(MaintenanceError::ImmutableCategoryField);
                }
                retained.insert(id);
                candidate.push(CategoryFieldDraft {
                    label: original.label.clone(),
                    field_type: original.field_type,
                    required: original.required,
                    options: original.options.clone(),
                });
            }
            None => additions.push(CategoryFieldDraft {
                label: field.label.trim().to_owned(),
                field_type: field.field_type,
                required: field.required,
                options: field
                    .options
                    .iter()
                    .map(|option| option.trim().to_owned())
                    .collect(),
            }),
        }
    }
    let retire_definition_ids = active
        .filter(|field| !retained.contains(&field.definition_id))
        .map(|field| field.definition_id)
        .collect::<Vec<_>>();
    candidate.extend(additions.iter().cloned());
    validate_category("category", &candidate)
        .map_err(|_| MaintenanceError::InvalidCategorySchema)?;
    Ok(CategorySchemaPlan {
        additions,
        retire_definition_ids,
    })
}

pub fn validate_maintenance_category(name: &str) -> Result<(), MaintenanceError> {
    (!name.trim().is_empty())
        .then_some(())
        .ok_or(MaintenanceError::InvalidProduct)
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
    purchase_price_centavos: i64,
    sale_price_centavos: i64,
    minimum_sale_price_centavos: i64,
    definitions: &[AttributeDefinition],
    values: &[AttributeValueDraft],
) -> Result<Vec<ValidatedAttributeValue>, MaintenanceError> {
    if sku.trim().is_empty() || name.trim().is_empty() {
        return Err(MaintenanceError::InvalidProduct);
    }
    validate_current_prices(
        purchase_price_centavos,
        sale_price_centavos,
        minimum_sale_price_centavos,
    )
    .map_err(|error| match error {
        CatalogValidationError::InvalidPurchasePrice => MaintenanceError::InvalidPurchasePrice,
        CatalogValidationError::InvalidSalePrice => MaintenanceError::InvalidSalePrice,
        CatalogValidationError::InvalidMinimumSalePrice => {
            MaintenanceError::InvalidMinimumSalePrice
        }
        _ => MaintenanceError::MinimumSalePriceExceedsSalePrice,
    })?;
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
    sale_price_centavos: i64,
    minimum_sale_price_centavos: i64,
    opening_quantity: i64,
    definitions: &[AttributeDefinition],
    values: &[AttributeValueDraft],
) -> Result<Vec<ValidatedAttributeValue>, CatalogValidationError> {
    if sku.trim().is_empty() || name.trim().is_empty() {
        return Err(CatalogValidationError::InvalidProduct);
    }
    if sale_price_centavos <= 0 || sale_price_centavos > MAX_CATALOG_PRICE_CENTAVOS {
        return Err(CatalogValidationError::InvalidSalePrice);
    }
    if minimum_sale_price_centavos <= 0 || minimum_sale_price_centavos > MAX_CATALOG_PRICE_CENTAVOS
    {
        return Err(CatalogValidationError::InvalidMinimumSalePrice);
    }
    if minimum_sale_price_centavos > sale_price_centavos {
        return Err(CatalogValidationError::MinimumSalePriceExceedsSalePrice);
    }
    if opening_quantity <= 0 {
        return Err(CatalogValidationError::InvalidOpeningQuantity);
    }

    validate_attribute_values(definitions, values)
}

pub fn validate_current_prices(
    purchase_price_centavos: i64,
    sale_price_centavos: i64,
    minimum_sale_price_centavos: i64,
) -> Result<(), CatalogValidationError> {
    if purchase_price_centavos <= 0 || purchase_price_centavos > MAX_CATALOG_PRICE_CENTAVOS {
        return Err(CatalogValidationError::InvalidPurchasePrice);
    }
    if sale_price_centavos <= 0 || sale_price_centavos > MAX_CATALOG_PRICE_CENTAVOS {
        return Err(CatalogValidationError::InvalidSalePrice);
    }
    if minimum_sale_price_centavos <= 0 || minimum_sale_price_centavos > MAX_CATALOG_PRICE_CENTAVOS
    {
        return Err(CatalogValidationError::InvalidMinimumSalePrice);
    }
    if minimum_sale_price_centavos > sale_price_centavos {
        return Err(CatalogValidationError::MinimumSalePriceExceedsSalePrice);
    }
    Ok(())
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

        let result = validate_product("SKU", "Product", 100, 100, 1, &definitions, &[]);

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

        let result = validate_product("SKU", "Product", 100, 100, 1, &definitions, &values);

        assert_eq!(result, Err(CatalogValidationError::InvalidAttributeValue));
    }
}
