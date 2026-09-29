use repuestos_autos::{
    application::inventory::movement_ledger::{
        MovementLedgerQuery, MovementLedgerReader, MovementType,
    },
    infrastructure::sqlite::{
        movement_ledger_repository::SqliteMovementLedgerReader, open_seeded_catalog,
    },
};
use rusqlite::Connection;

fn insert_movement(
    connection: &Connection,
    id: i64,
    movement_type: &str,
    occurred_at: &str,
    delta: i64,
    resulting: Option<i64>,
    reason: Option<&str>,
    note: Option<&str>,
) {
    let request_id = matches!(movement_type, "stock_entry" | "adjustment")
        .then(|| format!("movement-ledger-{id}"));
    let counted = (movement_type == "adjustment").then_some(resulting).flatten();
    connection.execute(
        "INSERT INTO inventory_movements
         (id, product_id, movement_type, quantity_delta, occurred_at, reason,
          source_reference, request_id, counted_quantity, resulting_quantity)
         VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, movement_type, delta, occurred_at, reason, note, request_id, counted, resulting],
    ).unwrap();
}

fn insert_sale_linked_movement(
    connection: &Connection,
    id: i64,
    sale_id: i64,
    sale_line_id: i64,
    movement_type: &str,
    source_reference: &str,
    reason: Option<&str>,
) {
    connection.execute(
        "INSERT INTO inventory_movements
         (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta,
          occurred_at, reason, source_reference)
         VALUES (?1, 1, ?2, ?3, ?4, 1, '2025-01-02 10:00:00', ?5, ?6)",
        rusqlite::params![id, sale_id, sale_line_id, movement_type, reason, source_reference],
    ).unwrap();
}

fn query(
    product_id: Option<i64>,
    movement_type: Option<MovementType>,
    page: u32,
    page_size: u32,
) -> MovementLedgerQuery {
    MovementLedgerQuery::new(
        "2025-01-01T00:00:00Z",
        "2025-01-03T00:00:00Z",
        product_id,
        movement_type,
        page,
        page_size,
    ).unwrap()
}

#[test]
fn rejects_fractional_bounds_that_sqlite_timestamp_text_cannot_preserve() {
    assert!(MovementLedgerQuery::new(
        "2025-01-01T00:00:00.1Z",
        "2025-01-02T00:00:00Z",
        None,
        None,
        1,
        50,
    ).is_err());
    assert!(MovementLedgerQuery::new(
        "2025-01-01T00:00:00Z",
        "2025-01-02T00:00:00.1Z",
        None,
        None,
        1,
        50,
    ).is_err());
}

#[test]
fn normalizes_offset_bounds_to_utc_without_changing_half_open_edges() {
    let connection = open_seeded_catalog().unwrap();
    insert_movement(&connection, 1, "opening_stock", "2025-01-01 00:00:00", 1, None, None, None);
    insert_movement(&connection, 2, "opening_stock", "2025-01-01 00:00:01", 1, None, None, None);
    let range = MovementLedgerQuery::new(
        "2025-01-01T01:00:00+01:00",
        "2025-01-01T01:00:01+01:00",
        None,
        None,
        1,
        50,
    ).unwrap();
    let result = SqliteMovementLedgerReader::new(&connection).list(&range).unwrap();
    assert_eq!(result.rows.iter().map(|row| row.movement_id).collect::<Vec<_>>(), [1]);
}

#[test]
fn ledger_shows_entry_notes_and_suppresses_generated_linkage_references() {
    let connection = open_seeded_catalog().unwrap();
    insert_movement(
        &connection,
        1,
        "stock_entry",
        "2025-01-02 10:00:00",
        2,
        Some(10),
        None,
        Some("Supplier delivery note"),
    );
    connection.execute(
        "INSERT INTO sales (id, request_id, status, total_centavos, confirmed_at)
         VALUES (71, 'movement-ledger-sale', 'confirmed', 100, '2025-01-02 09:00:00')",
        [],
    ).unwrap();
    connection.execute(
        "INSERT INTO sale_lines
         (id, sale_id, product_id, sku_snapshot, product_name_snapshot, quantity,
          negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos,
          list_price_snapshot_centavos, line_total_centavos)
         VALUES (81, 71, 1, 'FLT-001', 'Filtro de aceite', 1, 100, 100, 100, 100)",
        [],
    ).unwrap();
    insert_sale_linked_movement(
        &connection,
        2,
        71,
        81,
        "return",
        "return:71:81",
        None,
    );
    insert_sale_linked_movement(
        &connection,
        3,
        71,
        81,
        "cancellation",
        "cancellation:71:81",
        Some("Persisted cancellation reason"),
    );

    let page = SqliteMovementLedgerReader::new(&connection)
        .list(&query(None, None, 1, 10))
        .unwrap();
    let stock_entry = page.rows.iter().find(|row| row.movement_type == "stock_entry").unwrap();
    assert_eq!(stock_entry.note.as_deref(), Some("Supplier delivery note"));
    let returned = page.rows.iter().find(|row| row.movement_type == "return").unwrap();
    assert_eq!(returned.note, None);
    let cancellation = page.rows.iter().find(|row| row.movement_type == "cancellation").unwrap();
    assert_eq!(cancellation.reason.as_deref(), Some("Persisted cancellation reason"));
    assert_eq!(cancellation.note, None);
}

#[test]
fn ledger_filters_orders_paginates_and_keeps_nullable_persisted_facts_for_archived_products() {
    let connection = open_seeded_catalog().unwrap();
    insert_movement(&connection, 1, "opening_stock", "2025-01-01 10:00:00", 8, None, None, None);
    insert_movement(&connection, 2, "stock_entry", "2025-01-02 10:00:00", 3, Some(11), None, Some("stored note"));
    insert_movement(&connection, 3, "adjustment", "2025-01-02 10:00:00", -2, Some(9), Some("count reason"), None);
    connection.execute("UPDATE products SET active = 0 WHERE id = 1", []).unwrap();

    let reader = SqliteMovementLedgerReader::new(&connection);
    let products = reader.product_options().unwrap();
    let archived = products.iter().find(|product| product.product_id == 1).unwrap();
    assert_eq!(archived.product_name, "Filtro de aceite");
    assert!(!archived.active);
    let first_page = reader.list(&query(None, None, 1, 2)).unwrap();
    assert_eq!(first_page.rows.iter().map(|row| row.movement_id).collect::<Vec<_>>(), [3, 2]);
    assert!(first_page.has_more);
    assert_eq!((first_page.page, first_page.page_size), (1, 2));
    assert_eq!(first_page.rows[0].product_name, "Filtro de aceite");
    assert_eq!(first_page.rows[0].product_sku, "FLT-001");
    assert_eq!(first_page.rows[0].reason.as_deref(), Some("count reason"));
    assert_eq!(first_page.rows[1].note.as_deref(), Some("stored note"));

    let last_page = reader.list(&query(None, None, 2, 2)).unwrap();
    assert_eq!(last_page.rows.iter().map(|row| row.movement_id).collect::<Vec<_>>(), [1]);
    assert!(!last_page.has_more);
    assert_eq!(last_page.rows[0].resulting_quantity, None);

    let filtered = reader.list(&query(Some(1), Some(MovementType::StockEntry), 1, 10)).unwrap();
    assert_eq!(filtered.rows.iter().map(|row| row.movement_id).collect::<Vec<_>>(), [2]);
    let empty = reader.list(&query(Some(999), Some(MovementType::Sale), 1, 10)).unwrap();
    assert!(empty.rows.is_empty());
}
