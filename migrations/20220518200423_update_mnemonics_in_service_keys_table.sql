-- Add migration script here

-- Add missing mnemonic column in service_keys table

BEGIN;

CREATE SEQUENCE service_keys_id_sequence;

ALTER TABLE
    service_keys
ALTER COLUMN
    id
SET
    DEFAULT nextval('service_keys_id_sequence');

ALTER TABLE
    service_keys
ADD
    COLUMN mnemonic TEXT NOT NULL UNIQUE;

COMMIT;