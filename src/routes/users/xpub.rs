use crate::domain::{NewUser, UserEmail, UserXpubs};
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use crate::domain::user_xpub::CollectXpub;

pub struct SavedUser {
    email: String,
}

/// Collect and save user-provided xpubs to database
pub async fn collect_xpub(req: web::Json<CollectXpub>, pool: web::Data<PgPool>) -> HttpResponse {
    // 1. Create UserXpubs
    let user_xpubs = match  UserXpubs::try_from(req.0)  {
        Ok(usr_xpbs) => usr_xpbs,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    // 2. Check if user email exists in DB
    // 2.1 If no record, return 40x
    let _existing_user = match find_saved_user(&pool, &user_xpubs).await{
        Ok(saved_user) => saved_user,
        Err(_) => return HttpResponse::Forbidden().finish(),
    };
    // 2.2 If a record exists, update the record with provided xpubs
    match update_user_xpubs(&pool, & user_xpubs).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

/// Query database for saved record
/// /// ***
/// Parameters:
///     pool (&PgPool): A shared reference to a Postgres connection pool
///     user_xpub (&UserXpubs): A shared reference to a UserXpubs instance
pub async fn find_saved_user(pool: &PgPool, user_xpubs: &UserXpubs) -> Result<SavedUser, sqlx::Error> {
    let user = sqlx::query_as!(
        SavedUser,
        r#"
        SELECT email FROM users WHERE email = ($1)
        "#,
        user_xpubs.email.as_ref()
    )
        .fetch_one(pool)
        .await?;

    Ok(user)
}

/// Update database for saved user with provided xpubs
/// /// ***
/// Parameters:
///     pool (&PgPool): A shared reference to a Postgres connection pool
///     user_xpub (&UserXpubs): A shared reference to a UserXpubs instance
pub async fn update_user_xpubs(pool: &PgPool, user_xpubs: &UserXpubs) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE users
        SET xpub1 = ($1), xpub2 = ($2)
        WHERE email = ($3)
        "#,
        user_xpubs.xpub1.as_ref(),
        user_xpubs.xpub2.as_ref(),
        user_xpubs.email.as_ref()
    )
        .execute(pool)
        .await
        .map_err(|e| {
            println!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(())
}