use std::path::{Path, PathBuf};

use rusqlite::{Connection, Result};

pub mod backup;
pub mod dashboard_repository;
pub mod catalog_repository;
pub mod inventory_repository;
pub mod post_sale_repository;
pub mod post_sale_transaction;
pub mod sale_history_repository;
pub mod sale_repository;

pub use backup::{
    create_snapshot, stage_and_validate, validate_restored_database, BackupValidationError,
    DatabaseMetadata,
};
pub use catalog_repository::SqliteCatalogRepository;
pub use inventory_repository::SqliteInventoryRepository;
pub use post_sale_repository::SqlitePostSaleRepository;
pub use post_sale_transaction::SqlitePostSaleTransactionFactory;

pub const CURRENT_SCHEMA_VERSION: i64 = 17;
const MAX_CATALOG_PRICE_CENTAVOS: i64 = 9_007_199_254_740_991;
const CATALOG_PRICE_SENTINEL: i64 = i64::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrationCompatibility {
    DuplicateNormalizedProductName,
}

pub fn migration_compatibility(connection: &Connection) -> Result<Option<MigrationCompatibility>> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM products GROUP BY lower(trim(name)) HAVING COUNT(*) > 1)",
            [],
            |row| row.get::<_, bool>(0),
        )
        .map(|duplicate| {
            duplicate.then_some(MigrationCompatibility::DuplicateNormalizedProductName)
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    path: PathBuf,
}

impl DatabaseConfig {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn production_database_config(app_data_directory: impl AsRef<Path>) -> DatabaseConfig {
    DatabaseConfig {
        path: app_data_directory.as_ref().join("repuestos-autos.sqlite3"),
    }
}

pub fn database_config(path: impl AsRef<Path>) -> DatabaseConfig {
    DatabaseConfig {
        path: path.as_ref().to_path_buf(),
    }
}

pub fn default_application_database_config() -> std::result::Result<DatabaseConfig, String> {
    let data_root = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        Some(PathBuf::from(data_home))
    } else {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
    }
    .ok_or_else(|| "cannot resolve the application data directory".to_string())?;
    Ok(production_database_config(
        data_root.join("com.repuestosautos.app"),
    ))
}

pub fn open_existing_database(
    config: &DatabaseConfig,
) -> std::result::Result<Connection, String> {
    if !config.path().is_file() {
        return Err("database file does not exist".into());
    }
    let connection = Connection::open(config.path()).map_err(|_| "cannot open database")?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|_| "cannot enable foreign keys")?;
    validate_restored_database(&connection).map_err(|_| "database schema or integrity is invalid")?;
    Ok(connection)
}

pub fn open_database(
    config: &DatabaseConfig,
) -> std::result::Result<Connection, Box<dyn std::error::Error>> {
    if let Some(parent) = config.path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut connection = Connection::open(config.path())?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrate_if_needed(&mut connection)?;
    Ok(connection)
}

pub fn open_seeded_catalog() -> Result<Connection> {
    let mut connection = Connection::open_in_memory()?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrate_if_needed(&mut connection)?;
    Ok(connection)
}

