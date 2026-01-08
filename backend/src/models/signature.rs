use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Signature {
    pub id: Uuid,
    pub signer_id: Uuid,
    pub document_id: Uuid,
    pub field_id: Uuid,
    pub signature_data: String,
    pub signature_hash: String,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitSignatureRequest {
    pub field_id: Uuid,
    pub signature_data: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitFieldValueRequest {
    pub field_id: Uuid,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct CompleteSigningRequest {
    pub signatures: Vec<SubmitSignatureRequest>,
    pub field_values: Vec<SubmitFieldValueRequest>,
}
