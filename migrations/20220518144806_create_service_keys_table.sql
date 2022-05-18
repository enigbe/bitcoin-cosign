-- Add migration script here

-- Create service_keys table

BEGIN;

CREATE TABLE IF NOT EXISTS service_keys(
    id INT,
    PRIMARY KEY (id),
    master_xpriv TEXT NOT NULL UNIQUE,
    master_xpub TEXT NOT NULL UNIQUE
);

COMMIT;