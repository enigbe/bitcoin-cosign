use crate::domain::UserXpubs;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

/// Collect and save user-provided xpubs to database
pub async fn collect_xpub(req: web::Json<UserXpubs>, pool: web::Data<PgPool>) -> HttpResponse {
    todo!()
}
