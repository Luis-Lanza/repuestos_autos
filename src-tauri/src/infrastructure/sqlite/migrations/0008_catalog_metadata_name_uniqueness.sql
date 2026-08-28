CREATE UNIQUE INDEX products_normalized_name_idx ON products (lower(trim(name)));