fn migrate_if_needed(connection: &mut Connection) -> Result<()> {
    let mut version =
        connection.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))?;

    if version > CURRENT_SCHEMA_VERSION {
        return Err(rusqlite::Error::InvalidQuery);
    }

    if version == 0 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("migrations/0001_confirm_sale.sql"))?;
        transaction.pragma_update(None, "user_version", 1)?;
        transaction.commit()?;
        version = 1;
    }

    if version == 1 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("migrations/0002_fixed_price_checkout.sql"))?;
        validate_version_one_schema(&transaction)?;
        transaction.pragma_update(None, "user_version", 2)?;
        transaction.commit()?;
        version = 2;
    }

    if version == 2 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!(
            "migrations/0003_sale_line_product_snapshots.sql"
        ))?;
        transaction.pragma_update(None, "user_version", 3)?;
        transaction.commit()?;
        version = 3;
    }

    if version == 3 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("migrations/0004_product_onboarding.sql"))?;
        transaction.pragma_update(None, "user_version", 4)?;
        transaction.commit()?;
        version = 4;
    }

    if version == 4 {
        let transaction = connection.transaction()?;
        validate_version_four_schema(&transaction)?;
        transaction.execute_batch(include_str!(
            "migrations/0005_catalog_onboarding_hardening.sql"
        ))?;
        validate_foreign_keys(&transaction)?;
        transaction.pragma_update(None, "user_version", 5)?;
        transaction.commit()?;
        version = 5;
    }

    if version == 5 {
        let transaction = connection.transaction()?;
        validate_version_five_schema(&transaction)?;
        transaction.execute_batch(include_str!(
            "migrations/0006_operational_inventory_control.sql"
        ))?;
        validate_foreign_keys(&transaction)?;
        transaction.pragma_update(None, "user_version", 6)?;
        transaction.commit()?;
        version = 6;
    }

    if version == 6 {
        let transaction = connection.transaction()?;
        validate_version_six_schema(&transaction)?;
        transaction.execute_batch(include_str!("migrations/0007_catalog_maintenance.sql"))?;
        validate_version_seven_schema(&transaction)?;
        transaction.pragma_update(None, "user_version", 7)?;
        transaction.commit()?;
        version = 7;
    }

    if version == 7 {
        let transaction = connection.transaction()?;
        validate_version_seven_schema(&transaction)?;
        if migration_compatibility(&transaction)?.is_some() {
            return Err(rusqlite::Error::InvalidQuery);
        }
        transaction.execute_batch(include_str!(
            "migrations/0008_catalog_metadata_name_uniqueness.sql"
        ))?;
        transaction.pragma_update(None, "user_version", 8)?;
        transaction.commit()?;
        version = 8;
    }

    if version == 8 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("migrations/0009_sales_history_index.sql"))?;
        transaction.pragma_update(None, "user_version", 9)?;
        transaction.commit()?;
        version = 9;
    }

    if version == 9 {
        let transaction = connection.transaction()?;
        validate_version_six_schema(&transaction)?;
        transaction.execute_batch(include_str!("migrations/0010_post_sale_lifecycle.sql"))?;
        validate_version_ten_schema(&transaction)?;
        transaction.pragma_update(None, "user_version", 10)?;
        transaction.commit()?;
        version = 10;
    }

    if version == 10 {
        let transaction = connection.transaction()?;
        validate_version_ten_schema(&transaction)?;
        transaction.execute_batch(include_str!(
            "migrations/0011_sale_idempotency_conflicts.sql"
        ))?;
        validate_version_eleven_schema(&transaction)?;
        transaction.pragma_update(None, "user_version", 11)?;
        transaction.commit()?;
        version = 11;
    }

    if version == 11 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!(
            "migrations/0012_inventory_idempotency_conflicts.sql"
        ))?;
        validate_version_twelve_schema(&transaction)?;
        transaction.pragma_update(None, "user_version", 12)?;
        transaction.commit()?;
        version = 12;
    }

    if version == 12 {
        let transaction = connection.transaction()?;
        validate_version_twelve_schema(&transaction)?;
        if transaction.query_row(
            "SELECT EXISTS (SELECT 1 FROM products WHERE minimum_unit_price_centavos <= 0)",
            [],
            |row| row.get::<_, bool>(0),
        )? {
            return Err(rusqlite::Error::InvalidQuery);
        }
        transaction.execute_batch(include_str!(
            "migrations/0013_catalog_dual_pricing.sql"
        ))?;
        validate_version_thirteen_schema(&transaction)?;
        transaction.pragma_update(None, "user_version", 13)?;
        transaction.commit()?;
        version = 13;
    }

    if version == 13 {
        let transaction = connection.transaction()?;
        validate_version_thirteen_schema(&transaction)?;
        transaction.execute_batch(include_str!(
            "migrations/0014_sale_list_price_snapshot.sql"
        ))?;
        validate_version_fourteen_schema(&transaction)?;
        transaction.pragma_update(None, "user_version", 14)?;
        transaction.commit()?;
        version = 14;
    }

    if version == 14 {
        let transaction = connection.transaction()?;
        validate_version_fourteen_schema(&transaction)?;
        validate_catalog_price_data(&transaction)?;
        transaction.execute_batch(include_str!(
            "migrations/0015_catalog_price_cap.sql"
        ))?;
        validate_version_fifteen_schema(&transaction)?;
        transaction.pragma_update(None, "user_version", 15)?;
        transaction.commit()?;
        version = 15;
    }

    if version == 15 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("migrations/0016_product_images.sql"))?;
        transaction.pragma_update(None, "user_version", 16)?;
        validate_version_fifteen_schema(&transaction)?;
        transaction.commit()?;
        version = 16;
    }

    if version == 16 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("migrations/0017_product_image_thumbnails.sql"))?;
        transaction.pragma_update(None, "user_version", 17)?;
        validate_version_fifteen_schema(&transaction)?;
        transaction.commit()?;
        version = 17;
    }

    if version == CURRENT_SCHEMA_VERSION {
        validate_version_fifteen_schema(connection)?;
    }

    Ok(())
}

