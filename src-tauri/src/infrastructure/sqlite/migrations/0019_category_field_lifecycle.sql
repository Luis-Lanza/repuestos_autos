CREATE TABLE attribute_definitions_replacement (
    id INTEGER PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories (id),
    label TEXT NOT NULL,
    field_type TEXT NOT NULL CHECK (field_type IN ('text', 'number', 'option')),
    required INTEGER NOT NULL CHECK (required IN (0, 1)),
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1))
);

INSERT INTO attribute_definitions_replacement (
    id, category_id, label, field_type, required, active
)
SELECT id, category_id, label, field_type, required, 1
FROM attribute_definitions;

DROP TABLE attribute_definitions;
ALTER TABLE attribute_definitions_replacement RENAME TO attribute_definitions;

CREATE UNIQUE INDEX attribute_definitions_active_category_label_idx
ON attribute_definitions (category_id, label)
WHERE active = 1;

CREATE INDEX attribute_definitions_category_active_idx
ON attribute_definitions (category_id, active, id);
