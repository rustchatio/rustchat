use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TermsOfService {
    pub id: Uuid,
    pub version: String,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub is_active: bool,
    pub effective_date: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserTermsAcceptance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub terms_id: Uuid,
    pub accepted_at: DateTime<Utc>,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTermsRequest {
    pub version: String,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub effective_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTermsRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub effective_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermsAcceptanceRequest {
    pub terms_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermsStatusResponse {
    pub has_accepted: bool,
    pub current_terms: Option<TermsOfService>,
    pub accepted_version: Option<String>,
    pub acceptance_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermsStats {
    pub total_users: i64,
    pub accepted_count: i64,
    pub pending_count: i64,
    pub acceptance_rate: f64,
}
