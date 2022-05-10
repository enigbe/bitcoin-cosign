use crate::domain::{NewUser, User, UserEmail, UserPassword};
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

/// Create a new user and save record to database
/// The request body must be JSON and must contain an
/// email and a password field
/// e.g. {"email": "user@email.com", "password": "verysecret"}
pub async fn create_user(req: web::Json<User>, pool: web::Data<PgPool>) -> HttpResponse {
    // 1. create use
    // 1.1. TODO: hash password
    let new_user = match req.0.try_into() {
        Ok(user) => user,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    // 2. save record to DB
    match insert_user(&pool, &new_user).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Insert new user to database
/// ***
/// Parameters:
///     pool (&PgPool): A shared reference to a Postgres connection pool
///     new_user (&NewUser): A shared reference to a NewUser instance
pub async fn insert_user(pool: &PgPool, new_user: &NewUser) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        "#,
        new_user.email.as_ref(),
        new_user.password.as_ref()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        println!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

pub fn parse_user(req: User) -> Result<NewUser, String> {
    let email = UserEmail::parse(req.email)?;
    let password = UserPassword::parse(req.password)?;

    Ok(NewUser { email, password })
}
