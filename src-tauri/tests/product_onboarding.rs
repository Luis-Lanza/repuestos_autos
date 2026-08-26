use repuestos_autos::application::catalog::{
    create_category, create_product, AttributeValueInput, CategoryFieldInput, CreateCategoryInput,
    CreateProductInput,
};
use repuestos_autos::infrastructure::sqlite::open_seeded_catalog;

fn create_configured_category(connection: &mut rusqlite::Connection) -> i64 {
    create_category(
        connection,
        CreateCategoryInput {
            name: "Belts".into(),
            fields: vec![
                CategoryFieldInput { label: "Length".into(), field_type: "number".into(), required: true, options: vec![] },
                CategoryFieldInput { label: "Material".into(), field_type: "option".into(), required: false, options: vec!["Rubber".into(), "Polyurethane".into()] },
            ],
        },
    )
    .unwrap()
    .category_id
}

fn valid_product(category_id: i64, definitions: &[(i64, &str)]) -> CreateProductInput {
    CreateProductInput {
        sku: "BEL-101".into(),
        name: "Accessory belt".into(),
        category_id,
        catalog_unit_price_centavos: 4_500,
        opening_quantity: 6,
        attribute_values: definitions
            .iter()
            .map(|(definition_id, value)| AttributeValueInput {
                definition_id: *definition_id,
                value: (*value).into(),
            })
            .collect(),
    }
}

fn definition_ids(connection: &rusqlite::Connection, category_id: i64) -> Vec<i64> {
    let mut statement = connection
        .prepare("SELECT id FROM attribute_definitions WHERE category_id = ?1 ORDER BY id")
        .unwrap();
    statement
        .query_map([category_id], |row| row.get(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap()
}

#[test]
fn creates_product_attributes_balance_and_opening_movement_atomically() {
    let mut connection = open_seeded_catalog().unwrap();
    let category_id = create_configured_category(&mut connection);
    let definitions = definition_ids(&connection, category_id);

    let result = create_product(
        &mut connection,
        valid_product(
            category_id,
            &[(definitions[0], "1050.5"), (definitions[1], "Rubber")],
        ),
    )
    .unwrap();

    let persisted = connection
        .query_row(
            "SELECT p.active, p.minimum_unit_price_centavos, b.quantity, m.quantity_delta, m.movement_type, length(m.occurred_at) > 0, m.sale_id IS NULL, COUNT(v.definition_id) FROM products p JOIN stock_balances b ON b.product_id = p.id JOIN inventory_movements m ON m.product_id = p.id LEFT JOIN product_attribute_values v ON v.product_id = p.id WHERE p.id = ?1 GROUP BY p.id",
            [result.product_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?, row.get::<_, String>(4)?, row.get::<_, bool>(5)?, row.get::<_, bool>(6)?, row.get::<_, i64>(7)?)),
        )
        .unwrap();

    assert_eq!(
        persisted,
        (1, 4_500, 6, 6, "opening_stock".into(), true, true, 2)
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT content FROM catalog_product_search WHERE rowid = ?1",
                [result.product_id],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "bel-101 accessory belt belts 1050.5 rubber"
    );
    assert!(connection
        .execute(
            "UPDATE inventory_movements SET quantity_delta = 1 WHERE product_id = ?1",
            [result.product_id],
        )
        .is_err());
}
