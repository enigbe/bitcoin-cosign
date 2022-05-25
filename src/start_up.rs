use crate::routes::{collect_xpub, create_user, gen_multisig_address, masterkeys, collect_trx_input};
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, http::StatusCode};
use sqlx::PgPool;
use std::net::TcpListener;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct PingResponse {
    msg: String,
    status: u16,
    data: Option<String>,
}

/// Ping: test endpoint to check the server is running
async fn ping() -> HttpResponse {
    let rsp = PingResponse {
        msg: "SUCCESS: Server is running".to_string(),
        status: StatusCode::OK.as_u16(),
        data: None,
    };
    HttpResponse::Ok().json(rsp)
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
            .route("/collect_trx_input", web::post().to(collect_trx_input))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