fn validate_version_one_schema(connection: &Connection) -> Result<()> {
    const REQUIRED_COLUMNS: &[(&str, &[&str])] = &[
        ("categories", &["id", "name"]),
        (
            "products",
            &[
                "id",
                "category_id",
                "sku",
                "name",
                "active",
                "minimum_unit_price_centavos",
            ],
        ),
        (
            "product_searchable_values",
            &["product_id", "field_name", "value"],
        ),
        ("stock_balances", &["product_id", "quantity"]),
        (
            "sales",
            &[
                "id",
                "request_id",
                "status",
                "total_centavos",
                "confirmed_at",
            ],
        ),
        (
            "sale_lines",
            &[
                "id",
                "sale_id",
                "product_id",
                "quantity",
                "negotiated_unit_price_centavos",
                "minimum_unit_price_snapshot_centavos",
                "line_total_centavos",
            ],
        ),
        (
            "sale_payments",
            &[
                "id",
                "sale_id",
                "method",
                "amount_applied_centavos",
                "amount_tendered_centavos",
                "change_given_centavos",
            ],
        ),
        (
            "inventory_movements",
            &[
                "id",
                "product_id",
                "sale_id",
                "sale_line_id",
                "quantity_delta",
            ],
        ),
    ];

    for (table, columns) in REQUIRED_COLUMNS {
        let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
        let actual_columns = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>>>()?;
        if columns
            .iter()
            .any(|column| !actual_columns.iter().any(|actual| actual == column))
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }

    validate_foreign_keys(connection)
}

fn validate_version_four_schema(connection: &Connection) -> Result<()> {
    let mut statement = connection.prepare("PRAGMA table_info(inventory_movements)")?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;
    if [
        "id",
        "product_id",
        "sale_id",
        "sale_line_id",
        "movement_type",
        "quantity_delta",
        "occurred_at",
    ]
    .iter()
    .any(|column| !columns.iter().any(|actual| actual == column))
    {
        return Err(rusqlite::Error::InvalidQuery);
    }
    validate_foreign_keys(connection)
}

