use super::provider::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StripeProvider {
    config: StripeConfig,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct StripeSessionRequest {
    payment_method_types: Vec<String>,
    line_items: Vec<StripeLineItem>,
    mode: String,
    success_url: String,
    cancel_url: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StripeLineItem {
    price_data: StripePriceData,
    quantity: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct StripePriceData {
    currency: String,
    product_data: StripeProductData,
    unit_amount: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct StripeProductData {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StripeSessionResponse {
    id: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StripeWebhookEvent {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    data: StripeWebhookData,
}

#[derive(Debug, Serialize, Deserialize)]
struct StripeWebhookData {
    object: StripePaymentIntent,
}

#[derive(Debug, Serialize, Deserialize)]
struct StripePaymentIntent {
    id: String,
    amount: u32,
    currency: String,
    status: String,
    metadata: HashMap<String, String>,
}

impl StripeProvider {
    pub fn new(config: StripeConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.config.secret_key)
    }
}

#[async_trait]
impl PaymentProvider for StripeProvider {
    async fn initiate_payment(&self, request: InitiatePaymentRequest) -> Result<PaymentInstruction, String> {
        let session_request = StripeSessionRequest {
            payment_method_types: vec!["card".to_string()],
            line_items: vec![StripeLineItem {
                price_data: StripePriceData {
                    currency: request.currency.clone(),
                    product_data: StripeProductData {
                        name: "FundHub Donation".to_string(),
                        description: request.memo.clone(),
                    },
                    unit_amount: (request.amount * 100.0) as u32, // Convert to cents
                },
                quantity: 1,
            }],
            mode: "payment".to_string(),
            success_url: format!("{}?session_id={{CHECKOUT_SESSION_ID}}", self.config.success_url),
            cancel_url: self.config.cancel_url.clone(),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("project_id".to_string(), request.project_id.to_string());
                metadata.insert("donor_email".to_string(), request.donor_email.clone());
                if let Some(phone) = request.donor_phone {
                    metadata.insert("donor_phone".to_string(), phone);
                }
                if let Some(memo) = request.memo {
                    metadata.insert("memo".to_string(), memo);
                }
                metadata
            },
        };

        let response = self
            .client
            .post("https://api.stripe.com/v1/checkout/sessions")
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&session_request)
            .send()
            .await
            .map_err(|e| format!("Failed to create Stripe session: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Stripe API error: {}", error_text));
        }

        let session_response: StripeSessionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Stripe response: {}", e))?;

        let mut instructions = HashMap::new();
        instructions.insert("session_id".to_string(), session_response.id.clone());
        instructions.insert("checkout_url".to_string(), session_response.url.clone());

        Ok(PaymentInstruction {
            payment_id: session_response.id,
            checkout_url: Some(session_response.url),
            payment_method: "stripe".to_string(),
            instructions,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24), // Stripe sessions expire in 24 hours
        })
    }

    async fn verify_payment(&self, webhook: ProviderWebhook) -> Result<VerificationResult, String> {
        if webhook.provider != "stripe" {
            return Err("Invalid provider for Stripe webhook".to_string());
        }

        let event: StripeWebhookEvent = serde_json::from_value(webhook.raw_data.clone())
            .map_err(|e| format!("Failed to parse Stripe webhook: {}", e))?;

        let payment_intent = event.data.object;
        
        let status = match payment_intent.status.as_str() {
            "succeeded" => PaymentStatus::Completed,
            "processing" => PaymentStatus::Processing,
            "requires_payment_method" | "requires_confirmation" | "requires_action" => PaymentStatus::Pending,
            "canceled" => PaymentStatus::Cancelled,
            _ => PaymentStatus::Failed,
        };

        let payment_id = payment_intent.id.clone();
        Ok(VerificationResult {
            payment_id,
            status,
            amount: payment_intent.amount as f64 / 100.0, // Convert from cents
            currency: payment_intent.currency,
            transaction_id: Some(payment_intent.id),
            provider_response: webhook.raw_data,
        })
    }

    async fn refund(&self, request: RefundRequest) -> Result<String, String> {
        let refund_data = serde_json::json!({
            "payment_intent": request.payment_id,
            "amount": if let Some(amount) = request.amount {
                (amount * 100.0) as u32
            } else {
                return Err("Amount is required for refund".to_string());
            },
            "reason": request.reason
        });

        let response = self
            .client
            .post("https://api.stripe.com/v1/refunds")
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&refund_data)
            .send()
            .await
            .map_err(|e| format!("Failed to create refund: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Stripe refund error: {}", error_text));
        }

        let refund_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse refund response: {}", e))?;

        Ok(refund_response["id"].as_str().unwrap_or("unknown").to_string())
    }

    async fn get_payment_status(&self, payment_id: &str) -> Result<PaymentStatus, String> {
        let response = self
            .client
            .get(&format!("https://api.stripe.com/v1/payment_intents/{}", payment_id))
            .header("Authorization", self.get_auth_header())
            .send()
            .await
            .map_err(|e| format!("Failed to get payment status: {}", e))?;

        if !response.status().is_success() {
            return Err("Failed to get payment status".to_string());
        }

        let payment_intent: StripePaymentIntent = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse payment status: {}", e))?;

        let status = match payment_intent.status.as_str() {
            "succeeded" => PaymentStatus::Completed,
            "processing" => PaymentStatus::Processing,
            "requires_payment_method" | "requires_confirmation" | "requires_action" => PaymentStatus::Pending,
            "canceled" => PaymentStatus::Cancelled,
            _ => PaymentStatus::Failed,
        };

        Ok(status)
    }

    fn validate_webhook(&self, payload: &str, signature: &str) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let expected_signature = format!("t={},v1={}", 
            chrono::Utc::now().timestamp(), 
            hex::encode(Hmac::<Sha256>::new_from_slice(self.config.webhook_secret.as_bytes())
                .unwrap()
                .chain_update(payload)
                .finalize()
                .into_bytes())
        );

        signature == expected_signature
    }

    fn get_provider_name(&self) -> &str {
        "stripe"
    }
}
