use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::models::audit::AuditAction;
use crate::models::document::DocumentStatus;
use crate::models::signature::CompleteSigningRequest;
use crate::models::signer::SignerStatus;
use crate::services::{audit, crypto};

pub struct SigningContext {
    pub signer_id: Uuid,
    pub document_id: Uuid,
    pub ip_address: String,
    pub user_agent: String,
}

pub async fn process_signing(
    pool: &PgPool,
    ctx: &SigningContext,
    request: &CompleteSigningRequest,
) -> Result<()> {
    let signer = db::signer::get_signer_by_id(pool, ctx.signer_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Signer not found"))?;

    if signer.status == SignerStatus::Signed {
        return Err(anyhow::anyhow!("Document already signed by this signer"));
    }

    if signer.status == SignerStatus::Declined {
        return Err(anyhow::anyhow!("Signer has declined to sign"));
    }

    let document = db::document::get_document_by_id(pool, ctx.document_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Document not found"))?;

    if document.status == DocumentStatus::Completed {
        return Err(anyhow::anyhow!("Document already completed"));
    }

    if document.status == DocumentStatus::Voided {
        return Err(anyhow::anyhow!("Document has been voided"));
    }

    for sig_req in &request.signatures {
        let field = db::document::get_field_by_id(pool, sig_req.field_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Field not found: {}", sig_req.field_id))?;

        if field.document_id != ctx.document_id {
            return Err(anyhow::anyhow!("Field does not belong to this document"));
        }

        if let Some(assigned_signer) = field.signer_id {
            if assigned_signer != ctx.signer_id {
                return Err(anyhow::anyhow!("Field not assigned to this signer"));
            }
        }

        let signature_hash = crypto::hash_string(&sig_req.signature_data);

        db::signature::create_signature(
            pool,
            ctx.signer_id,
            ctx.document_id,
            sig_req.field_id,
            &sig_req.signature_data,
            &signature_hash,
            &ctx.ip_address,
            &ctx.user_agent,
        )
        .await?;

        audit::log_action(
            pool,
            ctx.document_id,
            Some(ctx.signer_id),
            None,
            AuditAction::SignatureApplied,
            Some(&ctx.ip_address),
            Some(&ctx.user_agent),
            Some(serde_json::json!({
                "field_id": sig_req.field_id,
                "signature_hash": signature_hash
            })),
        )
        .await?;
    }

    for field_req in &request.field_values {
        let field = db::document::get_field_by_id(pool, field_req.field_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Field not found: {}", field_req.field_id))?;

        if field.document_id != ctx.document_id {
            return Err(anyhow::anyhow!("Field does not belong to this document"));
        }

        db::document::update_field_value(pool, field_req.field_id, &field_req.value).await?;
    }

    db::signer::mark_signer_signed(pool, ctx.signer_id, &ctx.ip_address, &ctx.user_agent).await?;

    audit::log_action(
        pool,
        ctx.document_id,
        Some(ctx.signer_id),
        None,
        AuditAction::SignerSigned,
        Some(&ctx.ip_address),
        Some(&ctx.user_agent),
        Some(serde_json::json!({
            "signer_email": signer.email,
            "signer_name": signer.name
        })),
    )
    .await?;

    let updated_doc = db::document::increment_completed_signers(pool, ctx.document_id).await?;

    if updated_doc.completed_signers >= updated_doc.total_signers {
        db::document::mark_document_completed(pool, ctx.document_id).await?;

        audit::log_action(
            pool,
            ctx.document_id,
            None,
            None,
            AuditAction::DocumentCompleted,
            None,
            None,
            Some(serde_json::json!({
                "total_signers": updated_doc.total_signers,
                "completed_signers": updated_doc.completed_signers
            })),
        )
        .await?;
    }

    Ok(())
}

pub async fn decline_signing(
    pool: &PgPool,
    signer_id: Uuid,
    document_id: Uuid,
    reason: Option<&str>,
    ip_address: &str,
    user_agent: &str,
) -> Result<()> {
    let signer = db::signer::get_signer_by_id(pool, signer_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Signer not found"))?;

    if signer.status == SignerStatus::Signed {
        return Err(anyhow::anyhow!("Document already signed"));
    }

    if signer.status == SignerStatus::Declined {
        return Err(anyhow::anyhow!("Already declined"));
    }

    db::signer::mark_signer_declined(pool, signer_id, reason).await?;

    audit::log_action(
        pool,
        document_id,
        Some(signer_id),
        None,
        AuditAction::SignerDeclined,
        Some(ip_address),
        Some(user_agent),
        Some(serde_json::json!({
            "signer_email": signer.email,
            "reason": reason
        })),
    )
    .await?;

    Ok(())
}
