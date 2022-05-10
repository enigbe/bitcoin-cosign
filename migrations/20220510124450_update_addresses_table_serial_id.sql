-- Add migration script here

-- Create addresses table

BEGIN;

CREATE SEQUENCE addresses_id_sequence;

ALTER TABLE
	addresses
ALTER COLUMN
	id
SET
	DEFAULT nextval('addresses_id_sequence');

ALTER SEQUENCE addresses_id_sequence OWNED BY addresses.id;

COMMIT;