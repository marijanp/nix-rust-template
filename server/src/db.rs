use crate::item::Item;
use crate::user_session::{UserIdToken, UserSession};

use chrono::{DateTime, Utc};
use openidconnect::{CsrfToken, Nonce};
use sqlx::{query, query_as, query_scalar, FromRow, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

#[derive(FromRow)]
struct DbItem {
    id: String,
    name: String,
    price: String,
}

impl From<DbItem> for Item {
    fn from(db_item: DbItem) -> Item {
        Item {
            id: Uuid::parse_str(&db_item.id).expect(""),
            name: db_item.name,
            price: db_item.price.parse::<u128>().expect(""),
        }
    }
}

pub async fn new_item(pool: &SqlitePool, name: &str, price: u128) -> sqlx::Result<Item> {
    let id = Uuid::new_v4().to_string();
    let price = price.to_string();
    query_as!(
        DbItem,
        "insert into items (id, name, price) values ($1, $2, $3) returning *",
        id,
        name,
        price
    )
    .fetch_one(pool)
    .await
    .map(Into::into)
}

/// Returns all known items
pub async fn get_items(pool: &SqlitePool) -> sqlx::Result<Vec<Item>> {
    query_as!(DbItem, "select * from items")
        .fetch_all(pool)
        .await
        .map(|items| items.into_iter().map(Into::into).collect())
}

pub async fn new_authentication_session(
    pool: &SqlitePool,
    csrf_token: &CsrfToken,
    nonce: &Nonce,
) -> sqlx::Result<()> {
    let csrf_token = csrf_token.secret();
    let nonce = nonce.secret();
    query!(
        "insert into authentication_sessions (csrf_token, nonce) values ($1, $2)",
        csrf_token,
        nonce,
    )
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn get_nonce_by_csrf_token(
    pool: &SqlitePool,
    csrf_token: &CsrfToken,
) -> sqlx::Result<Option<Nonce>> {
    let csrf_token = csrf_token.secret();
    query_scalar!(
        "select nonce from authentication_sessions where csrf_token = ?",
        csrf_token
    )
    .fetch_optional(pool)
    .await
    .map(|maybe_secret| maybe_secret.map(Nonce::new))
}

pub async fn delete_authentication_sessios_for(
    pool: &SqlitePool,
    csrf_token: &CsrfToken,
) -> sqlx::Result<()> {
    let csrf_token = csrf_token.secret(); // Clean up the auth session
    query!(
        "delete from authentication_sessions where csrf_token = ?",
        csrf_token
    )
    .execute(pool)
    .await
    .map(|_| ())
}

#[derive(FromRow)]
struct DbUserSession {
    id: String,
    id_token: String,
    nonce: String,
    expires_at: String,
}

impl From<DbUserSession> for UserSession {
    fn from(db_user_session: DbUserSession) -> UserSession {
        UserSession {
            id: Uuid::parse_str(&db_user_session.id).expect(""),
            id_token: UserIdToken::from_str(&db_user_session.id_token).expect(""),
            nonce: Nonce::new(db_user_session.nonce),
            expires_at: DateTime::parse_from_rfc3339(&db_user_session.expires_at)
                .expect("Failed to parse expires_at")
                .with_timezone(&Utc),
        }
    }
}

pub async fn new_user_session(
    pool: &SqlitePool,
    id_token: &UserIdToken,
    nonce: &Nonce,
    expires_at: &DateTime<Utc>,
) -> sqlx::Result<UserSession> {
    let id = Uuid::new_v4().to_string();
    let id_token = id_token.to_string();
    let nonce = nonce.secret();
    let expires_at = expires_at.to_rfc3339();
    query_as!(
        DbUserSession,
        r#"
        insert into user_sessions (id, id_token, nonce, expires_at)
        values ($1, $2, $3, $4)
        returning
            id as "id!: String",
            id_token as "id_token!: String",
            nonce as "nonce!: String",
            expires_at as "expires_at!: String"
        "#,
        id,
        id_token,
        *nonce,
        expires_at
    )
    .fetch_one(pool)
    .await
    .map(Into::into)
}

pub async fn get_user_session(
    pool: &SqlitePool,
    session_id: &str,
) -> sqlx::Result<Option<UserSession>> {
    query_as!(
        DbUserSession,
        r#"
        select
            id as "id!: String",
            id_token as "id_token!: String",
            nonce as "nonce!: String",
            expires_at as "expires_at!: String"
        from user_sessions
        where id = $1
        "#,
        session_id
    )
    .fetch_optional(pool)
    .await
    .map(|opt| opt.map(Into::into))
}
