ALTER TABLE products
    ADD COLUMN purchase_price_centavos INTEGER CHECK (
        purchase_price_centavos IS NULL
        OR (
            typeof(purchase_price_centavos) = 'integer'
            AND purchase_price_centavos BETWEEN 1 AND 9007199254740991
        )
    );

ALTER TABLE inventory_movements
    ADD COLUMN unit_purchase_price_centavos INTEGER CHECK (
        unit_purchase_price_centavos IS NULL
        OR (
            typeof(unit_purchase_price_centavos) = 'integer'
            AND unit_purchase_price_centavos BETWEEN 1 AND 9007199254740991
        )
    );
