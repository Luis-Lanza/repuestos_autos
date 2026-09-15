ALTER TABLE sales ADD COLUMN operation_kind TEXT;
ALTER TABLE sales ADD COLUMN payload_version INTEGER;
ALTER TABLE sales ADD COLUMN canonical_payload BLOB;
ALTER TABLE sales ADD COLUMN payload_sha256 TEXT;