fn validate_version_five_schema(connection: &Connection) -> Result<()> {
    validate_version_four_schema(connection)?;
    let mut statement = connection.prepare("PRAGMA table_info(inventory_movements)")?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;
    if ["reason", "operator_id", "source_reference"]
        .iter()
        .any(|column| !columns.iter().any(|actual| actual == column))
    {
        return Err(rusqlite::Error::InvalidQuery);
    }
    if connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM inventory_movements m LEFT JOIN sale_lines l ON l.id = m.sale_line_id AND l.sale_id = m.sale_id AND l.product_id = m.product_id WHERE NOT ((m.movement_type = 'opening_stock' AND m.quantity_delta > 0 AND m.sale_id IS NULL AND m.sale_line_id IS NULL) OR (m.movement_type = 'sale' AND m.quantity_delta < 0 AND m.sale_id IS NOT NULL AND m.sale_line_id IS NOT NULL AND l.id IS NOT NULL)))",
        [],
        |row| row.get::<_, bool>(0),
    )? {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(())
}

fn validate_version_six_schema(connection: &Connection) -> Result<()> {
    let mut statement = connection.prepare("PRAGMA table_info(inventory_movements)")?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;
    if [
        "id",
        "product_id",
        "sale_id",
        "sale_line_id",
        "movement_type",
        "quantity_delta",
        "occurred_at",
        "reason",
        "operator_id",
        "source_reference",
        "request_id",
        "counted_quantity",
        "resulting_quantity",
    ]
    .iter()
    .any(|column| !columns.iter().any(|actual| actual == column))
    {
        return Err(rusqlite::Error::InvalidQuery);
    }
    if connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM inventory_movements WHERE NOT ((movement_type = 'opening_stock' AND quantity_delta > 0 AND sale_id IS NULL AND sale_line_id IS NULL) OR (movement_type = 'stock_entry' AND quantity_delta > 0 AND sale_id IS NULL AND sale_line_id IS NULL AND request_id IS NOT NULL AND trim(request_id) <> '' AND resulting_quantity IS NOT NULL AND resulting_quantity >= 0) OR (movement_type = 'sale' AND quantity_delta < 0 AND sale_id IS NOT NULL AND sale_line_id IS NOT NULL) OR (movement_type = 'return' AND quantity_delta > 0 AND sale_id IS NOT NULL AND sale_line_id IS NOT NULL) OR (movement_type = 'adjustment' AND quantity_delta <> 0 AND sale_id IS NULL AND sale_line_id IS NULL AND reason IS NOT NULL AND trim(reason) <> '' AND request_id IS NOT NULL AND trim(request_id) <> '' AND counted_quantity IS NOT NULL AND resulting_quantity IS NOT NULL AND counted_quantity >= 0 AND counted_quantity = resulting_quantity) OR (movement_type = 'cancellation' AND quantity_delta > 0 AND sale_id IS NOT NULL AND sale_line_id IS NOT NULL AND reason IS NOT NULL AND trim(reason) <> '')))",
        [],
        |row| row.get::<_, bool>(0),
    )? {
        return Err(rusqlite::Error::InvalidQuery);
    }
    for trigger in [
        "inventory_movements_immutable_update",
        "inventory_movements_immutable_delete",
    ] {
        let exists = connection.query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'trigger' AND name = ?1)",
            [trigger],
            |row| row.get::<_, bool>(0),
        )?;
        if !exists {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }
    validate_foreign_keys(connection)
}

