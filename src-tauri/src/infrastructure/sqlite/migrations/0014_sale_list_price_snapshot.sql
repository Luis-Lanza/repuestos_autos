-- Legacy confirmed lines intentionally retain NULL: their historical list price is unavailable.
ALTER TABLE sale_lines ADD COLUMN list_price_snapshot_centavos INTEGER
    CHECK (list_price_snapshot_centavos IS NULL OR list_price_snapshot_centavos > 0);

DROP TRIGGER confirmed_sale_lines_immutable_price;
CREATE TRIGGER confirmed_sale_lines_immutable_price
BEFORE UPDATE OF product_id, quantity, negotiated_unit_price_centavos,
    minimum_unit_price_snapshot_centavos, list_price_snapshot_centavos,
    line_total_centavos, sku_snapshot, product_name_snapshot ON sale_lines
WHEN (SELECT status FROM sales WHERE id = OLD.sale_id) = 'confirmed'
BEGIN
    SELECT RAISE(ABORT, 'confirmed sale lines are immutable');
END;
