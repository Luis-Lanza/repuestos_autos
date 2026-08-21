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
