CREATE TABLE categories (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories (id),
    sku TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL, active INTEGER NOT NULL CHECK (active IN (0, 1)),
    minimum_unit_price_centavos INTEGER NOT NULL CHECK (
        minimum_unit_price_centavos >= 0
    )
);
CREATE TABLE product_searchable_values (
    product_id INTEGER NOT NULL REFERENCES products (id),
    field_name TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (product_id, field_name)
);
CREATE TABLE stock_balances (
    product_id INTEGER PRIMARY KEY REFERENCES products (id),
    quantity INTEGER NOT NULL CHECK (quantity >= 0)
);
CREATE INDEX products_active_name_idx ON products (active, name);
CREATE INDEX products_active_sku_idx ON products (active, sku);
CREATE INDEX product_searchable_values_value_idx ON product_searchable_values (
    value
);
CREATE TABLE sales (
    id INTEGER PRIMARY KEY,
    request_id TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL CHECK (status IN ('pending', 'confirmed')),
    total_centavos INTEGER NOT NULL CHECK (total_centavos >= 0),
    confirmed_at TEXT
);
CREATE TABLE sale_lines (
    id INTEGER PRIMARY KEY, sale_id INTEGER NOT NULL REFERENCES sales (id),
    product_id INTEGER NOT NULL REFERENCES products (id), quantity INTEGER NOT NULL CHECK (quantity > 0),
    negotiated_unit_price_centavos INTEGER NOT NULL CHECK (negotiated_unit_price_centavos >= 0),
    minimum_unit_price_snapshot_centavos INTEGER NOT NULL CHECK (minimum_unit_price_snapshot_centavos >= 0),
    line_total_centavos INTEGER NOT NULL CHECK (line_total_centavos >= 0)
);
CREATE TABLE sale_payments (
    id INTEGER PRIMARY KEY, sale_id INTEGER NOT NULL REFERENCES sales (id),
    method TEXT NOT NULL CHECK (method IN ('cash', 'qr')), amount_applied_centavos INTEGER NOT NULL CHECK (amount_applied_centavos >= 0),
    amount_tendered_centavos INTEGER, change_given_centavos INTEGER,
    CHECK ((method = 'cash' AND amount_tendered_centavos >= 0 AND change_given_centavos >= 0) OR (method = 'qr' AND amount_tendered_centavos IS NULL AND change_given_centavos IS NULL))
);
CREATE TABLE inventory_movements (
    id INTEGER PRIMARY KEY, product_id INTEGER NOT NULL REFERENCES products (id), sale_id INTEGER NOT NULL REFERENCES sales (id),
    sale_line_id INTEGER NOT NULL REFERENCES sale_lines (id), quantity_delta INTEGER NOT NULL CHECK (quantity_delta < 0)
);
CREATE INDEX sale_lines_sale_id_idx ON sale_lines (sale_id);
CREATE INDEX sale_payments_sale_id_idx ON sale_payments (sale_id);
CREATE INDEX inventory_movements_sale_id_idx ON inventory_movements (sale_id);
CREATE TRIGGER inventory_movements_immutable_update BEFORE UPDATE ON inventory_movements BEGIN SELECT RAISE(ABORT, 'inventory movements are immutable'); END;
CREATE TRIGGER inventory_movements_immutable_delete BEFORE DELETE ON inventory_movements BEGIN SELECT RAISE(ABORT, 'inventory movements are immutable'); END;
INSERT INTO categories (id, name) VALUES (1, 'Filtros'), (2, 'Bujias');
INSERT INTO products (
    id, category_id, sku, name, active, minimum_unit_price_centavos
) VALUES
(1, 1, 'FLT-001', 'Filtro de aceite', 1, 2500),
(2, 2, 'BUJ-001', 'Bujia archivada', 0, 1800);
INSERT INTO product_searchable_values (product_id, field_name, value) VALUES (
    1, 'vehicle', 'Toyota'
);
INSERT INTO stock_balances (product_id, quantity) VALUES (1, 8), (2, 4);
