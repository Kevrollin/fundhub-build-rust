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

pub async fn connect(
    State(state): State<crate::state::AppState>, 
    headers: HeaderMap,
    Json(payload): Json<ConnectRequest>
) -> Result<Json<ConnectResponse>, StatusCode> {
    // Extract user ID from JWT token
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Validate wallet exists on Stellar network
    let is_valid = state.stellar
        .validate_wallet(&payload.public_key)
        .await
        .unwrap_or(false);
    
    if !is_valid {
        tracing::warn!("Invalid Stellar wallet: {}", payload.public_key);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if user has a student record
    let student = sqlx::query!(
        r#"SELECT id FROM students WHERE user_id = $1"#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking student: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let student_id = if let Some(student) = student {
        student.id
    } else {
        // Create student record for the user
        let new_student_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO students (id, user_id, school_email, verification_status)
            VALUES ($1, $2, $3, 'verified')
            "#,
            new_student_id,
            user_id,
            "", // Empty school email for now
        )
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error creating student record: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        new_student_id
    };

    // Check if wallet already exists for this student
    let existing = sqlx::query!(
        r#"SELECT id FROM wallets WHERE student_id = $1"#,
        student_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking wallet: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let wallet_id = if let Some(existing_wallet) = existing {
        // Update existing wallet
        sqlx::query!(
            r#"
            UPDATE wallets
            SET public_key = $2, status = 'connected', last_synced_at = NOW()
            WHERE id = $1
            "#,
            existing_wallet.id,
            payload.public_key
        )
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error updating wallet: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        existing_wallet.id
    } else {
        // Create new wallet
        let new_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO wallets (id, student_id, public_key, status, balance, last_synced_at)
            VALUES ($1, $2, $3, 'connected', 0, NOW())
            "#,
            new_id,
            student_id,
            payload.public_key
        )
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error creating wallet: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        new_id
    };

    tracing::info!("Wallet connected successfully for user: {}", user_id);

    Ok(Json(ConnectResponse {
        wallet_id,
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


