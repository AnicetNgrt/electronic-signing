use axum::{
    extract::{Multipart, Path, Query, Request, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::api::error::{ApiError, ApiResult};
use crate::api::middleware::{extract_client_info, AuthUser};
use crate::api::state::AppState;
use crate::db;
use crate::models::audit::AuditAction;
use crate::models::document::{
    AddFieldRequest, Document, DocumentFieldRow, DocumentStatus, DocumentWithFields,
    UpdateFieldRequest,
};
use crate::models::signer::{AddSignerRequest, Signer};
use crate::services::{audit, crypto, pdf};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DocumentListResponse {
    pub documents: Vec<Document>,
    pub total: i64,
}

pub async fn list_documents(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<DocumentListResponse>> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let documents =
        db::document::get_documents_by_owner(&state.pool, auth_user.user_id, limit, offset).await?;

    let total = db::document::count_documents_by_owner(&state.pool, auth_user.user_id).await?;

    Ok(Json(DocumentListResponse { documents, total }))
}

pub async fn get_document(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DocumentWithFields>> {
    let document = db::document::get_document_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    let fields = db::document::get_fields_by_document(&state.pool, id).await?;
    let signers = db::signer::get_signers_by_document(&state.pool, id).await?;

    Ok(Json(DocumentWithFields {
        document,
        fields,
        signers,
    }))
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentForm {
    pub title: String,
    pub self_sign_only: Option<bool>,
}

pub async fn create_document(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    request: Request,
    mut multipart: Multipart,
) -> ApiResult<Json<Document>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let mut title: Option<String> = None;
    let mut self_sign_only = false;
    let mut file_data: Option<(String, Vec<u8>)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "title" => {
                title = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| ApiError::BadRequest(e.to_string()))?,
                );
            }
            "self_sign_only" => {
                let value = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
                self_sign_only = value == "true" || value == "1";
            }
            "file" => {
                let filename = field
                    .file_name()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "document.pdf".to_string());

                let content_type = field.content_type().map(|s| s.to_string());

                if content_type.as_deref() != Some("application/pdf") {
                    if !filename.to_lowercase().ends_with(".pdf") {
                        return Err(ApiError::BadRequest("File must be a PDF".to_string()));
                    }
                }

                let data = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

                if data.len() as u64 > state.config.max_file_size_bytes() {
                    return Err(ApiError::BadRequest(format!(
                        "File too large. Maximum size is {} MB",
                        state.config.max_file_size_mb
                    )));
                }

                file_data = Some((filename, data.to_vec()));
            }
            _ => {}
        }
    }

    let title = title.ok_or_else(|| ApiError::BadRequest("Title is required".to_string()))?;
    let (filename, data) =
        file_data.ok_or_else(|| ApiError::BadRequest("PDF file is required".to_string()))?;

    let doc_id = Uuid::new_v4();
    let file_hash = crypto::hash_data(&data);

    let storage_dir = PathBuf::from(&state.config.storage_path)
        .join(auth_user.user_id.to_string())
        .join(doc_id.to_string());

    fs::create_dir_all(&storage_dir)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create storage dir: {}", e)))?;

    let file_path = storage_dir.join("original.pdf");

    let mut file = fs::File::create(&file_path)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create file: {}", e)))?;

    file.write_all(&data)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to write file: {}", e)))?;

    pdf::validate_pdf(&file_path)
        .map_err(|e| ApiError::BadRequest(format!("Invalid PDF file: {}", e)))?;

    let document = db::document::create_document(
        &state.pool,
        auth_user.user_id,
        &title,
        &filename,
        file_path.to_str().unwrap(),
        &file_hash,
        self_sign_only,
    )
    .await?;

    audit::log_action(
        &state.pool,
        document.id,
        None,
        Some(auth_user.user_id),
        AuditAction::DocumentCreated,
        Some(&ip_address),
        Some(&user_agent),
        Some(serde_json::json!({
            "title": title,
            "filename": filename,
            "file_hash": file_hash
        })),
    )
    .await?;

    info!(
        "Document created: {} by user {}",
        document.id, auth_user.user_id
    );

    Ok(Json(document))
}

