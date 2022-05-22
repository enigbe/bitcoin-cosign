-- Add migration script here

-- Add network to service_keys table

BEGIN;

ALTER TABLE
    service_keys
ADD
    COLUMN network TEXT NOT NULL UNIQUE;

COMMIT;