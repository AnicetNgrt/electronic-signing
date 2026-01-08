use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{header, Response},
    Json,
};
use serde::Serialize;
use tokio::fs;
use uuid::Uuid;

use crate::api::error::{ApiError, ApiResult};
use crate::api::middleware::extract_client_info;
use crate::api::state::AppState;
use crate::db;
use crate::models::audit::AuditAction;
use crate::models::document::{DocumentFieldRow, DocumentStatus};
use crate::models::signature::CompleteSigningRequest;
use crate::models::signer::{DeclineRequest, Signer, SignerStatus};
use crate::services::{audit, signing};

#[derive(Debug, Serialize)]
pub struct SigningSession {
    pub document_id: Uuid,
    pub document_title: String,
    pub signer: SignerInfo,
    pub fields: Vec<DocumentFieldRow>,
    pub page_count: usize,
}

#[derive(Debug, Serialize)]
pub struct SignerInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub status: SignerStatus,
}

pub async fn get_signing_session(
    State(state): State<AppState>,
    Path(token): Path<String>,
    request: Request,
) -> ApiResult<Json<SigningSession>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let signer = db::signer::get_signer_by_access_token(&state.pool, &token)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invalid signing link".to_string()))?;

    let document = db::document::get_document_by_id(&state.pool, signer.document_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.status == DocumentStatus::Voided {
        return Err(ApiError::BadRequest("This document has been voided".to_string()));
    }

    if document.status == DocumentStatus::Expired {
        return Err(ApiError::BadRequest("This document has expired".to_string()));
    }

    if signer.status == SignerStatus::Signed {
        return Err(ApiError::BadRequest(
            "You have already signed this document".to_string(),
        ));
    }

    if signer.status == SignerStatus::Declined {
        return Err(ApiError::BadRequest(
            "You have declined to sign this document".to_string(),
        ));
    }

    if signer.viewed_at.is_none() {
        db::signer::mark_signer_viewed(&state.pool, signer.id, &ip_address, &user_agent).await?;

        audit::log_action(
            &state.pool,
            document.id,
            Some(signer.id),
            None,
            AuditAction::SignerViewed,
            Some(&ip_address),
            Some(&user_agent),
            Some(serde_json::json!({
                "signer_email": signer.email
            })),
        )
        .await?;
    }

    let fields = db::document::get_fields_by_document(&state.pool, document.id).await?;

    let signer_fields: Vec<DocumentFieldRow> = fields
        .into_iter()
        .filter(|f| f.signer_id.is_none() || f.signer_id == Some(signer.id))
        .collect();

    let metadata = crate::services::pdf::get_pdf_metadata(std::path::Path::new(&document.file_path))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to read PDF: {}", e)))?;

    Ok(Json(SigningSession {
        document_id: document.id,
        document_title: document.title,
        signer: SignerInfo {
            id: signer.id,
            name: signer.name,
            email: signer.email,
            status: signer.status,
        },
        fields: signer_fields,
        page_count: metadata.page_count,
    }))
}

pub async fn get_signing_pdf(
    State(state): State<AppState>,
    Path(token): Path<String>,
    request: Request,
) -> ApiResult<Response<Body>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let signer = db::signer::get_signer_by_access_token(&state.pool, &token)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invalid signing link".to_string()))?;

    let document = db::document::get_document_by_id(&state.pool, signer.document_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.status == DocumentStatus::Voided || document.status == DocumentStatus::Expired {
        return Err(ApiError::BadRequest("Document not available".to_string()));
    }

    audit::log_action(
        &state.pool,
        document.id,
        Some(signer.id),
        None,
        AuditAction::DocumentViewed,
        Some(&ip_address),
        Some(&user_agent),
        None,
    )
    .await?;

    let file_data = fs::read(&document.file_path)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to read file: {}", e)))?;

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .body(Body::from(file_data))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to build response: {}", e)))?;

    Ok(response)
}

pub async fn submit_signing(
    State(state): State<AppState>,
    Path(token): Path<String>,
    request: Request,
    Json(req): Json<CompleteSigningRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let signer = db::signer::get_signer_by_access_token(&state.pool, &token)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invalid signing link".to_string()))?;

    let document = db::document::get_document_by_id(&state.pool, signer.document_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.status == DocumentStatus::Voided {
        return Err(ApiError::BadRequest("Document has been voided".to_string()));
    }

    if document.status == DocumentStatus::Completed {
        return Err(ApiError::BadRequest("Document already completed".to_string()));
    }

    let ctx = signing::SigningContext {
        signer_id: signer.id,
        document_id: document.id,
        ip_address,
        user_agent,
    };

    signing::process_signing(&state.pool, &ctx, &req)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let updated_doc = db::document::get_document_by_id(&state.pool, document.id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if updated_doc.status == DocumentStatus::Completed {
        if let Some(email_service) = &state.email_service {
            let owner = db::user::get_user_by_id(&state.pool, document.owner_id).await?;
            if let Some(owner) = owner {
                let _ = email_service
                    .send_completion_notification(&owner.email, &owner.name, &document.title)
                    .await;
            }

            let signers = db::signer::get_signers_by_document(&state.pool, document.id).await?;
            for s in signers {
                if s.status == SignerStatus::Signed {
                    let _ = email_service
                        .send_completion_notification(&s.email, &s.name, &document.title)
                        .await;
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "document_completed": updated_doc.status == DocumentStatus::Completed
    })))
}

pub async fn decline_signing_request(
    State(state): State<AppState>,
    Path(token): Path<String>,
    request: Request,
    Json(req): Json<DeclineRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let signer = db::signer::get_signer_by_access_token(&state.pool, &token)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invalid signing link".to_string()))?;

    signing::decline_signing(
        &state.pool,
        signer.id,
        signer.document_id,
        req.reason.as_deref(),
        &ip_address,
        &user_agent,
    )
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn get_signer_by_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> ApiResult<Json<Signer>> {
    let signer = db::signer::get_signer_by_access_token(&state.pool, &token)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invalid signing link".to_string()))?;

    Ok(Json(signer))
}
