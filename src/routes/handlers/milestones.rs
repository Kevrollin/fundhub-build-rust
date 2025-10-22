use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use crate::{
    models::{Milestone, MilestoneReleaseRequest},
    state::AppState,
    utils::jwt,
};

/// Create a milestone for a project
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/milestones",
    request_body = serde_json::Value,
    responses(
        (status = 201, description = "Milestone created successfully", body = Milestone),
        (status = 400, description = "Invalid request data"),
        (status = 403, description = "Forbidden - not project owner"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Milestones"
)]
pub async fn create_milestone(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Milestone>), (StatusCode, Json<serde_json::Value>)> {
    // User verification is handled by the middleware

    let title = payload.get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Title is required"})),
            )
        })?;

    let target_amount: sqlx::types::BigDecimal = payload.get("target_amount")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Target amount is required"})),
            )
        })?;

    // Create milestone
    let milestone = sqlx::query_as!(
        Milestone,
        r#"
        INSERT INTO milestones (project_id, title, target_amount)
        VALUES ($1, $2, $3)
        RETURNING id, project_id, title, target_amount as "target_amount!: sqlx::types::BigDecimal", released as "released!: bool", released_at, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        "#,
        project_id,
        title,
        target_amount
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create milestone"})),
        )
    })?;

    // Log activity
    let _ = sqlx::query!(
        r#"
        INSERT INTO activity_logs (action, target_id, target_type, metadata)
        VALUES ($1, $2, $3, $4)
        "#,
        "milestone_created",
        milestone.id,
        "milestone",
        serde_json::json!({
            "project_id": project_id,
            "title": title,
            "target_amount": target_amount
        })
    )
    .execute(&state.pool)
    .await;

    Ok((StatusCode::CREATED, Json(milestone)))
}

/// Get milestones for a project
#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/milestones",
    responses(
        (status = 200, description = "Milestones retrieved successfully", body = Vec<Milestone>),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Milestones"
)]
pub async fn get_project_milestones(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Milestone>>, (StatusCode, Json<serde_json::Value>)> {
    let milestones = sqlx::query_as!(
        Milestone,
        r#"
        SELECT id, project_id, title, target_amount as "target_amount!: sqlx::types::BigDecimal", released as "released!: bool", released_at, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM milestones
        WHERE project_id = $1
        ORDER BY created_at ASC
        "#,
        project_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch milestones"})),
        )
    })?;

    Ok(Json(milestones))
}

/// Release a milestone (admin only)
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/milestones/{milestone_id}/release",
    request_body = MilestoneReleaseRequest,
    responses(
        (status = 200, description = "Milestone released successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 403, description = "Forbidden - admin only"),
        (status = 404, description = "Milestone not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Milestones"
)]
pub async fn release_milestone(
    State(state): State<AppState>,
    Path((project_id, milestone_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<MilestoneReleaseRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Admin verification is handled by the middleware

    // Get milestone details
    let milestone = sqlx::query_as!(
        Milestone,
        r#"
        SELECT id, project_id, title, target_amount as "target_amount!: sqlx::types::BigDecimal", released as "released!: bool", released_at, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM milestones
        WHERE id = $1 AND project_id = $2
        "#,
        milestone_id,
        project_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Milestone not found"})),
        )
    })?;

    if milestone.released {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Milestone already released"})),
        ));
    }

    // Update milestone as released
    let _ = sqlx::query!(
        r#"
        UPDATE milestones 
        SET released = true, released_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
        milestone_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to release milestone"})),
        )
    })?;

    // Log activity
    let _ = sqlx::query!(
        r#"
        INSERT INTO activity_logs (action, target_id, target_type, metadata)
        VALUES ($1, $2, $3, $4)
        "#,
        "milestone_released",
        milestone_id,
        "milestone",
        serde_json::json!({
            "project_id": project_id,
            "tx_hash": payload.tx_hash,
            "amount": milestone.target_amount
        })
    )
    .execute(&state.pool)
    .await;

    Ok(Json(serde_json::json!({
        "message": "Milestone released successfully",
        "milestone_id": milestone_id,
        "tx_hash": payload.tx_hash
    })))
}
