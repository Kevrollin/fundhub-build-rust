use chrono::{DateTime, Utc};
use sqlx::Type;
use sqlx::types::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub base_role: BaseRole,
    pub is_verified: bool,
    pub status: UserStatus,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum UserRole {
    Guest,
    User,
    Student,
    Admin,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum BaseRole {
    Guest,
    BaseUser,
    Student,
    Admin,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingEmailVerification,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Student {
    pub id: Uuid,
    pub user_id: Uuid,
    pub school_email: String,
    pub admission_number: Option<String>,
    pub verification_status: String,
    pub verification_progress: Option<i32>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verified_by: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub student_id: Uuid,
    pub public_key: String,
    pub status: String,
    pub balance: BigDecimal,
    pub last_synced_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum WalletStatus {
    Connected,
    Disconnected,
    Suspended,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub student_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub repo_url: Option<String>,
    pub media_url: Option<String>,
    pub tags: Vec<String>,
    pub funding_goal: BigDecimal,
    pub status: String,
    pub contract_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ProjectStatus {
    PendingReview,
    Active,
    Paused,
    Completed,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMilestone {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub amount_stroops: i64,
    pub proof_type: Option<String>,
    pub position: Option<i32>,
    pub status: Option<String>,
    pub proof_url: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Donation {
    pub id: Uuid,
    pub donor_id: Option<Uuid>,
    pub project_id: Uuid,
    pub amount: BigDecimal,
    pub tx_hash: Option<String>,
    pub memo: Option<String>,
    pub status: String,
    pub payment_method: String,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum DonationStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum PaymentMethod {
    Stellar,
    MobileMoney,
    Card,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: Uuid,
    pub owner_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub path: String,
    pub filename: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnchainTransaction {
    pub id: Uuid,
    pub tx_hash: String,
    pub source_account: Option<String>,
    pub destination_account: Option<String>,
    pub amount_stroops: Option<i64>,
    pub amount_xlm: Option<BigDecimal>,
    pub memo: Option<String>,
    pub memo_type: Option<String>,
    pub ledger: Option<i32>,
    pub operation_type: Option<String>,
    pub successful: bool,
    pub created_at: DateTime<Utc>,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Campaign {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub reward_pool_xlm: BigDecimal,
    pub criteria: serde_json::Value,
    pub status: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignDistribution {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub recipient_id: Uuid,
    pub amount: f64,
    pub tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

// New models for role system and guest flow

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestDonation {
    pub id: Uuid,
    pub guest_name: String,
    pub guest_email: String,
    pub project_id: Uuid,
    pub tx_hash: Option<String>,
    pub amount: BigDecimal,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudentVerification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub school_email: String,
    pub status: VerificationStatus,
    pub admin_message: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Milestone {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub target_amount: BigDecimal,
    pub released: bool,
    pub released_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub target_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// Request/Response DTOs for new endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct GuestFundingRequest {
    pub guest_name: String,
    pub guest_email: String,
    pub project_id: Uuid,
    pub tx_hash: Option<String>,
    pub amount: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudentVerificationRequest {
    pub school_email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MilestoneReleaseRequest {
    pub milestone_id: Uuid,
    pub tx_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicProjectInfo {
    pub id: Uuid,
    pub title: String,
    pub short_description: Option<String>,
    pub media_url: Option<String>,
    pub funding_goal: BigDecimal,
    pub current_funding: BigDecimal,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}