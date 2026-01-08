use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::signer::{Signer, SignerStatus};

pub async fn create_signer(
    pool: &PgPool,
    document_id: Uuid,
    email: &str,
    name: &str,
    order_index: i32,
    access_token: &str,
) -> Result<Signer> {
    let signer = sqlx::query_as::<_, Signer>(
        r#"
        INSERT INTO signers (document_id, email, name, order_index, access_token)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, document_id, email, name, order_index, status, access_token,
                  ip_address, user_agent, viewed_at, signed_at, declined_at, decline_reason,
                  email_sent_at, created_at, updated_at
        "#,
    )
    .bind(document_id)
    .bind(email)
    .bind(name)
    .bind(order_index)
    .bind(access_token)
    .fetch_one(pool)
    .await?;

    Ok(signer)
}

pub async fn get_signer_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Signer>> {
    let signer = sqlx::query_as::<_, Signer>(
        r#"
        SELECT id, document_id, email, name, order_index, status, access_token,
               ip_address, user_agent, viewed_at, signed_at, declined_at, decline_reason,
               email_sent_at, created_at, updated_at
        FROM signers
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(signer)
}

pub async fn get_signer_by_access_token(pool: &PgPool, token: &str) -> Result<Option<Signer>> {
    let signer = sqlx::query_as::<_, Signer>(
        r#"
        SELECT id, document_id, email, name, order_index, status, access_token,
               ip_address, user_agent, viewed_at, signed_at, declined_at, decline_reason,
               email_sent_at, created_at, updated_at
        FROM signers
        WHERE access_token = $1
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(signer)
}

pub async fn get_signers_by_document(pool: &PgPool, document_id: Uuid) -> Result<Vec<Signer>> {
    let signers = sqlx::query_as::<_, Signer>(
        r#"
        SELECT id, document_id, email, name, order_index, status, access_token,
               ip_address, user_agent, viewed_at, signed_at, declined_at, decline_reason,
               email_sent_at, created_at, updated_at
        FROM signers
        WHERE document_id = $1
        ORDER BY order_index
        "#,
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;

    Ok(signers)
}

pub async fn update_signer_status(pool: &PgPool, id: Uuid, status: SignerStatus) -> Result<Signer> {
    let signer = sqlx::query_as::<_, Signer>(
        r#"
        UPDATE signers
        SET status = $1
        WHERE id = $2
        RETURNING id, document_id, email, name, order_index, status, access_token,
                  ip_address, user_agent, viewed_at, signed_at, declined_at, decline_reason,
                  email_sent_at, created_at, updated_at
        "#,
    )
    .bind(status)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(signer)
}

pub async fn mark_signer_viewed(
    pool: &PgPool,
    id: Uuid,
    ip_address: &str,
    user_agent: &str,
) -> Result<Signer> {
    let signer = sqlx::query_as::<_, Signer>(
        r#"
        UPDATE signers
        SET status = 'viewed', viewed_at = NOW(), ip_address = $1, user_agent = $2
        WHERE id = $3
        RETURNING id, document_id, email, name, order_index, status, access_token,
                  ip_address, user_agent, viewed_at, signed_at, declined_at, decline_reason,
                  email_sent_at, created_at, updated_at
        "#,
    )
    .bind(ip_address)
    .bind(user_agent)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(signer)
}

pub async fn mark_signer_signed(
    pool: &PgPool,
    id: Uuid,
    ip_address: &str,
    user_agent: &str,
) -> Result<Signer> {
    let signer = sqlx::query_as::<_, Signer>(
        r#"
        UPDATE signers
        SET status = 'signed', signed_at = NOW(), ip_address = $1, user_agent = $2
        WHERE id = $3
        RETURNING id, document_id, email, name, order_index, status, access_token,
                  ip_address, user_agent, viewed_at, signed_at, declined_at, decline_reason,
                  email_sent_at, created_at, updated_at
        "#,
    )
    .bind(ip_address)
    .bind(user_agent)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(signer)
}

pub async fn mark_signer_declined(pool: &PgPool, id: Uuid, reason: Option<&str>) -> Result<Signer> {
    let signer = sqlx::query_as::<_, Signer>(
        r#"
        UPDATE signers
        SET status = 'declined', declined_at = NOW(), decline_reason = $1
        WHERE id = $2
        RETURNING id, document_id, email, name, order_index, status, access_token,
                  ip_address, user_agent, viewed_at, signed_at, declined_at, decline_reason,
                  email_sent_at, created_at, updated_at
        "#,
    )
    .bind(reason)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(signer)
}

pub async fn mark_email_sent(pool: &PgPool, id: Uuid) -> Result<Signer> {
    let signer = sqlx::query_as::<_, Signer>(
        r#"
        UPDATE signers
        SET status = 'sent', email_sent_at = NOW()
        WHERE id = $1 AND status = 'pending'
        RETURNING id, document_id, email, name, order_index, status, access_token,
                  ip_address, user_agent, viewed_at, signed_at, declined_at, decline_reason,
                  email_sent_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(signer)
}

pub async fn delete_signer(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM signers WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn count_signers_by_document(pool: &PgPool, document_id: Uuid) -> Result<i64> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM signers WHERE document_id = $1")
        .bind(document_id)
        .fetch_one(pool)
        .await?;

    Ok(count.0)
}

pub async fn count_signed_by_document(pool: &PgPool, document_id: Uuid) -> Result<i64> {
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM signers WHERE document_id = $1 AND status = 'signed'")
            .bind(document_id)
            .fetch_one(pool)
            .await?;

    Ok(count.0)
}
