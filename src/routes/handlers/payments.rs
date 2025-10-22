use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::services::payment_service::PaymentService;
use crate::routes::payments::provider::*;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiatePaymentRequest {
    pub provider: String,
    pub amount: f64,
    pub currency: String,
    pub donor_email: String,
    pub donor_phone: Option<String>,
    pub project_id: Uuid,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentInstructionResponse {
    pub success: bool,
    pub payment_id: String,
    pub checkout_url: Option<String>,
    pub payment_method: String,
    pub instructions: HashMap<String, String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookRequest {
    pub provider: String,
    pub event_type: String,
    pub payment_id: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub raw_data: serde_json::Value,
    pub signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefundRequest {
    pub provider: String,
    pub payment_id: String,
    pub amount: Option<f64>,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefundResponse {
    pub success: bool,
    pub refund_id: String,
    pub message: String,
}

/// Initiate payment with specified provider
pub async fn initiate_payment(
    State(state): State<AppState>,
    Json(request): Json<InitiatePaymentRequest>,
) -> Result<Json<PaymentInstructionResponse>, StatusCode> {
    let mut payment_service = PaymentService::new(state.pool.clone());
    payment_service.initialize_providers().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let payment_request = crate::routes::payments::provider::InitiatePaymentRequest {
        amount: request.amount,
        currency: request.currency,
        donor_email: request.donor_email,
        donor_phone: request.donor_phone,
        project_id: request.project_id,
        memo: request.memo,
    };

    match payment_service.initiate_payment(&request.provider, payment_request).await {
        Ok(instruction) => Ok(Json(PaymentInstructionResponse {
            success: true,
            payment_id: instruction.payment_id,
            checkout_url: instruction.checkout_url,
            payment_method: instruction.payment_method,
            instructions: instruction.instructions,
            expires_at: instruction.expires_at,
        })),
        Err(e) => {
            eprintln!("Payment initiation error: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// M-Pesa webhook handler
pub async fn mpesa_webhook(
    State(state): State<AppState>,
    Json(webhook_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut payment_service = PaymentService::new(state.pool.clone());
    payment_service.initialize_providers().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let webhook = ProviderWebhook {
        provider: "mpesa".to_string(),
        event_type: "payment_completed".to_string(),
        payment_id: webhook_data["Body"]["stkCallback"]["CheckoutRequestID"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        amount: 0.0, // Will be extracted from callback
        currency: "KES".to_string(),
        status: "pending".to_string(),
        raw_data: webhook_data,
        signature: None,
    };

    match payment_service.process_webhook("mpesa", webhook).await {
        Ok(verification) => Ok(Json(serde_json::json!({
            "success": true,
            "payment_id": verification.payment_id,
            "status": format!("{:?}", verification.status),
            "amount": verification.amount
        }))),
        Err(e) => {
            eprintln!("M-Pesa webhook error: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Stripe webhook handler
pub async fn stripe_webhook(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    body: String,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut payment_service = PaymentService::new(state.pool.clone());
    payment_service.initialize_providers().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let signature = headers
        .get("stripe-signature")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let webhook_data: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let webhook = ProviderWebhook {
        provider: "stripe".to_string(),
        event_type: webhook_data["type"].as_str().unwrap_or("").to_string(),
        payment_id: webhook_data["data"]["object"]["id"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        amount: webhook_data["data"]["object"]["amount"]
            .as_f64()
            .unwrap_or(0.0) / 100.0,
        currency: webhook_data["data"]["object"]["currency"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        status: webhook_data["data"]["object"]["status"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        raw_data: webhook_data,
        signature: Some(signature.to_string()),
    };

    match payment_service.process_webhook("stripe", webhook).await {
        Ok(verification) => Ok(Json(serde_json::json!({
            "success": true,
            "payment_id": verification.payment_id,
            "status": format!("{:?}", verification.status),
            "amount": verification.amount
        }))),
        Err(e) => {
            eprintln!("Stripe webhook error: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Process refund
pub async fn process_refund(
    State(state): State<AppState>,
    Json(request): Json<RefundRequest>,
) -> Result<Json<RefundResponse>, StatusCode> {
    let mut payment_service = PaymentService::new(state.pool.clone());
    payment_service.initialize_providers().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let refund_request = crate::routes::payments::provider::RefundRequest {
        payment_id: request.payment_id,
        amount: request.amount,
        reason: request.reason,
    };

    match payment_service.process_refund(&request.provider, refund_request).await {
        Ok(refund_id) => Ok(Json(RefundResponse {
            success: true,
            refund_id,
            message: "Refund processed successfully".to_string(),
        })),
        Err(e) => {
            eprintln!("Refund error: {}", e);
            Ok(Json(RefundResponse {
                success: false,
                refund_id: "".to_string(),
                message: e,
            }))
        }
    }
}

/// Get available payment providers
pub async fn get_providers(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut payment_service = PaymentService::new(state.pool.clone());
    payment_service.initialize_providers().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let providers = payment_service.get_available_providers();
    
    Ok(Json(serde_json::json!({
        "success": true,
        "providers": providers
    })))
}

/// Get payment status
pub async fn get_payment_status(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let provider = params.get("provider").ok_or(StatusCode::BAD_REQUEST)?;
    let payment_id = params.get("payment_id").ok_or(StatusCode::BAD_REQUEST)?;

    let mut payment_service = PaymentService::new(state.pool.clone());
    payment_service.initialize_providers().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get status from database first
    let donation = sqlx::query!(
        "SELECT status, provider_status, amount FROM donations WHERE tx_hash = $1",
        payment_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(donation) = donation {
        Ok(Json(serde_json::json!({
            "success": true,
            "payment_id": payment_id,
            "status": donation.status,
            "provider_status": donation.provider_status,
            "amount": donation.amount
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