pub(super) fn validate_version_ten_schema(connection: &Connection) -> Result<()> {
    validate_version_six_schema(connection)?;
    const REQUIRED_COLUMNS: &[(&str, &[&str])] = &[
        (
            "post_sale_requests",
            &[
                "id",
                "request_id",
                "operation_kind",
                "sale_id",
                "payload_version",
                "canonical_payload",
                "payload_sha256",
                "created_at",
            ],
        ),
        (
            "sale_returns",
            &["id", "sale_id", "operation_kind", "occurred_at"],
        ),
        (
            "sale_return_lines",
            &[
                "return_id",
                "sale_id",
                "sale_line_id",
                "product_id",
                "quantity",
                "movement_id",
            ],
        ),
        (
            "sale_cancellations",
            &["id", "sale_id", "operation_kind", "reason", "occurred_at"],
        ),
        (
            "sale_cancellation_lines",
            &[
                "cancellation_id",
                "sale_id",
                "sale_line_id",
                "product_id",
                "restored_quantity",
                "movement_id",
            ],
        ),
    ];
    for (table, required) in REQUIRED_COLUMNS {
        if !has_columns(connection, table, required)? {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }

    for index in [
        "sale_lines_identity_idx",
        "post_sale_requests_sale_created_idx",
        "sale_return_lines_sale_line_idx",
        "sale_cancellation_lines_sale_line_idx",
    ] {
        if !schema_object_exists(connection, "index", index)? {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }
    for table in [
        "post_sale_requests",
        "sale_returns",
        "sale_return_lines",
        "sale_cancellations",
        "sale_cancellation_lines",
    ] {
        for action in ["update", "delete"] {
            if !schema_object_exists(
                connection,
                "trigger",
                &format!("{table}_immutable_{action}"),
            )? {
                return Err(rusqlite::Error::InvalidQuery);
            }
        }
    }
    for trigger in [
        "confirmed_sale_lines_immutable_price",
        "sale_return_lines_validate_insert",
        "sale_cancellation_lines_validate_insert",
    ] {
        if !schema_object_exists(connection, "trigger", trigger)? {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }

    const INVALID_LIFECYCLE_FACTS: &str = "
        SELECT EXISTS (
            SELECT 1
            FROM post_sale_requests request
            LEFT JOIN sale_returns returned
              ON returned.id = request.id
             AND returned.sale_id = request.sale_id
             AND returned.operation_kind = 'return'
            LEFT JOIN sale_cancellations cancelled
              ON cancelled.id = request.id
             AND cancelled.sale_id = request.sale_id
             AND cancelled.operation_kind = 'cancellation'
            WHERE (request.operation_kind = 'return' AND returned.id IS NULL)
               OR (request.operation_kind = 'cancellation' AND cancelled.id IS NULL)
               OR (request.operation_kind NOT IN ('return', 'cancellation'))

            UNION ALL

            SELECT 1
            FROM sale_return_lines line
            LEFT JOIN inventory_movements movement ON movement.id = line.movement_id
            WHERE movement.id IS NULL
               OR movement.product_id <> line.product_id
               OR movement.sale_id <> line.sale_id
               OR movement.sale_line_id <> line.sale_line_id
               OR movement.movement_type <> 'return'
               OR movement.quantity_delta <> line.quantity
               OR movement.quantity_delta <= 0

            UNION ALL

            SELECT 1
            FROM sale_cancellation_lines line
            LEFT JOIN inventory_movements movement ON movement.id = line.movement_id
            WHERE (line.restored_quantity = 0 AND line.movement_id IS NOT NULL)
               OR (line.restored_quantity > 0 AND (
                    movement.id IS NULL
                    OR movement.product_id <> line.product_id
                    OR movement.sale_id <> line.sale_id
                    OR movement.sale_line_id <> line.sale_line_id
                    OR movement.movement_type <> 'cancellation'
                    OR movement.quantity_delta <> line.restored_quantity
                    OR movement.quantity_delta <= 0
               ))

            UNION ALL

            SELECT 1
            FROM sale_lines line
            WHERE line.quantity < COALESCE((
                SELECT SUM(returned.quantity)
                FROM sale_return_lines returned
                WHERE returned.sale_line_id = line.id
            ), 0) + COALESCE((
                SELECT SUM(cancelled.restored_quantity)
                FROM sale_cancellation_lines cancelled
                WHERE cancelled.sale_line_id = line.id
            ), 0)

            UNION ALL

            SELECT 1
            FROM sale_cancellations cancellation
            JOIN sale_lines line ON line.sale_id = cancellation.sale_id
            LEFT JOIN sale_cancellation_lines cancellation_line
              ON cancellation_line.cancellation_id = cancellation.id
             AND cancellation_line.sale_line_id = line.id
            WHERE cancellation_line.sale_line_id IS NULL
        )";
    if connection.query_row(INVALID_LIFECYCLE_FACTS, [], |row| row.get::<_, bool>(0))? {
        return Err(rusqlite::Error::InvalidQuery);
    }
    validate_foreign_keys(connection)?;
    let version = connection.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))?;
    if version >= 11
        && !has_columns(
            connection,
            "sales",
            &[
                "operation_kind",
                "payload_version",
                "canonical_payload",
                "payload_sha256",
            ],
        )?
    {
        return Err(rusqlite::Error::InvalidQuery);
    }
    if version >= 12
        && !has_columns(
            connection,
            "inventory_movements",
            &[
                "operation_kind",
                "payload_version",
                "canonical_payload",
                "payload_sha256",
            ],
        )?
    {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(())
}

fn validate_version_eleven_schema(connection: &Connection) -> Result<()> {
    validate_version_ten_schema(connection)?;
    if !has_columns(
        connection,
        "sales",
        &[
            "operation_kind",
            "payload_version",
            "canonical_payload",
            "payload_sha256",
        ],
    )? {
        return Err(rusqlite::Error::InvalidQuery);
    }
    validate_foreign_keys(connection)
}

pub(super) fn validate_version_thirteen_schema(connection: &Connection) -> Result<()> {
    validate_version_twelve_schema(connection)?;
    if !has_columns(
        connection,
        "products",
        &["list_price_centavos", "minimum_sale_price_centavos"],
    )? {
        return Err(rusqlite::Error::InvalidQuery);
    }
    if connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM products WHERE list_price_centavos <= 0 OR minimum_unit_price_centavos <= 0 OR minimum_unit_price_centavos > list_price_centavos)",
        [],
        |row| row.get::<_, bool>(0),
    )? {
        return Err(rusqlite::Error::InvalidQuery);
    }
    validate_foreign_keys(connection)
}

