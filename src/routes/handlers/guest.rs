use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use sqlx::types::BigDecimal;
use crate::{
    models::{GuestDonation, GuestFundingRequest},
    state::AppState,
    utils::jwt::Claims,
};

/// Create a guest donation
#[utoipa::path(
    post,
    path = "/api/guest/fund",
    request_body = GuestFundingRequest,
    responses(
        (status = 201, description = "Guest donation created successfully", body = GuestDonation),
        (status = 400, description = "Invalid request data"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Guest"
)]
pub async fn create_guest_donation(
    State(state): State<AppState>,
    Json(payload): Json<GuestFundingRequest>,
) -> Result<(StatusCode, Json<GuestDonation>), (StatusCode, Json<serde_json::Value>)> {
    // Verify project exists
    let project_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1)",
        payload.project_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    if !project_exists.unwrap_or(false) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        ));
    }

    // Create guest donation
    let donation = sqlx::query_as!(
        GuestDonation,
        r#"
        INSERT INTO guest_donations (guest_name, guest_email, project_id, tx_hash, amount)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, guest_name, guest_email, project_id, tx_hash, amount, verified as "verified!: bool", created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        "#,
        payload.guest_name,
        payload.guest_email,
        payload.project_id,
        payload.tx_hash,
        payload.amount
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create guest donation"})),
        )
    })?;

    // Log activity
    let _ = sqlx::query!(
        r#"
        INSERT INTO activity_logs (action, target_id, target_type, metadata)
        VALUES ($1, $2, $3, $4)
        "#,
        "guest_donation_created",
        donation.id,
        "guest_donation",
        serde_json::json!({
            "guest_name": payload.guest_name,
            "amount": payload.amount,
            "project_id": payload.project_id
        })
    )
    .execute(&state.pool)
    .await;

    Ok((StatusCode::CREATED, Json(donation)))
}

/// Verify a guest donation transaction
#[utoipa::path(
    post,
    path = "/api/guest/verify",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Transaction verified successfully"),
        (status = 400, description = "Invalid transaction hash"),
        (status = 404, description = "Donation not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Guest"
)]
pub async fn verify_guest_donation(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tx_hash = payload.get("tx_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Transaction hash is required"})),
            )
        })?;

    // Update donation as verified
    let result = sqlx::query!(
        "UPDATE guest_donations SET verified = true WHERE tx_hash = $1",
        tx_hash
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Donation not found"})),
        ));
    }

    // Log activity
    let _ = sqlx::query!(
        r#"
        INSERT INTO activity_logs (action, target_type, metadata)
        VALUES ($1, $2, $3)
        "#,
        "guest_donation_verified",
        "guest_donation",
        serde_json::json!({"tx_hash": tx_hash})
    )
    .execute(&state.pool)
    .await;

    Ok(Json(serde_json::json!({
        "message": "Transaction verified successfully",
        "tx_hash": tx_hash
    })))
}

/// Get public project information (limited for guests)
#[utoipa::path(
    get,
    path = "/api/guest/projects",
    responses(
        (status = 200, description = "Public projects retrieved successfully", body = Vec<PublicProjectInfo>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Guest"
)]
pub async fn get_public_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::PublicProjectInfo>>, (StatusCode, Json<serde_json::Value>)> {
    let projects = sqlx::query_as!(
        crate::models::PublicProjectInfo,
        r#"
        SELECT 
            p.id,
            p.title,
            LEFT(p.description, 200) as short_description,
            p.media_url,
            p.funding_goal as "funding_goal!: sqlx::types::BigDecimal",
            COALESCE(SUM(d.amount), 0) as "current_funding!: sqlx::types::BigDecimal",
            p.tags,
            p.created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM projects p
        LEFT JOIN donations d ON p.id = d.project_id AND d.status = 'confirmed'
        WHERE p.visibility = 'public' AND p.status = 'active'
        GROUP BY p.id, p.title, p.description, p.media_url, p.funding_goal, p.tags, p.created_at
        ORDER BY p.created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch public projects"})),
        )
    })?;

    Ok(Json(projects))
}