pub async fn delete_document(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let document = db::document::get_document_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    if document.status == DocumentStatus::Completed {
        return Err(ApiError::BadRequest(
            "Cannot delete completed documents".to_string(),
        ));
    }

    let file_path = PathBuf::from(&document.file_path);
    if let Some(parent) = file_path.parent() {
        let _ = fs::remove_dir_all(parent).await;
    }

    db::document::delete_document(&state.pool, id).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn add_field(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    request: Request,
    Json(req): Json<AddFieldRequest>,
) -> ApiResult<Json<DocumentFieldRow>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let document = db::document::get_document_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    if document.status != DocumentStatus::Draft {
        return Err(ApiError::BadRequest(
            "Cannot modify non-draft documents".to_string(),
        ));
    }

    let field = db::document::add_field(&state.pool, id, &req).await?;

    audit::log_action(
        &state.pool,
        id,
        None,
        Some(auth_user.user_id),
        AuditAction::FieldAdded,
        Some(&ip_address),
        Some(&user_agent),
        Some(serde_json::json!({
            "field_id": field.id,
            "field_type": format!("{:?}", req.field_type),
            "page": req.page
        })),
    )
    .await?;

    Ok(Json(field))
}

pub async fn update_field(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((doc_id, field_id)): Path<(Uuid, Uuid)>,
    request: Request,
    Json(req): Json<UpdateFieldRequest>,
) -> ApiResult<Json<DocumentFieldRow>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let document = db::document::get_document_by_id(&state.pool, doc_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    if document.status != DocumentStatus::Draft {
        return Err(ApiError::BadRequest(
            "Cannot modify non-draft documents".to_string(),
        ));
    }

    let field = db::document::get_field_by_id(&state.pool, field_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Field not found".to_string()))?;

    if field.document_id != doc_id {
        return Err(ApiError::NotFound("Field not found".to_string()));
    }

    let updated = db::document::update_field(&state.pool, field_id, &req).await?;

    audit::log_action(
        &state.pool,
        doc_id,
        None,
        Some(auth_user.user_id),
        AuditAction::FieldUpdated,
        Some(&ip_address),
        Some(&user_agent),
        Some(serde_json::json!({
            "field_id": field_id
        })),
    )
    .await?;

    Ok(Json(updated))
}

pub async fn delete_field(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((doc_id, field_id)): Path<(Uuid, Uuid)>,
    request: Request,
) -> ApiResult<Json<serde_json::Value>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let document = db::document::get_document_by_id(&state.pool, doc_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    if document.status != DocumentStatus::Draft {
        return Err(ApiError::BadRequest(
            "Cannot modify non-draft documents".to_string(),
        ));
    }

    let field = db::document::get_field_by_id(&state.pool, field_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Field not found".to_string()))?;

    if field.document_id != doc_id {
        return Err(ApiError::NotFound("Field not found".to_string()));
    }

    db::document::delete_field(&state.pool, field_id).await?;

    audit::log_action(
        &state.pool,
        doc_id,
        None,
        Some(auth_user.user_id),
        AuditAction::FieldDeleted,
        Some(&ip_address),
        Some(&user_agent),
        Some(serde_json::json!({
            "field_id": field_id
        })),
    )
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn add_signer(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    request: Request,
    Json(req): Json<AddSignerRequest>,
) -> ApiResult<Json<Signer>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let document = db::document::get_document_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    if document.status != DocumentStatus::Draft {
        return Err(ApiError::BadRequest(
            "Cannot modify non-draft documents".to_string(),
        ));
    }

    if document.self_sign_only {
        return Err(ApiError::BadRequest(
            "Cannot add signers to self-sign documents".to_string(),
        ));
    }

    let existing_signers = db::signer::get_signers_by_document(&state.pool, id).await?;
    let order_index = req
        .order_index
        .unwrap_or(existing_signers.len() as i32);

    let access_token = crypto::generate_access_token();

    let signer = db::signer::create_signer(
        &state.pool,
        id,
        &req.email,
        &req.name,
        order_index,
        &access_token,
    )
    .await?;

    db::document::update_total_signers(&state.pool, id, (existing_signers.len() + 1) as i32)
        .await?;

    audit::log_action(
        &state.pool,
        id,
        Some(signer.id),
        Some(auth_user.user_id),
        AuditAction::SignerAdded,
        Some(&ip_address),
        Some(&user_agent),
        Some(serde_json::json!({
            "signer_email": req.email,
            "signer_name": req.name
        })),
    )
    .await?;

    Ok(Json(signer))
}

pub async fn remove_signer(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((doc_id, signer_id)): Path<(Uuid, Uuid)>,
    request: Request,
) -> ApiResult<Json<serde_json::Value>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let document = db::document::get_document_by_id(&state.pool, doc_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    if document.status != DocumentStatus::Draft {
        return Err(ApiError::BadRequest(
            "Cannot modify non-draft documents".to_string(),
        ));
    }

    let signer = db::signer::get_signer_by_id(&state.pool, signer_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Signer not found".to_string()))?;

    if signer.document_id != doc_id {
        return Err(ApiError::NotFound("Signer not found".to_string()));
    }

    audit::log_action(
        &state.pool,
        doc_id,
        Some(signer_id),
        Some(auth_user.user_id),
        AuditAction::SignerRemoved,
        Some(&ip_address),
        Some(&user_agent),
        Some(serde_json::json!({
            "signer_email": signer.email
        })),
    )
    .await?;

    db::signer::delete_signer(&state.pool, signer_id).await?;

    let remaining = db::signer::count_signers_by_document(&state.pool, doc_id).await?;
    db::document::update_total_signers(&state.pool, doc_id, remaining as i32).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn send_document(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    request: Request,
) -> ApiResult<Json<Document>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let document = db::document::get_document_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    if document.status != DocumentStatus::Draft {
        return Err(ApiError::BadRequest(
            "Document already sent or completed".to_string(),
        ));
    }

    let signers = db::signer::get_signers_by_document(&state.pool, id).await?;

    if signers.is_empty() && !document.self_sign_only {
        return Err(ApiError::BadRequest(
            "Add at least one signer before sending".to_string(),
        ));
    }

    let owner = db::user::get_user_by_id(&state.pool, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Owner not found".to_string()))?;

    if let Some(email_service) = &state.email_service {
        for signer in &signers {
            email_service
                .send_signing_request(
                    &signer.email,
                    &signer.name,
                    &document.title,
                    &owner.name,
                    &signer.access_token,
                )
                .await
                .map_err(|e| {
                    ApiError::Internal(anyhow::anyhow!("Failed to send email: {}", e))
                })?;

            db::signer::mark_email_sent(&state.pool, signer.id).await?;

            audit::log_action(
                &state.pool,
                id,
                Some(signer.id),
                Some(auth_user.user_id),
                AuditAction::SignerEmailSent,
                Some(&ip_address),
                Some(&user_agent),
                Some(serde_json::json!({
                    "signer_email": signer.email
                })),
            )
            .await?;
        }
    } else {
        info!(
            "Email service not configured. Signers would need manual access tokens."
        );
        for signer in &signers {
            info!(
                "Signing link for {}: {}/sign/{}",
                signer.email, state.config.public_url, signer.access_token
            );
        }
    }

    let updated = db::document::update_document_status(&state.pool, id, DocumentStatus::Pending)
        .await?;

    audit::log_action(
        &state.pool,
        id,
        None,
        Some(auth_user.user_id),
        AuditAction::DocumentSent,
        Some(&ip_address),
        Some(&user_agent),
        Some(serde_json::json!({
            "signer_count": signers.len()
        })),
    )
    .await?;

    Ok(Json(updated))
}

pub async fn void_document(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    request: Request,
) -> ApiResult<Json<Document>> {
    let (ip_address, user_agent) = extract_client_info(&request);

    let document = db::document::get_document_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    if document.status == DocumentStatus::Completed {
        return Err(ApiError::BadRequest(
            "Cannot void completed documents".to_string(),
        ));
    }

    let updated =
        db::document::update_document_status(&state.pool, id, DocumentStatus::Voided).await?;

    audit::log_action(
        &state.pool,
        id,
        None,
        Some(auth_user.user_id),
        AuditAction::DocumentVoided,
        Some(&ip_address),
        Some(&user_agent),
        None,
    )
    .await?;

    Ok(Json(updated))
}

pub async fn get_audit_logs(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<crate::models::audit::AuditLog>>> {
    let document = db::document::get_document_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    let logs = db::audit::get_audit_logs_by_document(&state.pool, id).await?;

    Ok(Json(logs))
}

pub async fn get_certificate(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<crate::models::audit::Certificate>> {
    let document = db::document::get_document_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    if document.status != DocumentStatus::Completed {
        return Err(ApiError::BadRequest(
            "Certificate only available for completed documents".to_string(),
        ));
    }

    let certificate = audit::generate_certificate(&state.pool, id).await?;

    Ok(Json(certificate))
}

pub async fn download_document(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    request: Request,
) -> ApiResult<axum::response::Response> {
    use axum::body::Body;
    use axum::http::{header, Response};

    let (ip_address, user_agent) = extract_client_info(&request);

    let document = db::document::get_document_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Document not found".to_string()))?;

    if document.owner_id != auth_user.user_id {
        return Err(ApiError::Forbidden);
    }

    let file_data = fs::read(&document.file_path)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to read file: {}", e)))?;

    audit::log_action(
        &state.pool,
        id,
        None,
        Some(auth_user.user_id),
        AuditAction::DocumentDownloaded,
        Some(&ip_address),
        Some(&user_agent),
        None,
    )
    .await?;

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", document.original_filename),
        )
        .body(Body::from(file_data))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to build response: {}", e)))?;

    Ok(response)
}