fn validate_version_fourteen_schema(connection: &Connection) -> Result<()> {
    validate_version_thirteen_schema(connection)?;
    if !has_columns(connection, "sale_lines", &["list_price_snapshot_centavos"])? {
        return Err(rusqlite::Error::InvalidQuery);
    }
    if connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM sale_lines WHERE list_price_snapshot_centavos IS NOT NULL AND list_price_snapshot_centavos <= 0)",
        [],
        |row| row.get::<_, bool>(0),
    )? {
        return Err(rusqlite::Error::InvalidQuery);
    }
    if !schema_object_exists(connection, "trigger", "confirmed_sale_lines_immutable_price")?
        || !connection.query_row(
            "SELECT COALESCE(sql, '') LIKE '%list_price_snapshot_centavos%' FROM sqlite_master WHERE type = 'trigger' AND name = 'confirmed_sale_lines_immutable_price'",
            [],
            |row| row.get::<_, bool>(0),
        )?
    {
        return Err(rusqlite::Error::InvalidQuery);
    }
    validate_foreign_keys(connection)
}

fn validate_version_fifteen_schema(connection: &Connection) -> Result<()> {
    validate_version_fourteen_schema(connection)?;
    validate_catalog_price_data(connection)?;
    for (name, event) in [
        (
            "products_validate_price_insert",
            "before insert on products",
        ),
        (
            "products_validate_price_update",
            "before update of list_price_centavos, minimum_unit_price_centavos on products",
        ),
    ] {
        if !schema_object_exists(connection, "trigger", name)? {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let sql = connection.query_row(
            "SELECT COALESCE(sql, '') FROM sqlite_master WHERE type = 'trigger' AND name = ?1",
            [name],
            |row| row.get::<_, String>(0),
        )?;
        let sql = sql.to_ascii_lowercase();
        if !sql.contains(event)
            || ![
                "raise(abort",
                "9007199254740991",
                "9223372036854775807",
                "new.list_price_centavos <= 0",
                "new.minimum_unit_price_centavos <= 0",
                "new.minimum_unit_price_centavos > new.list_price_centavos",
            ]
            .iter()
            .all(|fragment| sql.contains(fragment))
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }
    validate_foreign_keys(connection)?;
    let version = connection.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))?;
    if version >= 16 {
        validate_product_images_schema(connection)?;
    }
    if version >= 17 {
        validate_product_image_thumbnails_schema(connection)?;
    }
    Ok(())
}

