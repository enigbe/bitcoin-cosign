use crate::routes::{collect_xpub, create_user, gen_multisig_address, masterkeys};
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
            .route("/collect_xpubs", web::patch().to(collect_xpub))
            .route("/gen_multisig_addr", web::post().to(gen_multisig_address))
            .route("/masterkeys", web::post().to(masterkeys))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
