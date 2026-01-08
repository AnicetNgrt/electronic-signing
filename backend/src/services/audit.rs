use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::models::audit::{
    AuditAction, AuditLog, Certificate, CertificateAuditEntry, CertificateSigner,
};
use crate::services::crypto;

pub async fn log_action(
    pool: &PgPool,
    document_id: Uuid,
    signer_id: Option<Uuid>,
    user_id: Option<Uuid>,
    action: AuditAction,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    details: Option<serde_json::Value>,
) -> Result<AuditLog> {
    let previous = db::audit::get_latest_audit_log(pool, document_id).await?;
    let previous_hash = previous.as_ref().map(|p| p.entry_hash.as_str());

    let timestamp = Utc::now().to_rfc3339();
    let details_str = details.as_ref().map(|d| d.to_string());

    let entry_hash = crypto::compute_audit_hash(
        &document_id,
        &format!("{:?}", action),
        &timestamp,
        previous_hash,
        details_str.as_deref(),
    );

    let log = db::audit::create_audit_log(
        pool,
        document_id,
        signer_id,
        user_id,
        action,
        ip_address,
        user_agent,
        details,
        &entry_hash,
        previous_hash,
    )
    .await?;

    Ok(log)
}

pub async fn generate_certificate(pool: &PgPool, document_id: Uuid) -> Result<Certificate> {
    let document = db::document::get_document_by_id(pool, document_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Document not found"))?;

    let signers = db::signer::get_signers_by_document(pool, document_id).await?;
    let signatures = db::signature::get_signatures_by_document(pool, document_id).await?;
    let audit_logs = db::audit::get_audit_logs_by_document(pool, document_id).await?;

    let completed_at = document
        .completed_at
        .ok_or_else(|| anyhow::anyhow!("Document not completed"))?;

    let cert_signers: Vec<CertificateSigner> = signers
        .iter()
        .filter(|s| s.signed_at.is_some())
        .map(|s| {
            let sig_hash = signatures
                .iter()
                .find(|sig| sig.signer_id == s.id)
                .map(|sig| sig.signature_hash.clone())
                .unwrap_or_default();

            CertificateSigner {
                name: s.name.clone(),
                email: s.email.clone(),
                signed_at: s.signed_at.unwrap(),
                ip_address: s.ip_address.clone().unwrap_or_else(|| "Unknown".to_string()),
                signature_hash: sig_hash,
            }
        })
        .collect();

    let audit_trail: Vec<CertificateAuditEntry> = audit_logs
        .iter()
        .map(|log| {
            let actor = if let Some(sid) = log.signer_id {
                signers
                    .iter()
                    .find(|s| s.id == sid)
                    .map(|s| format!("{} ({})", s.name, s.email))
            } else {
                Some("System".to_string())
            };

            CertificateAuditEntry {
                action: format!("{:?}", log.action),
                actor,
                timestamp: log.created_at,
                ip_address: log.ip_address.clone(),
                details: log.details.as_ref().map(|d| d.to_string()),
            }
        })
        .collect();

    let generated_at = Utc::now();

    let signers_data = serde_json::to_string(&cert_signers)?;
    let audit_data = serde_json::to_string(&audit_trail)?;

    let certificate_hash = crypto::compute_certificate_hash(
        &document_id,
        &document.file_hash,
        &signers_data,
        &audit_data,
        &generated_at.to_rfc3339(),
    );

    let cert = Certificate {
        document_id,
        document_title: document.title,
        document_hash: document.file_hash,
        created_at: document.created_at,
        completed_at,
        signers: cert_signers,
        audit_trail,
        certificate_hash,
        generated_at,
    };

    log_action(
        pool,
        document_id,
        None,
        None,
        AuditAction::CertificateGenerated,
        None,
        None,
        Some(serde_json::json!({
            "certificate_hash": cert.certificate_hash
        })),
    )
    .await?;

    Ok(cert)
}

pub async fn verify_integrity(pool: &PgPool, document_id: Uuid) -> Result<bool> {
    db::audit::verify_audit_chain(pool, document_id).await
}
