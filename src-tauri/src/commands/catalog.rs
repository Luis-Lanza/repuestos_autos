use serde::{Deserialize, Serialize};

use crate::application::catalog;
use crate::application::catalog::{BrowseProductsInput, ProductActivityFilter, ProductStockFilter};
use crate::domain::catalog::{CatalogActivity, CatalogIntent, CatalogTarget, CategorySchemaField, FieldType};
use crate::infrastructure::sqlite::SqliteCatalogRepository;
use crate::application::catalog::access::{CatalogAccessError, CatalogAccessSession, CatalogAccessStatus};
use crate::application::catalog::locations as product_locations;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveLocationSchemaRequest {
    pub expected_revision: i64,
    pub segments: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListProductLocationsRequest {
    pub include_inactive: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProductLocationRequest {
    pub values: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductLocationLifecycleRequest {
    pub location_id: i64,
    pub expected_revision: i64,
}

#[derive(Debug, Default)]
pub enum ExplicitNullableLocationId {
    #[default]
    Missing,
    Null,
    Value(i64),
}

impl<'de> Deserialize<'de> for ExplicitNullableLocationId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::<i64>::deserialize(deserializer).map(|value| match value {
            Some(location_id) => Self::Value(location_id),
            None => Self::Null,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignProductLocationRequest {
    pub product_id: i64,
    pub expected_revision: i64,
    #[serde(default)]
    pub location_id: ExplicitNullableLocationId,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProductLocationResponse {
    SchemaSuccess { schema: product_locations::LocationSchemaContract },
    LocationsSuccess { locations: Vec<product_locations::ProductLocationContract> },
    LocationSuccess { location: product_locations::ProductLocationContract },
    AssignmentSuccess { product_id: i64, location_id: Option<i64>, revision: i64 },
    Deleted,
    Error(CatalogLocationError),
}

#[derive(Debug, PartialEq, Serialize)]
pub struct CatalogLocationError {
    pub code: &'static str,
    pub message: &'static str,
}

pub fn location_schema(connection: &rusqlite::Connection) -> ProductLocationResponse {
    match product_locations::read_location_schema(connection) {
        Ok(schema) => ProductLocationResponse::SchemaSuccess { schema },
        Err(error) => ProductLocationResponse::Error(map_location_error(error)),
    }
}

pub fn save_location_schema(connection: &mut rusqlite::Connection, request: SaveLocationSchemaRequest) -> ProductLocationResponse {
    if request.expected_revision < 0 { return ProductLocationResponse::Error(map_location_error(crate::domain::catalog::location::LocationValidationError::InvalidSchema)); }
    match product_locations::save_location_schema(connection, product_locations::SaveLocationSchemaInput { expected_revision: request.expected_revision, segments: request.segments }) {
        Ok(schema) => ProductLocationResponse::SchemaSuccess { schema },
        Err(error) => ProductLocationResponse::Error(map_location_error(error)),
    }
}

pub fn list_product_locations(connection: &rusqlite::Connection, request: ListProductLocationsRequest) -> ProductLocationResponse {
    match product_locations::list_product_locations(connection, request.include_inactive) {
        Ok(locations) => ProductLocationResponse::LocationsSuccess { locations },
        Err(error) => ProductLocationResponse::Error(map_location_error(error)),
    }
}

pub fn create_product_location(connection: &mut rusqlite::Connection, request: CreateProductLocationRequest) -> ProductLocationResponse {
    match product_locations::create_product_location(connection, product_locations::CreateProductLocationInput { values: request.values }) {
        Ok(location) => ProductLocationResponse::LocationSuccess { location },
        Err(error) => ProductLocationResponse::Error(map_location_error(error)),
    }
}

pub fn set_product_location_activity(connection: &mut rusqlite::Connection, request: ProductLocationLifecycleRequest, active: bool) -> ProductLocationResponse {
    match product_locations::set_location_activity(connection, request.location_id, request.expected_revision, active) {
        Ok(location) => ProductLocationResponse::LocationSuccess { location },
        Err(error) => ProductLocationResponse::Error(map_location_error(error)),
    }
}

pub fn delete_product_location(connection: &mut rusqlite::Connection, request: ProductLocationLifecycleRequest) -> ProductLocationResponse {
    if request.location_id <= 0 { return ProductLocationResponse::Error(map_location_error(crate::domain::catalog::location::LocationValidationError::MissingLocation)); }
    match product_locations::delete_product_location(connection, request.location_id, request.expected_revision) {
        Ok(()) => ProductLocationResponse::Deleted,
        Err(error) => ProductLocationResponse::Error(map_location_error(error)),
    }
}

pub fn assign_product_primary_location(connection: &mut rusqlite::Connection, request: AssignProductLocationRequest) -> ProductLocationResponse {
    let location_id = match request.location_id {
        ExplicitNullableLocationId::Missing => return ProductLocationResponse::Error(map_location_error(crate::domain::catalog::location::LocationValidationError::InvalidLocationValues)),
        ExplicitNullableLocationId::Null => None,
        ExplicitNullableLocationId::Value(location_id) => Some(location_id),
    };
    match product_locations::assign_product_location(connection, request.product_id, request.expected_revision, location_id) {
        Ok(revision) => ProductLocationResponse::AssignmentSuccess { product_id: request.product_id, location_id, revision },
        Err(error) => ProductLocationResponse::Error(map_location_error(error)),
    }
}

fn map_location_error(error: crate::domain::catalog::location::LocationValidationError) -> CatalogLocationError {
    use crate::domain::catalog::location::LocationValidationError as E;
    match error {
        E::InvalidSchema | E::InvalidLocationValues => CatalogLocationError { code: "validation_error", message: "Review the location values and try again." },
        E::UnsafeSchemaChange => CatalogLocationError { code: "location_schema_in_use", message: "Existing locations require the current segment order." },
        E::DuplicateCode => CatalogLocationError { code: "duplicate_location_code", message: "A location with these values already exists." },
        E::MissingLocation => CatalogLocationError { code: "location_unavailable", message: "This location is unavailable." },
        E::InactiveLocation => CatalogLocationError { code: "location_inactive", message: "Choose an active location." },
        E::LocationInUse => CatalogLocationError { code: "location_in_use", message: "A location assigned to a product cannot be removed or deactivated." },
        E::StaleLocation => CatalogLocationError { code: "stale_location", message: "Location data changed. Reload and try again." },
        E::PersistenceFailure => CatalogLocationError { code: "persistence_failure", message: "The location change could not be completed." },
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchProductsRequest {
    pub query: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowseProductsRequest {
    pub query: Option<String>,
    pub category_id: Option<i64>,
    pub stock_state: String,
    pub activity: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProductBrowseResponse {
    Success(catalog::ProductBrowsePage),
    Error(CatalogBrowseError),
}

#[derive(Debug, PartialEq, Serialize)]
pub struct CatalogBrowseError {
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaintainCatalogRequest {
    pub target: String,
    pub entity_id: i64,
    pub intent: String,
    pub expected_revision: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogMetadataDetailRequest {
    pub target: String,
    pub entity_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditCategorySchemaRequest {
    pub category_id: i64,
    pub expected_revision: i64,
    pub fields: Vec<EditCategorySchemaFieldRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditCategorySchemaFieldRequest {
    pub definition_id: Option<i64>,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditAttributeValueRequest {
    pub definition_id: i64,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductImageRequest {
    pub product_id: i64,
    pub expected_revision: i64,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProductImageResponse {
    Success { product_id: i64, revision: i64 },
    Cancelled,
    Error(CatalogMaintenanceError),
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProductImageThumbnailResponse {
    Success { product_id: i64, revision: i64, mime_type: &'static str, encoding: &'static str, bytes: String },
    Error(CatalogMaintenanceError),
}

pub fn parse_product_image_request(request: ProductImageRequest) -> Result<ProductImageRequest, ()> {
    if request.product_id <= 0 || request.expected_revision < 0 { Err(()) } else { Ok(request) }
}

#[cfg(feature = "desktop")]
pub(crate) fn persist_selected_product_image(connection: &mut rusqlite::Connection, product_id: i64, expected_revision: i64, mime_type: &str, bytes: Vec<u8>) -> ProductImageResponse {
    if product_id <= 0 || expected_revision < 0 { return ProductImageResponse::Error(validation_error()); }
    let Ok(image) = catalog::ProductImage::new(mime_type, bytes) else { return ProductImageResponse::Error(validation_error()); };
    match catalog::replace_product_image(connection, product_id, expected_revision, &image) {
        Ok(revision) => ProductImageResponse::Success { product_id, revision },
        Err(error) => ProductImageResponse::Error(map_image_error(error)),
    }
}

pub fn remove_product_image(connection: &mut rusqlite::Connection, request: ProductImageRequest) -> ProductImageResponse {
    let Ok(request) = parse_product_image_request(request) else { return ProductImageResponse::Error(validation_error()); };
    match catalog::remove_product_image(connection, request.product_id, request.expected_revision) {
        Ok(revision) => ProductImageResponse::Success { product_id: request.product_id, revision },
        Err(error) => ProductImageResponse::Error(map_image_error(error)),
    }
}

pub fn catalog_product_image_thumbnail(connection: &rusqlite::Connection, request: ProductImageRequest) -> ProductImageThumbnailResponse {
    let Ok(request) = parse_product_image_request(request) else { return ProductImageThumbnailResponse::Error(validation_error()); };
    match catalog::read_product_image_thumbnail(connection, request.product_id) {
        Ok(Some(thumbnail)) if thumbnail.revision == request.expected_revision => ProductImageThumbnailResponse::Success {
            product_id: thumbnail.product_id, revision: thumbnail.revision, mime_type: thumbnail.mime_type,
            encoding: "base64", bytes: encode_base64(&thumbnail.bytes),
        },
        Ok(Some(_)) => ProductImageThumbnailResponse::Error(stale_catalog_error()),
        Ok(None) => match catalog::read_catalog_metadata_detail(connection, CatalogTarget::Product, request.product_id) {
            Ok(Some(catalog::CatalogMetadataDetail::Product { revision, .. })) if revision == request.expected_revision => ProductImageThumbnailResponse::Error(image_unavailable_error()),
            Ok(Some(_)) => ProductImageThumbnailResponse::Error(stale_catalog_error()),
            Ok(None) => ProductImageThumbnailResponse::Error(unavailable_error()),
            Err(_) => ProductImageThumbnailResponse::Error(persistence_error()),
        },
        Err(_) => ProductImageThumbnailResponse::Error(persistence_error()),
    }
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let value = ((chunk[0] as u32) << 16)
            | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8)
            | chunk.get(2).copied().unwrap_or(0) as u32;
        encoded.push(TABLE[((value >> 18) & 63) as usize] as char);
        encoded.push(TABLE[((value >> 12) & 63) as usize] as char);
        encoded.push(if chunk.len() > 1 { TABLE[((value >> 6) & 63) as usize] as char } else { '=' });
        encoded.push(if chunk.len() > 2 { TABLE[(value & 63) as usize] as char } else { '=' });
    }
    encoded
}

fn map_image_error(error: catalog::ProductImagePersistenceError) -> CatalogMaintenanceError {
    match error {
        catalog::ProductImagePersistenceError::MissingProduct => unavailable_error(),
        catalog::ProductImagePersistenceError::StaleCatalogRecord => stale_catalog_error(),
        catalog::ProductImagePersistenceError::PersistenceFailure => persistence_error(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "target", rename_all = "snake_case", deny_unknown_fields)]
pub enum EditCatalogRequest {
    Category {
        entity_id: i64,
        expected_revision: i64,
        name: String,
    },
    Product {
        entity_id: i64,
        expected_revision: i64,
        sku: String,
        name: String,
        purchase_price_centavos: i64,
        #[serde(alias = "list_price_centavos")]
        sale_price_centavos: i64,
        minimum_sale_price_centavos: i64,
        #[serde(default)]
        low_stock_threshold: Option<i64>,
        expected_category_revision: i64,
        attribute_values: Vec<EditAttributeValueRequest>,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CatalogMaintenanceResponse {
    Success(CatalogMaintenanceRecord),
    Error(CatalogMaintenanceError),
}
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CatalogMaintenanceListResponse {
    Success {
        records: Vec<CatalogMaintenanceRecord>,
    },
    Error(CatalogMaintenanceError),
}
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CatalogMetadataDetailResponse {
    Success(catalog::CatalogMetadataDetail),
    Error(CatalogMaintenanceError),
}
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CatalogMaintenanceRecord {
    pub entity_id: i64,
    pub target: &'static str,
    pub label: String,
    pub activity: &'static str,
    pub revision: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_product_count: Option<i64>,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CatalogMaintenanceError {
    pub code: &'static str,
    pub message: &'static str,
}

pub use catalog::{ProductBrowseCategory, ProductBrowsePage, ProductBrowseResult, ProductSearchResult};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogSecretRequest { pub password: String }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogPasswordChangeRequest { pub current_password: String, pub new_password: String }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogRecoveryRequest { pub recovery_code: String, pub new_password: String }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogConfirmationRequest { pub confirmed: bool }

#[derive(PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CatalogAccessResponse {
    Status { status: &'static str },
    RecoveryCode { recovery_code: String },
    Success,
    Error { code: &'static str, message: &'static str },
}

pub fn catalog_access_status(session: &CatalogAccessSession) -> CatalogAccessResponse {
    let status = match session.status() {
        CatalogAccessStatus::SetupRequired => "setup_required",
        CatalogAccessStatus::Locked => "locked",
        CatalogAccessStatus::Unlocked => "unlocked",
        CatalogAccessStatus::Unavailable => "unavailable",
    };
    CatalogAccessResponse::Status { status }
}

pub fn begin_catalog_setup(session: &CatalogAccessSession, request: CatalogSecretRequest) -> CatalogAccessResponse {
    session.begin_setup(&request.password).map_or_else(access_error, |recovery_code| CatalogAccessResponse::RecoveryCode { recovery_code })
}

pub fn finish_catalog_setup(session: &CatalogAccessSession, request: CatalogConfirmationRequest) -> CatalogAccessResponse {
    session.finish_setup(request.confirmed).map_or_else(access_error, |_| CatalogAccessResponse::Success)
}

pub fn unlock_catalog(session: &CatalogAccessSession, request: CatalogSecretRequest) -> CatalogAccessResponse {
    session.unlock(&request.password).map_or_else(access_error, |_| CatalogAccessResponse::Success)
}

pub fn change_catalog_password(session: &CatalogAccessSession, request: CatalogPasswordChangeRequest) -> CatalogAccessResponse {
    session.change_password(&request.current_password, &request.new_password).map_or_else(access_error, |_| CatalogAccessResponse::Success)
}

pub fn begin_catalog_recovery(session: &CatalogAccessSession, request: CatalogRecoveryRequest) -> CatalogAccessResponse {
    session.begin_recovery(&request.recovery_code, &request.new_password).map_or_else(access_error, |recovery_code| CatalogAccessResponse::RecoveryCode { recovery_code })
}

pub fn finish_catalog_recovery(session: &CatalogAccessSession, request: CatalogConfirmationRequest) -> CatalogAccessResponse {
    session.finish_recovery(request.confirmed).map_or_else(access_error, |_| CatalogAccessResponse::Success)
}

fn access_error(error: CatalogAccessError) -> CatalogAccessResponse {
    let (code, message) = match error {
        CatalogAccessError::Invalid => ("validation_error", "Ingresá una contraseña válida y confirmá el código de recuperación guardado."),
        CatalogAccessError::Unauthorized => ("invalid_credentials", "La contraseña o el código de recuperación no es válido."),
        CatalogAccessError::AlreadyConfigured => ("already_configured", "El acceso al catálogo ya está configurado."),
        CatalogAccessError::NotConfigured => ("setup_required", "Configurá el acceso al catálogo para continuar."),
        CatalogAccessError::Pending => ("setup_pending", "Completá la confirmación pendiente para continuar."),
        CatalogAccessError::Storage | CatalogAccessError::Corrupt => ("access_unavailable", "No se pudo acceder a la configuración local del catálogo."),
    };
    CatalogAccessResponse::Error { code, message }
}

pub fn browse_products(
    connection: &rusqlite::Connection,
    request: BrowseProductsRequest,
) -> ProductBrowseResponse {
    let stock_filter = match request.stock_state.as_str() {
        "all" => ProductStockFilter::All,
        "low_stock" => ProductStockFilter::LowStock,
        "out_of_stock" => ProductStockFilter::OutOfStock,
        "available" => ProductStockFilter::Available,
        "alerts" => ProductStockFilter::Alerts,
        _ => return ProductBrowseResponse::Error(browse_validation_error()),
    };
    let activity_filter = match request.activity.as_str() {
        "active" => ProductActivityFilter::Active,
        "archived" => ProductActivityFilter::Archived,
        "all" => ProductActivityFilter::All,
        _ => return ProductBrowseResponse::Error(browse_validation_error()),
    };
    if request.category_id.is_some_and(|id| id <= 0)
        || request.page < 1
        || request.page_size < 1
        || request.page_size > 50
    {
        return ProductBrowseResponse::Error(browse_validation_error());
    }
    match catalog::browse_active_products(
        connection,
        &SqliteCatalogRepository,
        &BrowseProductsInput {
            query: request.query,
            category_id: request.category_id,
            stock_filter,
            activity_filter,
            page: request.page,
            page_size: request.page_size,
        },
    ) {
        Ok(page) => ProductBrowseResponse::Success(page),
        Err(_) => ProductBrowseResponse::Error(browse_persistence_error()),
    }
}

pub fn search_products(
    connection: &rusqlite::Connection,
    request: SearchProductsRequest,
) -> Result<Vec<ProductSearchResult>, String> {
    catalog::search_active_products(connection, &request.query)
        .map_err(|_| "persistence_failure".into())
}

pub fn search_sale_products(
    connection: &rusqlite::Connection,
    request: SearchProductsRequest,
) -> Result<Vec<catalog::SaleProductSearchResult>, String> {
    catalog::search_active_sale_products(connection, &request.query)
        .map_err(|_| "persistence_failure".into())
}

pub fn list_catalog_categories(
    connection: &rusqlite::Connection,
) -> Result<CatalogMaintenanceListResponse, String> {
    catalog::list_category_metadata(connection, SqliteCatalogRepository)
        .map(|categories| CatalogMaintenanceListResponse::Success {
            records: categories
                .into_iter()
                .map(|category| CatalogMaintenanceRecord {
                    entity_id: category.category_id,
                    target: "category",
                    label: category.name,
                    activity: if category.active { "active" } else { "archived" },
                    revision: category.revision,
                    active_product_count: Some(category.active_product_count),
                })
                .collect(),
        })
        .map_err(|_| "persistence_failure".into())
}

pub fn list_catalog_maintenance(
    connection: &rusqlite::Connection,
) -> Result<CatalogMaintenanceListResponse, String> {
    let categories = connection
        .prepare("SELECT id, name, active, revision FROM categories ORDER BY name")
        .and_then(|mut statement| {
            statement
                .query_map([], |row| {
                    Ok(CatalogMaintenanceRecord {
                        entity_id: row.get(0)?,
                        target: "category",
                        label: row.get(1)?,
                        activity: activity_name(row.get(2)?),
                        revision: row.get(3)?,
                        active_product_count: None,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        });
    let products = connection
        .prepare("SELECT id, sku || ' — ' || name, active, revision FROM products ORDER BY name")
        .and_then(|mut statement| {
            statement
                .query_map([], |row| {
                    Ok(CatalogMaintenanceRecord {
                        entity_id: row.get(0)?,
                        target: "product",
                        label: row.get(1)?,
                        activity: activity_name(row.get(2)?),
                        revision: row.get(3)?,
                        active_product_count: None,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        });
    match categories.and_then(|mut categories: Vec<_>| {
        categories.extend(products?);
        Ok(categories)
    }) {
        Ok(records) => Ok(CatalogMaintenanceListResponse::Success { records }),
        Err(_) => Ok(CatalogMaintenanceListResponse::Error(persistence_error())),
    }
}

pub fn maintain_catalog(
    connection: &mut rusqlite::Connection,
    request: MaintainCatalogRequest,
) -> Result<CatalogMaintenanceResponse, String> {
    let (target, intent) = match (target(&request.target), intent(&request.intent)) {
        (Some(target), Some(intent)) if request.entity_id > 0 && request.expected_revision >= 0 => {
            (target, intent)
        }
        _ => return Ok(CatalogMaintenanceResponse::Error(validation_error())),
    };
    Ok(
        match catalog::MaintainCatalogUseCase::new(connection, SqliteCatalogRepository).execute(
            catalog::MaintainCatalogInput::new(
                target,
                request.entity_id,
                intent,
                request.expected_revision,
            ),
        ) {
            Ok(snapshot) => CatalogMaintenanceResponse::Success(CatalogMaintenanceRecord {
                entity_id: request.entity_id,
                target: target_name(target),
                label: String::new(),
                activity: activity(snapshot.activity),
                revision: snapshot.revision,
                active_product_count: None,
            }),
            Err(error) => CatalogMaintenanceResponse::Error(match error {
                catalog::MaintainCatalogError::InvalidPricing => validation_error(),
                catalog::MaintainCatalogError::LifecycleBlocked => CatalogMaintenanceError {
                    code: "lifecycle_blocked",
                    message: "This lifecycle change is not allowed.",
                },
                catalog::MaintainCatalogError::StaleCatalogRecord => CatalogMaintenanceError {
                    code: "stale_catalog_record",
                    message: "This catalog record changed. Reload and try again.",
                },
                catalog::MaintainCatalogError::MissingCatalogRecord => validation_error(),
                catalog::MaintainCatalogError::StaleCategorySchema => CatalogMaintenanceError {
                    code: "stale_category_schema",
                    message: "Category fields changed. Reload before saving this product.",
                },
                catalog::MaintainCatalogError::PersistenceFailure => persistence_error(),
            }),
        },
    )
}

pub fn edit_catalog(
    connection: &mut rusqlite::Connection,
    request: EditCatalogRequest,
) -> Result<CatalogMaintenanceResponse, String> {
    let (entity_id, target, input) = match request {
        EditCatalogRequest::Category {
            entity_id,
            expected_revision,
            name,
        } if entity_id > 0 && expected_revision >= 0 => (
            entity_id,
            "category",
            catalog::EditCatalogInput::category(entity_id, expected_revision, name),
        ),
        EditCatalogRequest::Product {
            entity_id,
            expected_revision,
            sku,
            name,
            purchase_price_centavos,
            sale_price_centavos,
            minimum_sale_price_centavos,
            low_stock_threshold,
            expected_category_revision,
            attribute_values,
        } if entity_id > 0 && expected_revision >= 0 && expected_category_revision >= 0 => (
            entity_id,
            "product",
            catalog::EditCatalogInput::product_with_threshold(
                entity_id,
                expected_revision,
                sku,
                name,
                purchase_price_centavos,
                sale_price_centavos,
                minimum_sale_price_centavos,
                low_stock_threshold,
                expected_category_revision,
                attribute_values
                    .into_iter()
                    .map(|value| catalog::AttributeValueInput {
                        definition_id: value.definition_id,
                        value: value.value,
                    })
                    .collect(),
            ),
        ),
        _ => return Ok(CatalogMaintenanceResponse::Error(validation_error())),
    };
    Ok(
        match catalog::EditCatalogUseCase::new(connection, SqliteCatalogRepository).execute(input) {
            Ok(snapshot) => CatalogMaintenanceResponse::Success(CatalogMaintenanceRecord {
                entity_id,
                target,
                label: String::new(),
                activity: activity(snapshot.activity),
                revision: snapshot.revision,
                active_product_count: None,
            }),
            Err(error) => CatalogMaintenanceResponse::Error(map_maintenance_error(error)),
        },
    )
}

pub fn edit_category_schema(
    connection: &mut rusqlite::Connection,
    request: EditCategorySchemaRequest,
) -> CatalogMaintenanceResponse {
    if request.category_id <= 0 || request.expected_revision < 0 {
        return CatalogMaintenanceResponse::Error(validation_error());
    }
    let fields = request.fields.into_iter().map(|field| {
        let field_type = FieldType::parse(&field.field_type).map_err(|_| ())?;
        Ok::<_, ()>(CategorySchemaField {
            definition_id: field.definition_id,
            label: field.label,
            field_type,
            required: field.required,
            options: field.options,
        })
    }).collect::<Result<Vec<_>, _>>();
    let Ok(fields) = fields else {
        return CatalogMaintenanceResponse::Error(validation_error());
    };
    match catalog::EditCategorySchemaUseCase::new(connection, SqliteCatalogRepository).execute(
        catalog::EditCategorySchemaInput {
            category_id: request.category_id,
            expected_revision: request.expected_revision,
            fields,
        },
    ) {
        Ok(snapshot) => CatalogMaintenanceResponse::Success(CatalogMaintenanceRecord {
            entity_id: request.category_id,
            target: "category",
            label: String::new(),
            activity: activity(snapshot.activity),
            revision: snapshot.revision,
            active_product_count: None,
        }),
        Err(error) => CatalogMaintenanceResponse::Error(match error {
            catalog::EditCategorySchemaError::MissingCategory => unavailable_error(),
            catalog::EditCategorySchemaError::StaleCategory => CatalogMaintenanceError {
                code: "stale_category_schema",
                message: "Category fields changed. Reload before saving products or editing fields.",
            },
            catalog::EditCategorySchemaError::ImmutableField | catalog::EditCategorySchemaError::InvalidSchema => validation_error(),
            catalog::EditCategorySchemaError::PersistenceFailure => persistence_error(),
        }),
    }
}

pub fn catalog_metadata_detail(
    connection: &rusqlite::Connection,
    request: CatalogMetadataDetailRequest,
) -> Result<CatalogMetadataDetailResponse, String> {
    let Some(target) = target(&request.target).filter(|_| request.entity_id > 0) else {
        return Ok(CatalogMetadataDetailResponse::Error(validation_error()));
    };
    Ok(
        match catalog::read_catalog_metadata_detail(connection, target, request.entity_id) {
            Ok(Some(detail)) => CatalogMetadataDetailResponse::Success(detail),
            Ok(None) => CatalogMetadataDetailResponse::Error(unavailable_error()),
            Err(_) => CatalogMetadataDetailResponse::Error(persistence_error()),
        },
    )
}

fn target(value: &str) -> Option<CatalogTarget> {
    match value {
        "category" => Some(CatalogTarget::Category),
        "product" => Some(CatalogTarget::Product),
        _ => None,
    }
}
fn intent(value: &str) -> Option<CatalogIntent> {
    match value {
        "archive" => Some(CatalogIntent::Archive),
        "reactivate" => Some(CatalogIntent::Reactivate),
        _ => None,
    }
}
fn target_name(target: CatalogTarget) -> &'static str {
    match target {
        CatalogTarget::Category => "category",
        CatalogTarget::Product => "product",
    }
}
fn activity_name(value: i64) -> &'static str {
    if value == 1 {
        "active"
    } else {
        "archived"
    }
}
fn activity(value: CatalogActivity) -> &'static str {
    match value {
        CatalogActivity::Active => "active",
        CatalogActivity::Archived => "archived",
    }
}
fn browse_validation_error() -> CatalogBrowseError {
    CatalogBrowseError {
        code: "validation_error",
        message: "Review the product browse filters and try again.",
    }
}
fn browse_persistence_error() -> CatalogBrowseError {
    CatalogBrowseError {
        code: "persistence_failure",
        message: "The product catalog could not be loaded.",
    }
}
fn validation_error() -> CatalogMaintenanceError {
    CatalogMaintenanceError {
        code: "validation_error",
        message: "Review the catalog values and try again.",
    }
}
fn persistence_error() -> CatalogMaintenanceError {
    CatalogMaintenanceError {
        code: "persistence_failure",
        message: "The catalog could not be completed.",
    }
}
fn stale_catalog_error() -> CatalogMaintenanceError {
    CatalogMaintenanceError {
        code: "stale_catalog_record",
        message: "This catalog record changed. Reload and try again.",
    }
}
fn unavailable_error() -> CatalogMaintenanceError {
    CatalogMaintenanceError {
        code: "catalog_unavailable",
        message: "This catalog record is unavailable.",
    }
}
fn image_unavailable_error() -> CatalogMaintenanceError {
    CatalogMaintenanceError {
        code: "image_unavailable",
        message: "This product image is unavailable.",
    }
}
pub fn map_command_state_error(error: &str) -> CatalogMaintenanceError {
    if error == "database_unavailable" {
        unavailable_error()
    } else {
        persistence_error()
    }
}
fn map_maintenance_error(error: catalog::MaintainCatalogError) -> CatalogMaintenanceError {
    match error {
        catalog::MaintainCatalogError::InvalidPricing => validation_error(),
        catalog::MaintainCatalogError::LifecycleBlocked => CatalogMaintenanceError {
            code: "lifecycle_blocked",
            message: "This lifecycle change is not allowed.",
        },
        catalog::MaintainCatalogError::StaleCatalogRecord => CatalogMaintenanceError {
            code: "stale_catalog_record",
            message: "This catalog record changed. Reload and try again.",
        },
        catalog::MaintainCatalogError::MissingCatalogRecord => validation_error(),
        catalog::MaintainCatalogError::StaleCategorySchema => CatalogMaintenanceError {
            code: "stale_category_schema",
            message: "Category fields changed. Reload before saving this product.",
        },
        catalog::MaintainCatalogError::PersistenceFailure => persistence_error(),
    }
}
