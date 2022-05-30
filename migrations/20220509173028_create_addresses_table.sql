-- Add migration script here

-- Create addresses table

BEGIN;

CREATE TABLE addresses(
	id INT,
	PRIMARY KEY (id),
	user_id INT NOT NULL,
	derivation_path TEXT NOT NULL,
	child_pubk_1 TEXT NOT NULL,
	child_pubk_2 TEXT NOT NULL,
	service_pubk TEXT NOT NULL,
	CONSTRAINT fk_users FOREIGN KEY(user_id) REFERENCES users(id)
);

COMMIT;