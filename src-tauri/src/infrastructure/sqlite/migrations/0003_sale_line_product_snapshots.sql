-- Product labels on confirmed sales are historical facts. Existing sale lines retain
-- NULL snapshots and readers fall back to the current catalog values for compatibility.
ALTER TABLE sale_lines ADD COLUMN sku_snapshot TEXT;
ALTER TABLE sale_lines ADD COLUMN product_name_snapshot TEXT;
