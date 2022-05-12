use cosign::configuration::get_configuration;
use cosign::start_up::run;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Server
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let addr = format!("127.0.0.1:{}", configuration.port);
    let listener = TcpListener::bind(addr).expect("Failed to bind random port");
    println!(
        "Starting bitcoin-cosign server on port: {}",
        configuration.port
    );

    run(listener, connection_pool)?.await
}
