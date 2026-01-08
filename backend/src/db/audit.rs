use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::audit::{AuditAction, AuditLog};

pub async fn create_audit_log(
    pool: &PgPool,
    document_id: Uuid,
    signer_id: Option<Uuid>,
    user_id: Option<Uuid>,
    action: AuditAction,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    details: Option<serde_json::Value>,
    entry_hash: &str,
    previous_hash: Option<&str>,
) -> Result<AuditLog> {
    let log = sqlx::query_as::<_, AuditLog>(
        r#"
        INSERT INTO audit_logs (document_id, signer_id, user_id, action, ip_address, user_agent, details, entry_hash, previous_hash)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, document_id, signer_id, user_id, action, ip_address, user_agent, details, entry_hash, previous_hash, created_at
        "#,
    )
    .bind(document_id)
    .bind(signer_id)
    .bind(user_id)
    .bind(action)
    .bind(ip_address)
    .bind(user_agent)
    .bind(details)
    .bind(entry_hash)
    .bind(previous_hash)
    .fetch_one(pool)
    .await?;

    Ok(log)
}

pub async fn get_audit_logs_by_document(pool: &PgPool, document_id: Uuid) -> Result<Vec<AuditLog>> {
    let logs = sqlx::query_as::<_, AuditLog>(
        r#"
        SELECT id, document_id, signer_id, user_id, action, ip_address, user_agent, details, entry_hash, previous_hash, created_at
        FROM audit_logs
        WHERE document_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

pub async fn get_latest_audit_log(pool: &PgPool, document_id: Uuid) -> Result<Option<AuditLog>> {
    let log = sqlx::query_as::<_, AuditLog>(
        r#"
        SELECT id, document_id, signer_id, user_id, action, ip_address, user_agent, details, entry_hash, previous_hash, created_at
        FROM audit_logs
        WHERE document_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await?;

    Ok(log)
}

pub async fn verify_audit_chain(pool: &PgPool, document_id: Uuid) -> Result<bool> {
    let logs = get_audit_logs_by_document(pool, document_id).await?;

    if logs.is_empty() {
        return Ok(true);
    }

    for (i, log) in logs.iter().enumerate() {
        if i == 0 {
            if log.previous_hash.is_some() {
                return Ok(false);
            }
        } else {
            let prev_hash = &logs[i - 1].entry_hash;
            if log.previous_hash.as_ref() != Some(prev_hash) {
                return Ok(false);
            }
        }
    }

    Ok(true)
}
