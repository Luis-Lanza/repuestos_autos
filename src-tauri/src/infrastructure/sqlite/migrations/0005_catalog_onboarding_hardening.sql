CREATE VIRTUAL TABLE catalog_product_search USING fts5(
    product_id UNINDEXED,
    content,
    tokenize = 'unicode61 remove_diacritics 2',
    prefix = '2 3 4'
);

INSERT INTO catalog_product_search (rowid, product_id, content)
SELECT p.id, p.id, lower(
    p.sku || ' ' || p.name || ' ' || c.name || ' ' ||
    COALESCE((SELECT group_concat(value, ' ') FROM product_searchable_values WHERE product_id = p.id), '') || ' ' ||
    COALESCE((SELECT group_concat(searchable_value, ' ') FROM product_attribute_values WHERE product_id = p.id), '')
)
FROM products p JOIN categories c ON c.id = p.category_id;

DROP TRIGGER inventory_movements_immutable_update;
DROP TRIGGER inventory_movements_immutable_delete;
ALTER TABLE inventory_movements RENAME TO inventory_movements_v4;
CREATE TABLE inventory_movements (
    id INTEGER PRIMARY KEY,
    product_id INTEGER NOT NULL REFERENCES products (id),
    sale_id INTEGER REFERENCES sales (id),
    sale_line_id INTEGER REFERENCES sale_lines (id),
    movement_type TEXT NOT NULL CHECK (movement_type IN ('opening_stock', 'stock_entry', 'sale', 'return', 'adjustment', 'cancellation')),
    quantity_delta INTEGER NOT NULL CHECK (quantity_delta <> 0),
    occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reason TEXT,
    operator_id TEXT,
    source_reference TEXT,
    CHECK ((movement_type <> 'sale') OR (quantity_delta < 0 AND sale_id IS NOT NULL AND sale_line_id IS NOT NULL))
);
INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, occurred_at)
SELECT id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, occurred_at FROM inventory_movements_v4;
DROP TABLE inventory_movements_v4;
CREATE INDEX inventory_movements_sale_id_idx ON inventory_movements (sale_id);
CREATE INDEX inventory_movements_product_id_idx ON inventory_movements (product_id);
CREATE TRIGGER inventory_movements_immutable_update BEFORE UPDATE ON inventory_movements BEGIN SELECT RAISE(ABORT, 'inventory movements are immutable'); END;
CREATE TRIGGER inventory_movements_immutable_delete BEFORE DELETE ON inventory_movements BEGIN SELECT RAISE(ABORT, 'inventory movements are immutable'); END;
