use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

/// Ping: test endpoint to check the server is running
async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}

/// Run the server
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/ping", web::get().to(ping)))
        .listen(listener)?
        .run();

    Ok(server)
}
