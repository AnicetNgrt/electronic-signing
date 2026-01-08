use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Draft,
    Pending,
    Completed,
    Voided,
    Expired,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Document {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub original_filename: String,
    pub file_path: String,
    pub file_hash: String,
    pub status: DocumentStatus,
    pub self_sign_only: bool,
    pub total_signers: i32,
    pub completed_signers: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDocumentRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: String,
    pub self_sign_only: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDocumentRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentField {
    pub id: Uuid,
    pub field_type: FieldType,
    pub page: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub signer_id: Option<Uuid>,
    pub value: Option<String>,
    pub font_size: Option<i32>,
    pub font_family: Option<String>,
    pub date_format: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "field_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Signature,
    Date,
    Text,
    Initial,
}

#[derive(Debug, FromRow, Serialize)]
pub struct DocumentFieldRow {
    pub id: Uuid,
    pub document_id: Uuid,
    pub field_type: FieldType,
    pub page: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub signer_id: Option<Uuid>,
    pub value: Option<String>,
    pub font_size: Option<i32>,
    pub font_family: Option<String>,
    pub date_format: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddFieldRequest {
    pub field_type: FieldType,
    pub page: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub signer_id: Option<Uuid>,
    pub value: Option<String>,
    pub font_size: Option<i32>,
    pub font_family: Option<String>,
    pub date_format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFieldRequest {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub value: Option<String>,
    pub font_size: Option<i32>,
    pub font_family: Option<String>,
    pub date_format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DocumentWithFields {
    #[serde(flatten)]
    pub document: Document,
    pub fields: Vec<DocumentFieldRow>,
    pub signers: Vec<super::signer::Signer>,
}

#[derive(Debug, Serialize)]
pub struct DocumentListItem {
    pub id: Uuid,
    pub title: String,
    pub status: DocumentStatus,
    pub self_sign_only: bool,
    pub total_signers: i32,
    pub completed_signers: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
