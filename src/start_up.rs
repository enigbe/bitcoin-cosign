use crate::routes::create_user;
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

/// Ping: test endpoint to check the server is running
async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}

/// Run the server
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/ping", web::get().to(ping))
            .route("/create_user", web::post().to(create_user))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
