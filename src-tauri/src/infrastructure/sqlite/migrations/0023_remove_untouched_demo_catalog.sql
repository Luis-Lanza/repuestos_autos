-- Remove only the historical demo rows that still match their untouched seed state.
-- Any dependency, changed value, or missing/mismatched seed fact excludes the product.
CREATE TEMP TABLE demo_catalog_cleanup_candidates (
    product_id INTEGER PRIMARY KEY,
    category_id INTEGER NOT NULL
);

INSERT INTO demo_catalog_cleanup_candidates (product_id, category_id)
SELECT p.id, c.id
FROM products p
JOIN categories c ON c.id = p.category_id
WHERE ((
        p.sku = 'FLT-001'
        AND p.name = 'Filtro de aceite'
        AND p.active = 1
        AND p.list_price_centavos = 2500
        AND p.minimum_unit_price_centavos = 2500
        AND p.purchase_price_centavos IS NULL
        AND p.revision = 0
        AND p.low_stock_threshold = 1
        AND p.primary_location_id IS NULL
        AND c.id = 1
        AND c.name = 'Filtros'
        AND c.active = 1
        AND c.revision = 0
        AND EXISTS (
            SELECT 1 FROM stock_balances b
            WHERE b.product_id = p.id AND b.quantity = 8
        )
        AND EXISTS (
            SELECT 1 FROM product_searchable_values s
            WHERE s.product_id = p.id
              AND s.field_name = 'vehicle'
              AND s.value = 'Toyota'
        )
        AND (SELECT COUNT(*) FROM product_searchable_values s WHERE s.product_id = p.id) = 1
        AND EXISTS (
            SELECT 1 FROM catalog_product_search f
            WHERE f.rowid = p.id AND f.product_id = p.id
              AND f.content = 'flt-001 filtro de aceite filtros toyota '
        )
        AND (SELECT COUNT(*) FROM catalog_product_search f WHERE f.product_id = p.id) = 1
    ) OR (
        p.sku = 'BUJ-001'
        AND p.name = 'Bujia archivada'
        AND p.active = 0
        AND p.list_price_centavos = 1800
        AND p.minimum_unit_price_centavos = 1800
        AND p.purchase_price_centavos IS NULL
        AND p.revision = 0
        AND p.low_stock_threshold = 1
        AND p.primary_location_id IS NULL
        AND c.id = 2
        AND c.name = 'Bujias'
        AND c.active = 1
        AND c.revision = 0
        AND EXISTS (
            SELECT 1 FROM stock_balances b
            WHERE b.product_id = p.id AND b.quantity = 4
        )
        AND NOT EXISTS (
            SELECT 1 FROM product_searchable_values s WHERE s.product_id = p.id
        )
        AND EXISTS (
            SELECT 1 FROM catalog_product_search f
            WHERE f.rowid = p.id AND f.product_id = p.id
              AND f.content = 'buj-001 bujia archivada bujias  '
        )
        AND (SELECT COUNT(*) FROM catalog_product_search f WHERE f.product_id = p.id) = 1
    ))
  AND (SELECT COUNT(*) FROM stock_balances b WHERE b.product_id = p.id) = 1
  AND NOT EXISTS (SELECT 1 FROM sale_lines l WHERE l.product_id = p.id)
  AND NOT EXISTS (SELECT 1 FROM inventory_movements m WHERE m.product_id = p.id)
  AND NOT EXISTS (SELECT 1 FROM product_attribute_values a WHERE a.product_id = p.id)
  AND NOT EXISTS (SELECT 1 FROM product_images i WHERE i.product_id = p.id)
  AND NOT EXISTS (
      SELECT 1 FROM catalog_audit a
      WHERE (a.entity_type = 'product' AND a.entity_id = p.id)
         OR (a.entity_type = 'category' AND a.entity_id = c.id)
  )
  AND NOT EXISTS (
      SELECT 1 FROM attribute_definitions d WHERE d.category_id = c.id
  );

DELETE FROM catalog_product_search
WHERE rowid IN (SELECT product_id FROM demo_catalog_cleanup_candidates);
DELETE FROM product_searchable_values
WHERE product_id IN (SELECT product_id FROM demo_catalog_cleanup_candidates);
DELETE FROM stock_balances
WHERE product_id IN (SELECT product_id FROM demo_catalog_cleanup_candidates);
DELETE FROM products
WHERE id IN (SELECT product_id FROM demo_catalog_cleanup_candidates);

-- A category is removed only when its exact seed identity remains untouched and
-- it has no remaining products or category-owned definitions/audit history.
DELETE FROM categories
WHERE (
        (id = 1 AND name = 'Filtros' AND active = 1 AND revision = 0)
        OR (id = 2 AND name = 'Bujias' AND active = 1 AND revision = 0)
    )
  AND id IN (SELECT category_id FROM demo_catalog_cleanup_candidates)
  AND NOT EXISTS (SELECT 1 FROM products p WHERE p.category_id = categories.id)
  AND NOT EXISTS (SELECT 1 FROM attribute_definitions d WHERE d.category_id = categories.id)
  AND NOT EXISTS (
      SELECT 1 FROM catalog_audit a
      WHERE a.entity_type = 'category' AND a.entity_id = categories.id
  );

DROP TABLE demo_catalog_cleanup_candidates;
