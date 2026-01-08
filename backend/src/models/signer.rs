use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "signer_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SignerStatus {
    Pending,
    Sent,
    Viewed,
    Signed,
    Declined,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Signer {
    pub id: Uuid,
    pub document_id: Uuid,
    pub email: String,
    pub name: String,
    pub order_index: i32,
    pub status: SignerStatus,
    pub access_token: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub signed_at: Option<DateTime<Utc>>,
    pub declined_at: Option<DateTime<Utc>>,
    pub decline_reason: Option<String>,
    pub email_sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddSignerRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    pub order_index: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSignerRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub order_index: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct SignerPublic {
    pub id: Uuid,
    pub document_id: Uuid,
    pub email: String,
    pub name: String,
    pub order_index: i32,
    pub status: SignerStatus,
    pub viewed_at: Option<DateTime<Utc>>,
    pub signed_at: Option<DateTime<Utc>>,
    pub declined_at: Option<DateTime<Utc>>,
    pub email_sent_at: Option<DateTime<Utc>>,
}

impl From<Signer> for SignerPublic {
    fn from(s: Signer) -> Self {
        Self {
            id: s.id,
            document_id: s.document_id,
            email: s.email,
            name: s.name,
            order_index: s.order_index,
            status: s.status,
            viewed_at: s.viewed_at,
            signed_at: s.signed_at,
            declined_at: s.declined_at,
            email_sent_at: s.email_sent_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DeclineRequest {
    pub reason: Option<String>,
}
