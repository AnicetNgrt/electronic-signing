use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_action", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    DocumentCreated,
    DocumentUploaded,
    DocumentViewed,
    DocumentSent,
    DocumentCompleted,
    DocumentVoided,
    DocumentDownloaded,
    FieldAdded,
    FieldUpdated,
    FieldDeleted,
    SignerAdded,
    SignerRemoved,
    SignerEmailSent,
    SignerViewed,
    SignerSigned,
    SignerDeclined,
    SignatureApplied,
    CertificateGenerated,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub document_id: Uuid,
    pub signer_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub entry_hash: String,
    pub previous_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogPublic {
    pub id: Uuid,
    pub action: AuditAction,
    pub actor_email: Option<String>,
    pub actor_name: Option<String>,
    pub ip_address: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct Certificate {
    pub document_id: Uuid,
    pub document_title: String,
    pub document_hash: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub signers: Vec<CertificateSigner>,
    pub audit_trail: Vec<CertificateAuditEntry>,
    pub certificate_hash: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CertificateSigner {
    pub name: String,
    pub email: String,
    pub signed_at: DateTime<Utc>,
    pub ip_address: String,
    pub signature_hash: String,
}

#[derive(Debug, Serialize)]
pub struct CertificateAuditEntry {
    pub action: String,
    pub actor: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub details: Option<String>,
}
