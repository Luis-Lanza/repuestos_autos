-- Historical sale lines remain unknown; only pending lines capture the current product cost.
ALTER TABLE sale_lines ADD COLUMN unit_cost_snapshot_centavos INTEGER CHECK (
    unit_cost_snapshot_centavos IS NULL
    OR (
        typeof(unit_cost_snapshot_centavos) = 'integer'
        AND unit_cost_snapshot_centavos BETWEEN 1 AND 9007199254740991
    )
);

DROP TRIGGER confirmed_sale_lines_immutable_price;
CREATE TRIGGER confirmed_sale_lines_immutable_price
BEFORE UPDATE OF product_id, quantity, negotiated_unit_price_centavos,
    minimum_unit_price_snapshot_centavos, list_price_snapshot_centavos,
    unit_cost_snapshot_centavos, line_total_centavos, sku_snapshot,
    product_name_snapshot ON sale_lines
WHEN (SELECT status FROM sales WHERE id = OLD.sale_id) = 'confirmed'
BEGIN
    SELECT RAISE(ABORT, 'confirmed sale lines are immutable');
END;

CREATE TRIGGER sale_lines_capture_unit_cost_snapshot
AFTER INSERT ON sale_lines
WHEN (SELECT status FROM sales WHERE id = NEW.sale_id) = 'pending'
BEGIN
    UPDATE sale_lines
    SET unit_cost_snapshot_centavos = (
        SELECT purchase_price_centavos FROM products WHERE id = NEW.product_id
    )
    WHERE id = NEW.id;
END;
