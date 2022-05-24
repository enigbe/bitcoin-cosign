use crate::domain::{NewUser, User, UserEmail, UserPassword};
use actix_web::{web, HttpResponse, http::StatusCode};
use sqlx::{PgPool};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserResponse {
    pub msg: String,
    pub status: u16,
    pub data: Option<UserEmail>,
}

/// Create a new user and save record to database
/// The request body must be JSON and must contain an
/// email and a password field
/// e.g. {"email": "user@email.com", "password": "verysecret"}
pub async fn create_user(req: web::Json<User>, pool: web::Data<PgPool>) -> HttpResponse {
    // 1. create user
    // 1.1. TODO: hash password
    let new_user = match req.0.try_into() {
        Ok(user) => user,
        Err(e) => {
            let rsp_msg = CreateUserResponse {
                msg: format!("ERROR: Unable to parse inputs. {:?}", e),
                status: StatusCode::BAD_REQUEST.as_u16(),
                data: None,
            };
            return HttpResponse::BadRequest().json(rsp_msg);
         },
    };

    // 2. save record to DB
    match insert_user(&pool, &new_user).await {
        Ok(_) => {
            let rsp_msg = CreateUserResponse {
                msg: format!("SUCCESS: User account created successfully"),
                status: StatusCode::CREATED.as_u16(),
                data: Some(new_user.email),
            };
            HttpResponse::Created().json(rsp_msg)
        },
        Err(e) => {
            println!("ERROR: {:?}", e);
            let rsp_msg = CreateUserResponse {
                msg: format!("ERROR: Error saving user to database. Duplicate email not permitted"),
                status: StatusCode::BAD_REQUEST.as_u16(),
                data: None,
            };
            return HttpResponse::BadRequest().json(rsp_msg);
        },
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
