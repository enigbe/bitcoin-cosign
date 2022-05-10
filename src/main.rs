use cosign::start_up::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    println!("Starting bitcoin-cosign server on port: {}", port);

    run(listener)?.await
}
