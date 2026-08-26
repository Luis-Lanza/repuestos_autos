DROP TRIGGER inventory_movements_immutable_update;
DROP TRIGGER inventory_movements_immutable_delete;
DROP INDEX inventory_movements_sale_id_idx;
DROP INDEX inventory_movements_product_id_idx;
ALTER TABLE inventory_movements RENAME TO inventory_movements_v5;

CREATE UNIQUE INDEX sale_lines_inventory_link_idx ON sale_lines (id, sale_id, product_id);
CREATE TABLE inventory_movements (
    id INTEGER PRIMARY KEY,
    product_id INTEGER NOT NULL REFERENCES products (id),
    sale_id INTEGER,
    sale_line_id INTEGER,
    movement_type TEXT NOT NULL CHECK (movement_type IN ('opening_stock', 'stock_entry', 'sale', 'return', 'adjustment', 'cancellation')),
    quantity_delta INTEGER NOT NULL,
    occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reason TEXT,
    operator_id TEXT,
    source_reference TEXT,
    request_id TEXT,
    counted_quantity INTEGER,
    resulting_quantity INTEGER,
    FOREIGN KEY (sale_line_id, sale_id, product_id) REFERENCES sale_lines (id, sale_id, product_id),
    CHECK (
        (movement_type = 'opening_stock' AND quantity_delta > 0 AND sale_id IS NULL AND sale_line_id IS NULL)
        OR (movement_type = 'stock_entry' AND quantity_delta > 0 AND sale_id IS NULL AND sale_line_id IS NULL AND request_id IS NOT NULL AND trim(request_id) <> '' AND resulting_quantity IS NOT NULL AND resulting_quantity >= 0)
        OR (movement_type = 'sale' AND quantity_delta < 0 AND sale_id IS NOT NULL AND sale_line_id IS NOT NULL)
        OR (movement_type = 'return' AND quantity_delta > 0 AND sale_id IS NOT NULL AND sale_line_id IS NOT NULL)
        OR (movement_type = 'adjustment' AND quantity_delta <> 0 AND sale_id IS NULL AND sale_line_id IS NULL AND reason IS NOT NULL AND trim(reason) <> '' AND request_id IS NOT NULL AND trim(request_id) <> '' AND counted_quantity IS NOT NULL AND resulting_quantity IS NOT NULL AND counted_quantity >= 0 AND counted_quantity = resulting_quantity)
        OR (movement_type = 'cancellation' AND quantity_delta > 0 AND sale_id IS NOT NULL AND sale_line_id IS NOT NULL AND reason IS NOT NULL AND trim(reason) <> '')
    )
);
INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, occurred_at, reason, operator_id, source_reference)
SELECT id, product_id, sale_id, sale_line_id, movement_type, quantity_delta, occurred_at, reason, operator_id, source_reference
FROM inventory_movements_v5;
DROP TABLE inventory_movements_v5;

CREATE INDEX inventory_movements_sale_id_idx ON inventory_movements (sale_id);
CREATE INDEX inventory_movements_product_id_idx ON inventory_movements (product_id);
CREATE UNIQUE INDEX inventory_movements_request_id_idx ON inventory_movements (request_id) WHERE request_id IS NOT NULL;
CREATE TRIGGER inventory_movements_immutable_update BEFORE UPDATE ON inventory_movements BEGIN SELECT RAISE(ABORT, 'inventory movements are immutable'); END;
CREATE TRIGGER inventory_movements_immutable_delete BEFORE DELETE ON inventory_movements BEGIN SELECT RAISE(ABORT, 'inventory movements are immutable'); END;
