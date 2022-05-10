use actix_web::{web, HttpRequest, HttpResponse};

#[derive(serde::Deserialize)]
pub struct User {
    email: String,
    password: String,
}

/// Create a new user and save record to database
/// The request body must be JSON and must contain an
/// email and a password field
/// e.g. {"email": "user@email.com", "password": "verysecret"}
pub async fn create_user(req: web::Json<User>) -> HttpResponse {
    HttpResponse::Created().finish()
}
