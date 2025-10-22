use super::provider::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MpesaProvider {
    config: MpesaConfig,
    client: Client,
    access_token: Option<String>,
    token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpesaTokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpesaStkPushRequest {
    business_short_code: String,
    password: String,
    timestamp: String,
    transaction_type: String,
    amount: u32,
    party_a: String,
    party_b: String,
    phone_number: String,
    call_back_url: String,
    account_reference: String,
    transaction_desc: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpesaStkPushResponse {
    merchant_request_id: String,
    checkout_request_id: String,
    response_code: String,
    response_description: String,
    customer_message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpesaCallbackData {
    body: MpesaCallbackBody,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpesaCallbackBody {
    stk_callback: MpesaStkCallback,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpesaStkCallback {
    merchant_request_id: String,
    checkout_request_id: String,
    result_code: i32,
    result_desc: String,
    callback_metadata: Option<MpesaCallbackMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpesaCallbackMetadata {
    item: Vec<MpesaCallbackItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpesaCallbackItem {
    name: String,
    value: String,
}

impl MpesaProvider {
    pub fn new(config: MpesaConfig) -> Self {
        Self {
            config,
            client: Client::new(),
            access_token: None,
            token_expires_at: None,
        }
    }

    async fn get_access_token(&mut self) -> Result<String, String> {
        // Check if we have a valid token
        if let (Some(token), Some(expires_at)) = (&self.access_token, &self.token_expires_at) {
            if chrono::Utc::now() < *expires_at {
                return Ok(token.clone());
            }
        }

        // Get new token
        let url = if self.config.environment == "production" {
            "https://api.safaricom.co.ke/oauth/v1/generate?grant_type=client_credentials"
        } else {
            "https://sandbox.safaricom.co.ke/oauth/v1/generate?grant_type=client_credentials"
        };

        let response = self
            .client
            .get(url)
            .basic_auth(&self.config.consumer_key, Some(&self.config.consumer_secret))
            .send()
            .await
            .map_err(|e| format!("Failed to get access token: {}", e))?;

        if !response.status().is_success() {
            return Err("Failed to get access token".to_string());
        }

        let token_response: MpesaTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))?;

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64);
        
        self.access_token = Some(token_response.access_token.clone());
        self.token_expires_at = Some(expires_at);

        Ok(token_response.access_token)
    }

    fn generate_password(&self) -> String {
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let password_string = format!("{}{}{}", 
            self.config.business_short_code, 
            self.config.passkey, 
            timestamp
        );
        base64::encode(password_string)
    }

    fn format_phone_number(&self, phone: &str) -> String {
        let mut formatted = phone.replace("+", "").replace(" ", "");
        if formatted.starts_with("0") {
            formatted = formatted.replacen("0", "254", 1);
        } else if !formatted.starts_with("254") {
            formatted = format!("254{}", formatted);
        }
        formatted
    }
}

#[async_trait]
impl PaymentProvider for MpesaProvider {
    async fn initiate_payment(&self, request: InitiatePaymentRequest) -> Result<PaymentInstruction, String> {
        let mut provider = self.clone();
        let access_token = provider.get_access_token().await?;
        
        let phone = request.donor_phone
            .as_ref()
            .ok_or("Phone number required for M-Pesa")?;
        
        let formatted_phone = self.format_phone_number(phone);
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let password = self.generate_password();
        
        let stk_push_request = MpesaStkPushRequest {
            business_short_code: self.config.business_short_code.clone(),
            password,
            timestamp: timestamp.clone(),
            transaction_type: "CustomerPayBillOnline".to_string(),
            amount: (request.amount * 100.0) as u32, // Convert to cents
            party_a: formatted_phone.clone(),
            party_b: self.config.business_short_code.clone(),
            phone_number: formatted_phone,
            call_back_url: self.config.callback_url.clone(),
            account_reference: request.project_id.to_string(),
            transaction_desc: request.memo.unwrap_or_else(|| "FundHub Donation".to_string()),
        };

        let url = if self.config.environment == "production" {
            "https://api.safaricom.co.ke/mpesa/stkpush/v1/processrequest"
        } else {
            "https://sandbox.safaricom.co.ke/mpesa/stkpush/v1/processrequest"
        };

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&stk_push_request)
            .send()
            .await
            .map_err(|e| format!("Failed to initiate STK push: {}", e))?;

        if !response.status().is_success() {
            return Err("Failed to initiate STK push".to_string());
        }

        let stk_response: MpesaStkPushResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse STK push response: {}", e))?;

        if stk_response.response_code != "0" {
            return Err(format!("STK push failed: {}", stk_response.response_description));
        }

        let mut instructions = HashMap::new();
        instructions.insert("checkout_request_id".to_string(), stk_response.checkout_request_id.clone());
        instructions.insert("merchant_request_id".to_string(), stk_response.merchant_request_id.clone());
        instructions.insert("customer_message".to_string(), stk_response.customer_message.clone());

        Ok(PaymentInstruction {
            payment_id: stk_response.checkout_request_id,
            checkout_url: None, // M-Pesa doesn't use checkout URLs
            payment_method: "mpesa".to_string(),
            instructions,
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(10), // M-Pesa STK push expires in 10 minutes
        })
    }

    async fn verify_payment(&self, webhook: ProviderWebhook) -> Result<VerificationResult, String> {
        if webhook.provider != "mpesa" {
            return Err("Invalid provider for M-Pesa webhook".to_string());
        }

        let callback_data: MpesaCallbackData = serde_json::from_value(webhook.raw_data.clone())
            .map_err(|e| format!("Failed to parse M-Pesa callback: {}", e))?;

        let stk_callback = callback_data.body.stk_callback;
        
        let status = match stk_callback.result_code {
            0 => PaymentStatus::Completed,
            _ => PaymentStatus::Failed,
        };

        let amount = if let Some(metadata) = stk_callback.callback_metadata {
            if let Some(amount_item) = metadata.item.iter().find(|item| item.name == "Amount") {
                amount_item.value.parse::<f64>().unwrap_or(0.0) / 100.0 // Convert from cents
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(VerificationResult {
            payment_id: stk_callback.checkout_request_id,
            status,
            amount,
            currency: "KES".to_string(),
            transaction_id: Some(stk_callback.merchant_request_id),
            provider_response: webhook.raw_data,
        })
    }

    async fn refund(&self, _request: RefundRequest) -> Result<String, String> {
        Err("M-Pesa refunds not implemented yet".to_string())
    }

    async fn get_payment_status(&self, _payment_id: &str) -> Result<PaymentStatus, String> {
        // M-Pesa doesn't provide a direct status check API
        // Status is typically determined through webhooks
        Ok(PaymentStatus::Pending)
    }

    fn validate_webhook(&self, _payload: &str, _signature: &str) -> bool {
        // M-Pesa webhook validation would require HMAC verification
        // For now, we'll trust the webhook (in production, implement proper validation)
        true
    }

    fn get_provider_name(&self) -> &str {
        "mpesa"
    }
}

