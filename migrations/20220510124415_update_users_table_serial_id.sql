-- Add migration script here

-- Create users table

BEGIN;

CREATE SEQUENCE users_id_sequence;

ALTER TABLE
	users
ALTER COLUMN
	id
SET
	DEFAULT nextval('users_id_sequence');

ALTER SEQUENCE users_id_sequence OWNED BY users.id;

COMMIT;