fn validate_product_images_schema(connection: &Connection) -> Result<()> {
    let mut statement = connection.prepare("PRAGMA table_info(product_images)")?;
    let columns = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, i64>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>>>()?;
    if columns.get(..3)
        != Some(&[
            ("product_id".into(), "INTEGER".into(), false, 1),
            ("mime_type".into(), "TEXT".into(), true, 0),
            ("image_bytes".into(), "BLOB".into(), true, 0),
        ])
    {
        return Err(rusqlite::Error::InvalidQuery);
    }

    let foreign_key = connection.query_row(
        "SELECT \"table\", \"from\", \"to\", on_delete FROM pragma_foreign_key_list('product_images') WHERE \"from\" = 'product_id'",
        [],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        )),
    )?;
    if foreign_key != ("products".into(), "product_id".into(), "id".into(), "CASCADE".into()) {
        return Err(rusqlite::Error::InvalidQuery);
    }

    let sql = connection.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'product_images'",
        [],
        |row| row.get::<_, String>(0),
    )?;
    let actual = normalize_product_images_ddl(&sql);
    let expected = normalize_product_images_ddl(
        "CREATE TABLE product_images (
            product_id INTEGER PRIMARY KEY REFERENCES products(id) ON DELETE CASCADE,
            mime_type TEXT NOT NULL CHECK (mime_type IN ('image/png', 'image/jpeg', 'image/webp')),
            image_bytes BLOB NOT NULL CHECK (
                typeof(image_bytes) = 'blob'
                AND length(image_bytes) BETWEEN 1 AND 2097152
            )
        )",
    );
    if !actual.starts_with(expected.strip_suffix(')').unwrap_or(&expected)) {
        return Err(rusqlite::Error::InvalidQuery);
    }
    validate_foreign_keys(connection)
}

fn validate_product_image_thumbnails_schema(connection: &Connection) -> Result<()> {
    let mut statement = connection.prepare("PRAGMA table_info(product_images)")?;
    let columns = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>>>()?;
    if columns.len() != 5
        || columns.get(3) != Some(&("thumbnail_mime_type".into(), "TEXT".into(), false))
        || columns.get(4) != Some(&("thumbnail_bytes".into(), "BLOB".into(), false))
    {
        return Err(rusqlite::Error::InvalidQuery);
    }

    let sql = connection.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'product_images'",
        [],
        |row| row.get::<_, String>(0),
    )?;
    let actual = normalize_product_images_ddl(&sql);
    let expected = normalize_product_images_ddl(
        "CREATE TABLE product_images (
            product_id INTEGER PRIMARY KEY REFERENCES products(id) ON DELETE CASCADE,
            mime_type TEXT NOT NULL CHECK (mime_type IN ('image/png', 'image/jpeg', 'image/webp')),
            image_bytes BLOB NOT NULL CHECK (
                typeof(image_bytes) = 'blob'
                AND length(image_bytes) BETWEEN 1 AND 2097152
            ),
            thumbnail_mime_type TEXT CHECK (
                thumbnail_mime_type IS NULL OR thumbnail_mime_type = 'image/jpeg'
            ),
            thumbnail_bytes BLOB CHECK (
                (thumbnail_mime_type IS NULL AND thumbnail_bytes IS NULL)
                OR (
                    thumbnail_mime_type IS NOT NULL
                    AND typeof(thumbnail_bytes) = 'blob'
                    AND length(thumbnail_bytes) BETWEEN 1 AND 1048576
                )
            )
        )",
    );
    if actual != expected {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(())
}

