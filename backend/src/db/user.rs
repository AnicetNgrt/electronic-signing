use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    name: &str,
    is_admin: bool,
) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash, name, is_admin)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, password_hash, name, is_admin, created_at, updated_at
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(name)
    .bind(is_admin)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, name, is_admin, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, name, is_admin, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn update_user_password(pool: &PgPool, id: Uuid, password_hash: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $1
        WHERE id = $2
        "#,
    )
    .bind(password_hash)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn count_admin_users(pool: &PgPool) -> Result<i64> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE is_admin = true")
        .fetch_one(pool)
        .await?;

    Ok(count.0)
}
