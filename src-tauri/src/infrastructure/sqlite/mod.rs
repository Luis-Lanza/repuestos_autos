use rusqlite::{Connection, Result};

pub fn open_seeded_catalog() -> Result<Connection> {
    let connection = Connection::open_in_memory()?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    connection.execute_batch(include_str!("migrations/0001_confirm_sale.sql"))?;
    Ok(connection)
}
