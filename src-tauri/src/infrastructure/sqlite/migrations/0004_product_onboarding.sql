CREATE TABLE attribute_definitions (
    id INTEGER PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories (id),
    label TEXT NOT NULL,
    field_type TEXT NOT NULL CHECK (field_type IN ('text', 'number', 'option')),
    required INTEGER NOT NULL CHECK (required IN (0, 1)),
    UNIQUE (category_id, label)
);

CREATE TABLE attribute_options (
    definition_id INTEGER NOT NULL REFERENCES attribute_definitions (id),
    value TEXT NOT NULL,
    PRIMARY KEY (definition_id, value)
);

CREATE TABLE product_attribute_values (
    product_id INTEGER NOT NULL REFERENCES products (id),
    definition_id INTEGER NOT NULL REFERENCES attribute_definitions (id),
    text_value TEXT,
    number_value REAL,
    option_value TEXT,
    searchable_value TEXT NOT NULL,
    PRIMARY KEY (product_id, definition_id),
    CHECK (
        (text_value IS NOT NULL) +
        (number_value IS NOT NULL) +
        (option_value IS NOT NULL) = 1
    )
);

CREATE INDEX product_attribute_values_search_idx
ON product_attribute_values (searchable_value);

DROP TRIGGER IF EXISTS inventory_movements_immutable_update;
DROP TRIGGER IF EXISTS inventory_movements_immutable_delete;
DROP INDEX IF EXISTS inventory_movements_sale_id_idx;
ALTER TABLE inventory_movements RENAME TO inventory_movements_legacy;

CREATE TABLE inventory_movements (
    id INTEGER PRIMARY KEY,
    product_id INTEGER NOT NULL REFERENCES products (id),
    sale_id INTEGER REFERENCES sales (id),
    sale_line_id INTEGER REFERENCES sale_lines (id),
    movement_type TEXT NOT NULL CHECK (movement_type IN ('opening_stock', 'sale')),
    quantity_delta INTEGER NOT NULL,
    occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (
        (movement_type = 'opening_stock' AND quantity_delta > 0 AND sale_id IS NULL AND sale_line_id IS NULL)
        OR
        (movement_type = 'sale' AND quantity_delta < 0 AND sale_id IS NOT NULL AND sale_line_id IS NOT NULL)
    )
);

INSERT INTO inventory_movements (
    id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, occurred_at
)
SELECT
    m.id,
    m.product_id,
    m.sale_id,
    m.sale_line_id,
    'sale',
    m.quantity_delta,
    COALESCE(s.confirmed_at, CURRENT_TIMESTAMP)
FROM inventory_movements_legacy m
JOIN sales s ON s.id = m.sale_id;

DROP TABLE inventory_movements_legacy;

CREATE INDEX inventory_movements_sale_id_idx ON inventory_movements (sale_id);
CREATE INDEX inventory_movements_product_id_idx ON inventory_movements (product_id);
CREATE TRIGGER inventory_movements_immutable_update
BEFORE UPDATE ON inventory_movements
BEGIN
    SELECT RAISE(ABORT, 'inventory movements are immutable');
END;
CREATE TRIGGER inventory_movements_immutable_delete
BEFORE DELETE ON inventory_movements
BEGIN
    SELECT RAISE(ABORT, 'inventory movements are immutable');
END;
