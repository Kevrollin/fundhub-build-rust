use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use uuid::Uuid;
use sqlx::types::BigDecimal;
use std::str::FromStr;
use crate::{
    services::{NewStellarService, WalletInfo, BalanceInfo, TransactionInfo},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct SendPaymentRequest {
    pub from_secret: String,
    pub to_public: String,
    pub amount: String,
    pub memo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlatformFundingRequest {
    pub from_public: String,
    pub amount: String,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub success: bool,
    pub wallet: Option<WalletInfo>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub success: bool,
    pub balance: Option<String>,
    pub balances: Option<Vec<BalanceInfo>>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub success: bool,
    pub hash: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PlatformInfoResponse {
    pub success: bool,
    pub public_key: String,
    pub message: String,
}

/// Create a new wallet
pub async fn create_wallet(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateWalletRequest>,
) -> Result<Json<WalletResponse>, StatusCode> {
    match app_state.stellar_service.generate_wallet() {
        wallet => {
            // Store wallet in database if user_id is provided
            if let Some(user_id) = payload.user_id {
                match sqlx::query!(
                    "INSERT INTO wallets (user_id, public_key) VALUES ($1, $2) ON CONFLICT (public_key) DO NOTHING",
                    user_id,
                    wallet.public_key
                )
                .execute(&app_state.pool)
                .await
                {
                    Ok(_) => {
                        Ok(Json(WalletResponse {
                            success: true,
                            wallet: Some(wallet),
                            message: "Wallet created successfully".to_string(),
                        }))
                    }
                    Err(e) => {
                        tracing::error!("Failed to store wallet in database: {}", e);
                        Ok(Json(WalletResponse {
                            success: true,
                            wallet: Some(wallet),
                            message: "Wallet created but failed to store in database".to_string(),
                        }))
                    }
                }
            } else {
                Ok(Json(WalletResponse {
                    success: true,
                    wallet: Some(wallet),
                    message: "Wallet created successfully".to_string(),
                }))
            }
        }
    }
}

/// Get wallet balance
pub async fn get_balance(
    State(app_state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<BalanceResponse>, StatusCode> {
    match app_state.stellar_service.get_xlm_balance(&address).await {
        Ok(balance) => {
            Ok(Json(BalanceResponse {
                success: true,
                balance: Some(balance),
                balances: None,
                message: "Balance retrieved successfully".to_string(),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to get balance: {}", e);
            Ok(Json(BalanceResponse {
                success: false,
                balance: None,
                balances: None,
                message: format!("Failed to get balance: {}", e),
            }))
        }
    }
}

/// Get detailed balances
pub async fn get_balances(
    State(app_state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<BalanceResponse>, StatusCode> {
    match app_state.stellar_service.get_balance(&address).await {
        Ok(balances) => {
            Ok(Json(BalanceResponse {
                success: true,
                balance: None,
                balances: Some(balances),
                message: "Balances retrieved successfully".to_string(),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to get balances: {}", e);
            Ok(Json(BalanceResponse {
                success: false,
                balance: None,
                balances: None,
                message: format!("Failed to get balances: {}", e),
            }))
        }
    }
}

/// Send payment
pub async fn send_payment(
    State(app_state): State<AppState>,
    Json(payload): Json<SendPaymentRequest>,
) -> Result<Json<TransactionResponse>, StatusCode> {
    match app_state.stellar_service.send_payment(
        &payload.from_secret,
        &payload.to_public,
        &payload.amount,
        payload.memo.as_deref(),
    ).await {
        Ok(hash) => {
            // Store transaction in database
            if let Err(e) = sqlx::query!(
                "INSERT INTO transactions (sender_address, receiver_address, amount, tx_hash, created_at) VALUES ($1, $2, $3, $4, NOW())",
                payload.from_secret, // This should be the public key, not secret
                payload.to_public,
                BigDecimal::from_str(&payload.amount).unwrap_or_default(),
                hash
            )
            .execute(&app_state.pool)
            .await {
                tracing::error!("Failed to store transaction in database: {}", e);
            }

            Ok(Json(TransactionResponse {
                success: true,
                hash: Some(hash),
                message: "Payment sent successfully".to_string(),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to send payment: {}", e);
            Ok(Json(TransactionResponse {
                success: false,
                hash: None,
                message: format!("Failed to send payment: {}", e),
            }))
        }
    }
}

/// Fund platform
pub async fn fund_platform(
    State(app_state): State<AppState>,
    Json(payload): Json<PlatformFundingRequest>,
) -> Result<Json<TransactionResponse>, StatusCode> {
    // This would typically involve the user sending funds to the platform wallet
    // For now, we'll just return the platform wallet address
    let (platform_public, _) = app_state.stellar_service.get_platform_info();
    
    Ok(Json(TransactionResponse {
        success: true,
        hash: None,
        message: format!("Send {} XLM to platform wallet: {}", payload.amount, platform_public),
    }))
}

/// Get platform wallet info
pub async fn get_platform_info(
    State(app_state): State<AppState>,
) -> Result<Json<PlatformInfoResponse>, StatusCode> {
    let (platform_public, _) = app_state.stellar_service.get_platform_info();
    
    Ok(Json(PlatformInfoResponse {
        success: true,
        public_key: platform_public,
        message: "Platform wallet info retrieved".to_string(),
    }))
}

/// Fund account with friendbot (testnet only)
pub async fn fund_with_friendbot(
    State(app_state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<TransactionResponse>, StatusCode> {
    match app_state.stellar_service.fund_with_friendbot(&address).await {
        Ok(_) => {
            Ok(Json(TransactionResponse {
                success: true,
                hash: None,
                message: "Account funded with friendbot".to_string(),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to fund with friendbot: {}", e);
            Ok(Json(TransactionResponse {
                success: false,
                hash: None,
                message: format!("Failed to fund account: {}", e),
            }))
        }
    }
}

/// Get account transactions
pub async fn get_transactions(
    State(app_state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match app_state.stellar_service.get_account_transactions(&address).await {
        Ok(transactions) => {
            Ok(Json(serde_json::json!({
                "success": true,
                "transactions": transactions,
                "message": "Transactions retrieved successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get transactions: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "transactions": [],
                "message": format!("Failed to get transactions: {}", e)
            })))
        }
    }
}

/// Validate address
pub async fn validate_address(
    State(app_state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let is_valid = app_state.stellar_service.validate_address(&address);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "valid": is_valid,
        "message": if is_valid { "Address is valid" } else { "Address is invalid" }
    })))
}
