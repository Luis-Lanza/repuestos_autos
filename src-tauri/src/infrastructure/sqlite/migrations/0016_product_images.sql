CREATE TABLE product_images (
    product_id INTEGER PRIMARY KEY REFERENCES products(id) ON DELETE CASCADE,
    mime_type TEXT NOT NULL CHECK (mime_type IN ('image/png', 'image/jpeg', 'image/webp')),
    image_bytes BLOB NOT NULL CHECK (
        typeof(image_bytes) = 'blob'
        AND length(image_bytes) BETWEEN 1 AND 2097152
    )
);
