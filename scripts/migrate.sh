# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER=${POSTGRES_USER:=postgres}
# Check if a custom password has been set, otherwise default to 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# Check if a custom database name has been set, otherwise default to 'newsletter'
DB_NAME="${POSTGRES_DB:=cosign}"
# Check if a custom port has been set, otherwise default to '5432'
DB_PORT="${POSTGRES_PORT:=5432}"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
# sqlx migrate add create_users_table
# sqlx migrate add create_addresses_table
# sqlx migrate add update_users_table_serial_id
# sqlx migrate add update_addresses_table_serial_id
# sqlx migrate add create_users_table_if_not_exist
# sqlx migrate add create_addresses_table_if_not_exist
# sqlx migrate add create_service_keys_table
# sqlx migrate add update_mnemonics_in_service_keys_table



