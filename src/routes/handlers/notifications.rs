use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Serialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

pub async fn get_notifications(
    State(state): State<crate::state::AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<NotificationResponse>>, StatusCode> {
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let notifications = sqlx::query_as!(
        NotificationResponse,
        r#"
        SELECT 
            id,
            user_id,
            notification_type,
            title,
            message,
            is_read,
            metadata,
            created_at,
            updated_at
        FROM notifications 
        WHERE user_id = $1 
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(notifications))
}

pub async fn mark_notification_read(
    State(state): State<crate::state::AppState>,
    headers: axum::http::HeaderMap,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let result = sqlx::query!(
        r#"
        UPDATE notifications 
        SET is_read = true, updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        "#,
        notification_id,
        user_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({
        "message": "Notification marked as read"
    })))
}

pub async fn mark_all_notifications_read(
    State(state): State<crate::state::AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    sqlx::query!(
        r#"
        UPDATE notifications 
        SET is_read = true, updated_at = NOW()
        WHERE user_id = $1 AND is_read = false
        "#,
        user_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "message": "All notifications marked as read"
    })))
}

pub async fn delete_notification(
    State(state): State<crate::state::AppState>,
    headers: axum::http::HeaderMap,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let result = sqlx::query!(
        r#"
        DELETE FROM notifications 
        WHERE id = $1 AND user_id = $2
        "#,
        notification_id,
        user_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({
        "message": "Notification deleted"
    })))
}

pub async fn create_notification(
    State(state): State<crate::state::AppState>,
    Json(req): Json<CreateNotificationRequest>,
) -> Result<Json<NotificationResponse>, StatusCode> {
    let notification = sqlx::query_as!(
        NotificationResponse,
        r#"
        INSERT INTO notifications (
            id, user_id, notification_type, title, message, metadata, is_read, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, false, NOW())
        RETURNING id, user_id, notification_type, title, message, is_read, metadata, created_at, updated_at
        "#,
        Uuid::new_v4(),
        req.user_id,
        req.notification_type,
        req.title,
        req.message,
        req.metadata
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(notification))
}

pub async fn get_unread_count(
    State(state): State<crate::state::AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM notifications 
        WHERE user_id = $1 AND is_read = false
        "#,
        user_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "unread_count": count.count.unwrap_or(0)
    })))
}
