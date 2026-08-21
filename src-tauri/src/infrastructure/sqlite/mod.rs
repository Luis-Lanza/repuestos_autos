use std::path::{Path, PathBuf};

use rusqlite::{Connection, Result};

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
    let connection = Connection::open_in_memory()?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    connection.execute_batch(include_str!("migrations/0001_confirm_sale.sql"))?;
    Ok(connection)
}

fn migrate_if_needed(connection: &mut Connection) -> Result<()> {
    let version = connection.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))?;
    if version == 0 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("migrations/0001_confirm_sale.sql"))?;
        transaction.pragma_update(None, "user_version", 1)?;
        transaction.commit()?;
    }
    Ok(())
}
