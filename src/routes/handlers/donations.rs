use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use uuid::Uuid;

use crate::{
    models::{Donation, DonationStatus, PaymentMethod},
};

#[derive(Debug, Deserialize)]
pub struct InitiateDonationRequest {
    pub donor_id: Option<Uuid>,
    pub project_id: Uuid,
    pub amount_xlm: String,
    pub payment_method: String,
    pub memo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlatformDonationRequest {
    pub amount: f64,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyDonationRequest {
    pub donation_id: Uuid,
    pub tx_hash: String,
}

#[derive(Debug, Serialize)]
pub struct DonationResponse {
    pub donation_id: Uuid,
    pub status: String,
    pub payment_instruction: serde_json::Value,
}

pub async fn initiate(
    State(state): State<crate::state::AppState>,
    Json(payload): Json<InitiateDonationRequest>,
) -> Result<(StatusCode, Json<DonationResponse>), StatusCode> {
    // Get project with contract address
    let project = sqlx::query!(
        r#"
        SELECT id, student_id, contract_address, status
        FROM projects 
        WHERE id = $1
        "#,
        payload.project_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Check project is active
    if project.status != "active" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get student's wallet for escrow address
    let wallet = sqlx::query!(
        r#"
        SELECT public_key FROM wallets WHERE student_id = $1
        "#,
        project.student_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parse amount to BigDecimal
    let amount: BigDecimal = payload.amount_xlm
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Generate donation ID and use as memo
    let donation_id = Uuid::new_v4();
    let memo = format!("donation:{}", donation_id);

    // Create donation record
    let _donation = sqlx::query!(
        r#"
        INSERT INTO donations (
            id,
            donor_id,
            project_id,
            amount,
            payment_method,
            memo,
            status
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'pending')
        RETURNING id
        "#,
        donation_id,
        payload.donor_id,
        payload.project_id,
        amount,
        payload.payment_method,
        memo,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Build payment instruction based on payment method
    let payment_instruction = match payload.payment_method.as_str() {
        "stellar" => {
            let destination = wallet
                .map(|w| w.public_key)
                .or(project.contract_address)
                .unwrap_or_else(|| std::env::var("PLATFORM_WALLET_PUBLIC_KEY").unwrap_or_default());

            serde_json::json!({
                "destination": destination,
                "amount_xlm": payload.amount_xlm,
                "memo": memo,
                "memo_type": "text"
            })
        }
        "mpesa" | "card" => {
            serde_json::json!({
                "checkout_url": format!("/checkout/{}", donation_id),
                "amount": payload.amount_xlm
            })
        }
        _ => serde_json::json!({})
    };

    Ok((StatusCode::CREATED, Json(DonationResponse {
        donation_id,
        status: "pending".to_string(),
        payment_instruction,
    })))
}

pub async fn verify(
    State(state): State<crate::state::AppState>,
    Json(payload): Json<VerifyDonationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get donation
    let donation = sqlx::query!(
        r#"
        SELECT id, project_id, amount, memo, status
        FROM donations
        WHERE id = $1
        "#,
        payload.donation_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if donation.status != "pending" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Verify transaction on Stellar network
    let is_valid = state.stellar
        .verify_transaction(&payload.tx_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !is_valid {
        // Mark as failed
        sqlx::query!(
            r#"
            UPDATE donations
            SET status = 'failed', tx_hash = $2
            WHERE id = $1
            "#,
            payload.donation_id,
            payload.tx_hash
        )
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Err(StatusCode::BAD_REQUEST);
    }

    // Update donation status to confirmed
    sqlx::query!(
        r#"
        UPDATE donations
        SET status = 'confirmed', 
            tx_hash = $2,
            confirmed_at = NOW()
        WHERE id = $1
        "#,
        payload.donation_id,
        payload.tx_hash
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Emit SSE notification
    let _ = state.notifier.send(format!(
        "donation_confirmed:{}:{}",
        donation.project_id.map(|id| id.to_string()).unwrap_or_else(|| "platform".to_string()),
        payload.donation_id
    ));

    Ok(Json(serde_json::json!({
        "donation_id": payload.donation_id,
        "status": "confirmed",
        "tx_hash": payload.tx_hash
    })))
}

pub async fn get_project_donations(
    State(state): State<crate::state::AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Donation>>, StatusCode> {
    let donations = sqlx::query_as!(
        Donation,
        r#"
        SELECT id, donor_id, project_id, amount, tx_hash, memo,
               status, payment_method, donation_type, confirmed_at, created_at
        FROM donations
        WHERE project_id = $1
        ORDER BY created_at DESC
        "#,
        project_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(donations))
}

pub async fn get_student_donations(
    State(state): State<crate::state::AppState>,
    Path(student_id): Path<Uuid>,
) -> Result<Json<Vec<Donation>>, StatusCode> {
    let donations = sqlx::query_as!(
        Donation,
        r#"
        SELECT d.id, d.donor_id, d.project_id, d.amount, d.tx_hash, d.memo,
               d.status, d.payment_method, d.donation_type, d.confirmed_at, d.created_at
        FROM donations d
        JOIN projects p ON p.id = d.project_id
        WHERE p.student_id = $1
        ORDER BY d.created_at DESC
        "#,
        student_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(donations))
}

pub async fn initiate_platform_donation(
    State(state): State<crate::state::AppState>,
    Json(payload): Json<PlatformDonationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    // Platform wallet address (this should be configured in environment)
    let platform_wallet = std::env::var("PLATFORM_WALLET_ADDRESS")
        .unwrap_or_else(|_| "GALAXY_PLATFORM_WALLET_ADDRESS".to_string());
    
    // Parse amount to BigDecimal
    let amount: BigDecimal = payload.amount
        .to_string()
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Generate donation ID and use as memo
    let donation_id = Uuid::new_v4();
    let memo = format!("platform_donation:{}", donation_id);

    // Create platform donation record (project_id = NULL for platform donations)
    let _donation = sqlx::query!(
        r#"
        INSERT INTO donations (
            id,
            donor_id,
            project_id,
            amount,
            payment_method,
            memo,
            status,
            donation_type
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'pending', 'platform')
        RETURNING id
        "#,
        donation_id,
        None::<Uuid>, // No donor_id for platform donations
        None::<Uuid>, // No project_id for platform donations
        amount,
        "platform_fund",
        memo,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Build payment instruction for platform donation
    let payment_instruction = serde_json::json!({
        "type": "platform_donation",
        "recipient_wallet": platform_wallet,
        "amount_xlm": payload.amount,
        "memo": memo,
        "donation_id": donation_id,
        "message": payload.message
    });

    Ok((StatusCode::OK, Json(serde_json::json!({
        "donation_id": donation_id,
        "status": "pending",
        "payment_instruction": payment_instruction,
        "tx_hash": null // Will be set when transaction is submitted
    }))))
}