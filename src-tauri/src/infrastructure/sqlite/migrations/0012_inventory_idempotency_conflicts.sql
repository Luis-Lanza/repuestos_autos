ALTER TABLE inventory_movements ADD COLUMN operation_kind TEXT;
ALTER TABLE inventory_movements ADD COLUMN payload_version INTEGER;
ALTER TABLE inventory_movements ADD COLUMN canonical_payload BLOB;
ALTER TABLE inventory_movements ADD COLUMN payload_sha256 TEXT;
