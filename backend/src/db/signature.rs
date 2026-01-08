use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::signature::Signature;

pub async fn create_signature(
    pool: &PgPool,
    signer_id: Uuid,
    document_id: Uuid,
    field_id: Uuid,
    signature_data: &str,
    signature_hash: &str,
    ip_address: &str,
    user_agent: &str,
) -> Result<Signature> {
    let sig = sqlx::query_as::<_, Signature>(
        r#"
        INSERT INTO signatures (signer_id, document_id, field_id, signature_data, signature_hash, ip_address, user_agent)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, signer_id, document_id, field_id, signature_data, signature_hash, ip_address, user_agent, created_at
        "#,
    )
    .bind(signer_id)
    .bind(document_id)
    .bind(field_id)
    .bind(signature_data)
    .bind(signature_hash)
    .bind(ip_address)
    .bind(user_agent)
    .fetch_one(pool)
    .await?;

    Ok(sig)
}

pub async fn get_signatures_by_document(pool: &PgPool, document_id: Uuid) -> Result<Vec<Signature>> {
    let sigs = sqlx::query_as::<_, Signature>(
        r#"
        SELECT id, signer_id, document_id, field_id, signature_data, signature_hash, ip_address, user_agent, created_at
        FROM signatures
        WHERE document_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;

    Ok(sigs)
}

pub async fn get_signatures_by_signer(pool: &PgPool, signer_id: Uuid) -> Result<Vec<Signature>> {
    let sigs = sqlx::query_as::<_, Signature>(
        r#"
        SELECT id, signer_id, document_id, field_id, signature_data, signature_hash, ip_address, user_agent, created_at
        FROM signatures
        WHERE signer_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(signer_id)
    .fetch_all(pool)
    .await?;

    Ok(sigs)
}

pub async fn get_signature_by_field(pool: &PgPool, field_id: Uuid) -> Result<Option<Signature>> {
    let sig = sqlx::query_as::<_, Signature>(
        r#"
        SELECT id, signer_id, document_id, field_id, signature_data, signature_hash, ip_address, user_agent, created_at
        FROM signatures
        WHERE field_id = $1
        "#,
    )
    .bind(field_id)
    .fetch_optional(pool)
    .await?;

    Ok(sig)
}