fn normalize_product_images_ddl(sql: &str) -> String {
    let mut uncommented = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();
    let mut quote = None;

    while let Some(character) = chars.next() {
        if let Some(end_quote) = quote {
            uncommented.push(character);
            if character == end_quote {
                if chars.peek() == Some(&end_quote) {
                    uncommented.push(chars.next().expect("peeked character"));
                } else {
                    quote = None;
                }
            }
            continue;
        }

        if matches!(character, '\'' | '"' | '`') {
            quote = Some(character);
            uncommented.push(character);
        } else if character == '[' {
            quote = Some(']');
            uncommented.push(character);
        } else if character == '-' && chars.peek() == Some(&'-') {
            chars.next();
            for comment_char in chars.by_ref() {
                if comment_char == '\n' {
                    uncommented.push(' ');
                    break;
                }
            }
        } else if character == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut previous = None;
            for comment_char in chars.by_ref() {
                if previous == Some('*') && comment_char == '/' {
                    break;
                }
                previous = Some(comment_char);
            }
            uncommented.push(' ');
        } else {
            uncommented.push(character.to_ascii_lowercase());
        }
    }

    uncommented
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn validate_catalog_price_data(connection: &Connection) -> Result<()> {
    if connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM products WHERE list_price_centavos IS NULL OR list_price_centavos <= 0 OR list_price_centavos > ?1 OR list_price_centavos = ?2 OR minimum_unit_price_centavos IS NULL OR minimum_unit_price_centavos <= 0 OR minimum_unit_price_centavos > ?1 OR minimum_unit_price_centavos = ?2 OR minimum_unit_price_centavos > list_price_centavos)",
        [MAX_CATALOG_PRICE_CENTAVOS, CATALOG_PRICE_SENTINEL],
        |row| row.get::<_, bool>(0),
    )? {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(())
}

fn validate_version_twelve_schema(connection: &Connection) -> Result<()> {
    validate_version_eleven_schema(connection)?;
    if !has_columns(
        connection,
        "inventory_movements",
        &[
            "operation_kind",
            "payload_version",
            "canonical_payload",
            "payload_sha256",
        ],
    )? {
        return Err(rusqlite::Error::InvalidQuery);
    }
    validate_foreign_keys(connection)
}

fn has_columns(connection: &Connection, table: &str, required: &[&str]) -> Result<bool> {
    let mut statement = connection.prepare(&format!("PRAGMA table_xinfo({table})"))?;
    let actual = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;
    Ok(required
        .iter()
        .all(|column| actual.iter().any(|actual| actual == column)))
}

fn schema_object_exists(connection: &Connection, kind: &str, name: &str) -> Result<bool> {
    connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = ?1 AND name = ?2)",
        [kind, name],
        |row| row.get(0),
    )
}

fn validate_version_seven_schema(connection: &Connection) -> Result<()> {
    validate_version_six_schema(connection)?;
    for (table, columns) in [
        ("categories", &["active", "revision"][..]),
        ("products", &["revision"][..]),
        (
            "catalog_audit",
            &[
                "entity_type",
                "entity_id",
                "operation",
                "before_json",
                "after_json",
                "revision",
                "occurred_at",
            ][..],
        ),
    ] {
        let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
        let actual = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>>>()?;
        if columns
            .iter()
            .any(|column| !actual.iter().any(|item| item == column))
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }
    validate_foreign_keys(connection)
}

fn validate_foreign_keys(connection: &Connection) -> Result<()> {
    if connection
        .prepare("PRAGMA foreign_key_check")?
        .query([])?
        .next()?
        .is_some()
    {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(())
}
