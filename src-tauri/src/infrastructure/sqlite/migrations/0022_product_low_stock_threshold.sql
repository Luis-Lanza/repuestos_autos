ALTER TABLE products
ADD COLUMN low_stock_threshold INTEGER NOT NULL DEFAULT 1
CHECK (typeof(low_stock_threshold) = 'integer' AND low_stock_threshold >= 1);
