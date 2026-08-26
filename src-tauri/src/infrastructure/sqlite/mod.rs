use std::path::{Path, PathBuf};

use rusqlite::{Connection, Result};

pub mod catalog_repository;
pub mod sale_repository;

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

    if version > 5 {
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
