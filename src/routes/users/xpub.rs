use crate::domain::user_xpub::CollectXpub;
use crate::domain::UserXpubs;
use actix_web::{http::StatusCode, web, HttpResponse};
use sqlx::PgPool;

pub struct SavedUser {
    email: String,
}

#[derive(Debug, serde::Serialize)]
struct CollectXpubResponse {
    msg: String,
    status: u16,
}

/// Collect and save user-provided xpubs to database
pub async fn collect_xpub(req: web::Json<CollectXpub>, pool: web::Data<PgPool>) -> HttpResponse {
    // 1. Create UserXpubs
    let user_xpubs = match UserXpubs::try_from(req.0) {
        Ok(usr_xpbs) => usr_xpbs,
        Err(e) => {
            let rsp_msg = CollectXpubResponse {
                msg: format!("ERROR: Error parsing input. {}", e),
                status: StatusCode::BAD_REQUEST.as_u16(),
            };
            return HttpResponse::BadRequest().json(rsp_msg);
        }
    };
    // 2. Check if user email exists in DB
    // 2.1 If no record, return 40x
    let existing_user = match find_saved_user(&pool, &user_xpubs).await {
        Ok(saved_user) => saved_user,
        Err(e) => {
            let rsp_msg = CollectXpubResponse {
                msg: format!("ERROR: User record does not exist. {:?}", e),
                status: StatusCode::BAD_REQUEST.as_u16(),
            };
            return HttpResponse::Forbidden().json(rsp_msg);
        }
    };
    // 2.2 If a record exists, update the record with provided xpubs
    match update_user_xpubs(&pool, &user_xpubs).await {
        Ok(_) => {
            let rsp_msg = CollectXpubResponse {
                msg: format!(
                    "Extended public keys for user {} sucessfully uploaded",
                    existing_user.email
                ),
                status: StatusCode::OK.as_u16(),
            };
            HttpResponse::Ok().json(rsp_msg)
        }
        Err(e) => {
            let rsp_msg = CollectXpubResponse {
                msg: format!("ERROR: Error updating user record. {:?}", e),
                status: StatusCode::BAD_REQUEST.as_u16(),
            };
            return HttpResponse::BadRequest().json(rsp_msg);
        }
    }
}

/// Query database for saved record
/// /// ***
/// Parameters:
///     pool (&PgPool): A shared reference to a Postgres connection pool
///     user_xpub (&UserXpubs): A shared reference to a UserXpubs instance
pub async fn find_saved_user(
    pool: &PgPool,
    user_xpubs: &UserXpubs,
) -> Result<SavedUser, sqlx::Error> {
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
/// ***
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
