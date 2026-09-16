-- Keep minimum_unit_price_centavos as the storage-compatible minimum-sale authority
-- for unchanged Sales consumers; list_price_centavos is the new explicit list price.
ALTER TABLE products ADD COLUMN list_price_centavos INTEGER NOT NULL DEFAULT 9223372036854775807 CHECK (list_price_centavos > 0);

UPDATE products
SET list_price_centavos = minimum_unit_price_centavos;

-- This generated compatibility alias lets unchanged Sales reads keep using the
-- legacy name without introducing a second writable minimum-price authority.
ALTER TABLE products ADD COLUMN minimum_sale_price_centavos INTEGER
    GENERATED ALWAYS AS (minimum_unit_price_centavos) VIRTUAL
    CHECK (minimum_sale_price_centavos > 0);
