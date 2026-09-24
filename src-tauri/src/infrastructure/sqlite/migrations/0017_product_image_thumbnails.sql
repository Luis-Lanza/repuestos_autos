ALTER TABLE product_images
    ADD COLUMN thumbnail_mime_type TEXT CHECK (
        thumbnail_mime_type IS NULL OR thumbnail_mime_type = 'image/jpeg'
    );

ALTER TABLE product_images
    ADD COLUMN thumbnail_bytes BLOB CHECK (
        (thumbnail_mime_type IS NULL AND thumbnail_bytes IS NULL)
        OR (
            thumbnail_mime_type IS NOT NULL
            AND typeof(thumbnail_bytes) = 'blob'
            AND length(thumbnail_bytes) BETWEEN 1 AND 1048576
        )
    );
