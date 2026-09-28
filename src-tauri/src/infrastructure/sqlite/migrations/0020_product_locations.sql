CREATE TABLE location_schema (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0)
);
INSERT INTO location_schema (id) VALUES (1);

CREATE TABLE location_segments (
    id INTEGER PRIMARY KEY,
    schema_id INTEGER NOT NULL DEFAULT 1 REFERENCES location_schema(id),
    position INTEGER NOT NULL CHECK (position >= 0),
    label TEXT NOT NULL CHECK (length(trim(label)) BETWEEN 1 AND 40),
    UNIQUE (schema_id, position),
    UNIQUE (schema_id, label)
);

CREATE TABLE product_locations (
    id INTEGER PRIMARY KEY,
    code TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(code) BETWEEN 1 AND 328),
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0)
);

CREATE TABLE product_location_values (
    location_id INTEGER NOT NULL REFERENCES product_locations(id) ON DELETE CASCADE,
    segment_id INTEGER NOT NULL REFERENCES location_segments(id) ON DELETE RESTRICT,
    value TEXT NOT NULL CHECK (length(trim(value)) BETWEEN 1 AND 40),
    PRIMARY KEY (location_id, segment_id)
);

ALTER TABLE products ADD COLUMN primary_location_id INTEGER REFERENCES product_locations(id) ON DELETE RESTRICT;
CREATE INDEX products_primary_location_idx ON products(primary_location_id) WHERE primary_location_id IS NOT NULL;

CREATE TRIGGER location_segments_append_only_when_used
BEFORE INSERT ON location_segments
WHEN EXISTS (SELECT 1 FROM product_locations)
BEGIN
    SELECT RAISE(ABORT, 'location schema cannot change while locations exist');
END;
CREATE TRIGGER location_segments_immutable_when_used_update
BEFORE UPDATE ON location_segments
WHEN EXISTS (SELECT 1 FROM product_locations)
BEGIN
    SELECT RAISE(ABORT, 'location schema is immutable while locations exist');
END;
CREATE TRIGGER location_segments_immutable_when_used_delete
BEFORE DELETE ON location_segments
WHEN EXISTS (SELECT 1 FROM product_locations)
BEGIN
    SELECT RAISE(ABORT, 'location schema is immutable while locations exist');
END;

CREATE TRIGGER product_locations_code_immutable
BEFORE UPDATE OF code ON product_locations
BEGIN
    SELECT RAISE(ABORT, 'generated location code is immutable');
END;
CREATE TRIGGER product_location_values_immutable
BEFORE UPDATE ON product_location_values
BEGIN
    SELECT RAISE(ABORT, 'location segment values are immutable');
END;

CREATE TRIGGER product_locations_validate_delete
BEFORE DELETE ON product_locations
WHEN EXISTS (SELECT 1 FROM products WHERE primary_location_id = OLD.id)
BEGIN
    SELECT RAISE(ABORT, 'location is assigned to a product');
END;

CREATE TRIGGER products_primary_location_active_insert
BEFORE INSERT ON products
WHEN NEW.primary_location_id IS NOT NULL
 AND NOT EXISTS (SELECT 1 FROM product_locations WHERE id = NEW.primary_location_id AND active = 1)
BEGIN
    SELECT RAISE(ABORT, 'primary location must be active');
END;
CREATE TRIGGER products_primary_location_active_update
BEFORE UPDATE OF primary_location_id ON products
WHEN NEW.primary_location_id IS NOT NULL
 AND NOT EXISTS (SELECT 1 FROM product_locations WHERE id = NEW.primary_location_id AND active = 1)
BEGIN
    SELECT RAISE(ABORT, 'primary location must be active');
END;
CREATE TRIGGER product_locations_cannot_deactivate_assigned
BEFORE UPDATE OF active ON product_locations
WHEN NEW.active = 0 AND EXISTS (SELECT 1 FROM products WHERE primary_location_id = OLD.id)
BEGIN
    SELECT RAISE(ABORT, 'assigned location cannot be deactivated');
END;
