-- Add migration script here

-- Create users table

BEGIN;

CREATE TABLE users(
	id INT,
	PRIMARY KEY (id),
	email TEXT NOT NULL UNIQUE,
	password_hash TEXT NOT NULL,
	xpub1 TEXT NULL,
	xpub2 TEXT NULL
);

COMMIT;