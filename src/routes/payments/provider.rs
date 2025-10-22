use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiatePaymentRequest {
    pub amount: f64,
    pub currency: String,
    pub donor_email: String,
    pub donor_phone: Option<String>,
    pub project_id: uuid::Uuid,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInstruction {
    pub payment_id: String,
    pub checkout_url: Option<String>,
    pub payment_method: String,
    pub instructions: HashMap<String, String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderWebhook {
    pub provider: String,
    pub event_type: String,
    pub payment_id: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub raw_data: serde_json::Value,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub payment_id: String,
    pub status: PaymentStatus,
    pub amount: f64,
    pub currency: String,
    pub transaction_id: Option<String>,
    pub provider_response: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub payment_id: String,
    pub amount: Option<f64>,
    pub reason: String,
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    /// Initialize a payment and return payment instructions
    async fn initiate_payment(&self, request: InitiatePaymentRequest) -> Result<PaymentInstruction, String>;
    
    /// Verify a payment from webhook notification
    async fn verify_payment(&self, webhook: ProviderWebhook) -> Result<VerificationResult, String>;
    
    /// Process a refund
    async fn refund(&self, request: RefundRequest) -> Result<String, String>;
    
    /// Get payment status
    async fn get_payment_status(&self, payment_id: &str) -> Result<PaymentStatus, String>;
    
    /// Validate webhook signature
    fn validate_webhook(&self, payload: &str, signature: &str) -> bool;
    
    /// Get provider name
    fn get_provider_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct PaymentProviderFactory;

impl PaymentProviderFactory {
    pub fn create_mpesa_provider(config: MpesaConfig) -> Box<dyn PaymentProvider> {
        Box::new(crate::routes::payments::mpesa::MpesaProvider::new(config))
    }
    
    pub fn create_stripe_provider(config: StripeConfig) -> Box<dyn PaymentProvider> {
        Box::new(crate::routes::payments::stripe::StripeProvider::new(config))
    }
}

#[derive(Debug, Clone)]
pub struct MpesaConfig {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub business_short_code: String,
    pub passkey: String,
    pub callback_url: String,
    pub environment: String, // sandbox or production
}

#[derive(Debug, Clone)]
pub struct StripeConfig {
    pub secret_key: String,
    pub publishable_key: String,
    pub webhook_secret: String,
    pub success_url: String,
    pub cancel_url: String,
}
