use axum::{extract::{Path, State}, Json, http::{StatusCode, HeaderMap}};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::types::BigDecimal;

#[derive(Serialize)]
pub struct ApiMessage { 
    pub message: String 
}

#[derive(Deserialize)]
pub struct ConnectRequest { 
    pub public_key: String 
}

#[derive(Serialize)]
pub struct ConnectResponse {
    pub wallet_id: Uuid,
    pub status: String,
}

#[derive(Serialize)]
pub struct WalletDetails {
    pub id: Uuid,
    pub public_key: String,
    pub status: String,
    pub balance: Option<BigDecimal>,
    pub last_synced_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn connect(
    State(state): State<crate::state::AppState>, 
    headers: HeaderMap,
    Json(payload): Json<ConnectRequest>
) -> Result<Json<ConnectResponse>, StatusCode> {
    tracing::info!("Wallet connect request for public key: {}", payload.public_key);
    
    // Extract user ID from JWT token
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|e| {
            tracing::error!("JWT extraction failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    
    tracing::info!("User ID extracted: {}", user_id);

    // Validate wallet exists on Stellar network
    tracing::info!("Validating Stellar wallet: {}", payload.public_key);
    let is_valid = state.stellar
        .validate_wallet(&payload.public_key)
        .await
        .unwrap_or(false);
    
    if !is_valid {
        tracing::warn!("Invalid Stellar wallet: {}", payload.public_key);
        return Err(StatusCode::BAD_REQUEST);
    }
    tracing::info!("Stellar wallet validation passed");

    // Check if user already has a wallet
    tracing::info!("Checking for existing wallet for user: {}", user_id);
    let existing_wallet = sqlx::query!(
        r#"SELECT id FROM wallets WHERE user_id = $1"#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking wallet: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(wallet) = existing_wallet {
        tracing::info!("Found existing wallet: {}", wallet.id);
        // Update existing wallet
        sqlx::query!(
            r#"
            UPDATE wallets
            SET public_key = $2, status = 'connected', last_synced_at = NOW()
            WHERE id = $1
            "#,
            wallet.id,
            payload.public_key
        )
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error updating wallet: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        tracing::info!("Wallet updated successfully");
        
        return Ok(Json(ConnectResponse {
            wallet_id: wallet.id,
            status: "connected".to_string(),
        }));
    }

    // Create new wallet for user
    let new_wallet_id = Uuid::new_v4();
    tracing::info!("Creating new wallet: {}", new_wallet_id);
    
    sqlx::query!(
        r#"
        INSERT INTO wallets (id, user_id, public_key, status, balance, last_synced_at)
        VALUES ($1, $2, $3, 'connected', 0, NOW())
        "#,
        new_wallet_id,
        user_id,
        payload.public_key
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating wallet: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    tracing::info!("Wallet created successfully");

    tracing::info!("Wallet connected successfully for user: {}", user_id);

    Ok(Json(ConnectResponse {
        wallet_id: new_wallet_id,
        status: "connected".to_string(),
    }))
}
pub async fn get_balance(State(state): State<crate::state::AppState>, Path(wallet_id): Path<Uuid>) -> Json<serde_json::Value> {
    let rec = sqlx::query!("SELECT public_key FROM wallets WHERE id = $1", wallet_id)
        .fetch_optional(&state.pool).await.ok().flatten();
    if let Some(r) = rec {
        if let Ok(b) = state.stellar.fetch_wallet_balance(&r.public_key).await {
            return Json(serde_json::json!({"xlm": b.xlm, "usdc": b.usdc}));
        }
    }
    Json(serde_json::json!({"xlm": 0.0, "usdc": 0.0}))
}

pub async fn get_wallet_details(
    State(state): State<crate::state::AppState>,
    headers: HeaderMap,
    Path(wallet_id): Path<Uuid>
) -> Result<Json<WalletDetails>, StatusCode> {
    tracing::info!("Getting wallet details for wallet_id: {}", wallet_id);

    // Extract user ID from JWT token
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|e| {
            tracing::error!("JWT extraction failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    // Get wallet details from database
    let wallet = sqlx::query!(
        r#"
        SELECT id, public_key, status, balance, last_synced_at
        FROM wallets 
        WHERE id = $1 AND user_id = $2
        "#,
        wallet_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching wallet: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match wallet {
        Some(wallet) => {
            tracing::info!("Wallet found: {}", wallet.id);
            Ok(Json(WalletDetails {
                id: wallet.id,
                public_key: wallet.public_key,
                status: wallet.status,
                balance: wallet.balance,
                last_synced_at: wallet.last_synced_at,
            }))
        }
        None => {
            tracing::warn!("Wallet not found: {}", wallet_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub async fn test_connection() -> Json<serde_json::Value> {
    tracing::info!("Test connection endpoint called");
    Json(serde_json::json!({"status": "ok", "message": "Backend is running"}))
}

pub async fn get_user_wallet(
    State(state): State<crate::state::AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>
) -> Result<Json<Vec<WalletDetails>>, StatusCode> {
    tracing::info!("Getting wallet for user: {}", user_id);

    // Extract user ID from JWT token
    let authenticated_user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|e| {
            tracing::error!("JWT extraction failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    // Users can only access their own wallet
    if authenticated_user_id != user_id {
        tracing::warn!("User {} attempted to access wallet for user {}", authenticated_user_id, user_id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Get user's wallet from database
    let wallets = sqlx::query!(
        r#"
        SELECT id, public_key, status, balance, last_synced_at
        FROM wallets 
        WHERE user_id = $1
        ORDER BY last_synced_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching user wallet: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let wallet_details: Vec<WalletDetails> = wallets.into_iter().map(|wallet| WalletDetails {
        id: wallet.id,
        public_key: wallet.public_key,
        status: wallet.status,
        balance: wallet.balance,
        last_synced_at: wallet.last_synced_at,
    }).collect();

    tracing::info!("Found {} wallet(s) for user {}", wallet_details.len(), user_id);
    Ok(Json(wallet_details))
}
pub async fn get_transactions(State(state): State<crate::state::AppState>, Path(wallet_id): Path<Uuid>) -> Json<serde_json::Value> {
    let rec = sqlx::query!("SELECT public_key FROM wallets WHERE id = $1", wallet_id)
        .fetch_optional(&state.pool).await.ok().flatten();
    if let Some(r) = rec {
        if let Ok(txs) = state.stellar.fetch_wallet_transactions(&r.public_key).await {
            let json: Vec<_> = txs.into_iter().map(|t| serde_json::json!({
                "hash": t.hash,
                "amount": t.amount,
                "asset": t.asset,
                "from": t.from,
                "to": t.to,
                "timestamp": t.timestamp,
            })).collect();
            return Json(serde_json::json!(json));
        }
    }
    Json(serde_json::json!([]))
}

#[derive(Deserialize)]
pub struct VerifyTransactionRequest {
    pub tx_hash: String,
}

#[derive(Serialize)]
pub struct VerifyTransactionResponse {
    pub hash: String,
    pub successful: bool,
    pub ledger_attr: Option<i64>,
    pub created_at: String,
    pub fee_charged: String,
    pub operation_count: i32,
    pub memo: Option<String>,
    pub source_account: String,
    pub network: String,
}

pub async fn verify_transaction(
    State(state): State<crate::state::AppState>,
    Json(payload): Json<VerifyTransactionRequest>
) -> Result<Json<VerifyTransactionResponse>, StatusCode> {
    tracing::info!("Verifying transaction: {}", payload.tx_hash);
    
    // Validate transaction hash format
    if payload.tx_hash.len() != 64 || !payload.tx_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        tracing::warn!("Invalid transaction hash format: {}", payload.tx_hash);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Fetch transaction details from Stellar network
    match state.stellar.fetch_transaction_details(&payload.tx_hash).await {
        Ok(tx_details) => {
            tracing::info!("Transaction verified successfully: {}", payload.tx_hash);
            
            Ok(Json(VerifyTransactionResponse {
                hash: tx_details.hash,
                successful: tx_details.successful,
                ledger_attr: tx_details.ledger_attr,
                created_at: tx_details.created_at,
                fee_charged: tx_details.fee_charged,
                operation_count: tx_details.operation_count,
                memo: tx_details.memo,
                source_account: tx_details.source_account,
                network: "testnet".to_string(), // For now, assume testnet
            }))
        }
        Err(e) => {
            tracing::error!("Failed to verify transaction {}: {}", payload.tx_hash, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}


