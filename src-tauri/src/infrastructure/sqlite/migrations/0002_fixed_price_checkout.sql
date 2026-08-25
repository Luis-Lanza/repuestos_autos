-- Fixed-price checkout compatibility migration (schema version 2).
--
-- This migration intentionally performs no DDL or data updates. The migration
-- runner validates the version-1 physical tables and columns, runs
-- PRAGMA foreign_key_check, and then advances user_version inside its single
-- transaction. Existing negotiated and minimum snapshot columns remain the
-- compatible physical storage for legacy and fixed-price records.
SELECT 1;
