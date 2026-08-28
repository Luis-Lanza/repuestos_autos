ALTER TABLE categories ADD COLUMN active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1));
ALTER TABLE categories ADD COLUMN revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0);
ALTER TABLE products ADD COLUMN revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0);

CREATE UNIQUE INDEX categories_normalized_name_idx ON categories (lower(trim(name)));
CREATE UNIQUE INDEX products_normalized_sku_idx ON products (lower(trim(sku)));
CREATE INDEX products_category_active_idx ON products (category_id, active);

CREATE TABLE catalog_audit (
    id INTEGER PRIMARY KEY,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('category', 'product')),
    entity_id INTEGER NOT NULL,
    operation TEXT NOT NULL,
    before_json TEXT NOT NULL,
    after_json TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0),
    occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX catalog_audit_entity_idx ON catalog_audit (entity_type, entity_id, id);
CREATE TRIGGER catalog_audit_immutable_update BEFORE UPDATE ON catalog_audit BEGIN SELECT RAISE(ABORT, 'catalog audit is immutable'); END;
CREATE TRIGGER catalog_audit_immutable_delete BEFORE DELETE ON catalog_audit BEGIN SELECT RAISE(ABORT, 'catalog audit is immutable'); END;

CREATE TRIGGER confirmed_sale_lines_immutable_price
BEFORE UPDATE OF product_id, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, line_total_centavos, sku_snapshot, product_name_snapshot ON sale_lines
WHEN (SELECT status FROM sales WHERE id = OLD.sale_id) = 'confirmed'
BEGIN SELECT RAISE(ABORT, 'confirmed sale lines are immutable'); END;
