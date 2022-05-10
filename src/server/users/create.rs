use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct User {
    email: String,
    password: String,
}

/// Create a new user and save record to database
/// The request body must be JSON and must contain an
/// email and a password field
/// e.g. {"email": "user@email.com", "password": "verysecret"}
pub async fn create_user(req: web::Json<User>, pool: web::Data<PgPool>) -> HttpResponse {
    // 1. validate email
    // 2. hash password
    // 3. save record to DB
    match sqlx::query!(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        "#,
        req.email,
        req.password
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
