use axum::{extract::{State, Json, Path}, http::StatusCode};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::{Student, StudentVerification, VerificationStatus};

#[derive(Serialize)]
pub struct ApiMessage { 
    pub message: String 
}

#[derive(Deserialize)]
pub struct VerifyStudentRequest { 
    pub user_id: Uuid, 
    pub approve: bool, 
    pub message: Option<String> 
}

#[derive(Deserialize)]
pub struct ApproveVerificationRequest {
    pub admin_id: Uuid,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct RejectVerificationRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct VerificationResponse {
    pub verification_id: Uuid,
    pub status: String,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct PendingVerification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub school_email: String,
    pub admission_number: Option<String>,
    pub verification_status: String,
    pub created_at: Option<DateTime<Utc>>,
}

pub async fn list_students(
    State(state): State<crate::state::AppState>
) -> Result<Json<Vec<Student>>, StatusCode> {
    let students = sqlx::query_as!(
        Student,
        r#"
        SELECT id, user_id, school_email, admission_number, 
               verification_status, verification_progress,
               verified_at, verified_by, created_at
        FROM students
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(students))
}

pub async fn list_pending_verifications(
    State(state): State<crate::state::AppState>
) -> Result<Json<Vec<PendingVerification>>, StatusCode> {
    let verifications = sqlx::query_as!(
        PendingVerification,
        r#"
        SELECT s.id, s.user_id, u.username, u.email, s.school_email, 
               s.admission_number, s.verification_status, s.created_at
        FROM students s
        JOIN users u ON u.id = s.user_id
        WHERE s.verification_status = 'pending'
        ORDER BY s.created_at ASC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(verifications))
}

pub async fn approve_verification(
    State(state): State<crate::state::AppState>,
    Path(verification_id): Path<Uuid>,
    Json(req): Json<ApproveVerificationRequest>,
) -> Result<Json<VerificationResponse>, StatusCode> {
    // Update student verification status
    let result = sqlx::query!(
        r#"
        UPDATE students
        SET verification_status = 'verified',
            verification_progress = 100,
            verified_at = NOW(),
            verified_by = $1
        WHERE id = $2
        RETURNING id, user_id, verified_at
        "#,
        req.admin_id,
        verification_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update user role to student
    sqlx::query!(
        r#"
        UPDATE users
        SET role = 'student'
        WHERE id = $1
        "#,
        result.user_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Emit SSE notification
    let _ = state.notifier.send(format!(
        "verification_status:{}:verified", 
        result.user_id
    ));

    Ok(Json(VerificationResponse {
        verification_id: result.id,
        status: "verified".to_string(),
        verified_at: result.verified_at,
    }))
}

pub async fn reject_verification(
    State(state): State<crate::state::AppState>,
    Path(verification_id): Path<Uuid>,
    Json(req): Json<RejectVerificationRequest>,
) -> Result<Json<VerificationResponse>, StatusCode> {
    // Update student verification status
    let result = sqlx::query!(
        r#"
        UPDATE students
        SET verification_status = 'rejected'
        WHERE id = $1
        RETURNING id, user_id
        "#,
        verification_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Emit SSE notification with rejection message
    let _ = state.notifier.send(format!(
        "verification_status:{}:rejected:{}", 
        result.user_id,
        req.message
    ));

    Ok(Json(VerificationResponse {
        verification_id: result.id,
        status: "rejected".to_string(),
        verified_at: None,
    }))
}

pub async fn verify_student(
    State(state): State<crate::state::AppState>, 
    Json(req): Json<VerifyStudentRequest>
) -> Result<Json<ApiMessage>, StatusCode> {
    let status = if req.approve { "verified" } else { "rejected" };
    let progress = if req.approve { 100 } else { 0 };
    
    if req.approve {
        sqlx::query!(
            r#"
            UPDATE students
            SET verification_status = $1,
                verification_progress = $2,
                verified_at = NOW()
            WHERE user_id = $3
            "#,
            status,
            progress,
            req.user_id
        )
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        sqlx::query!(
            r#"
            UPDATE students
            SET verification_status = $1
            WHERE user_id = $2
            "#,
            status,
            req.user_id
        )
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    // Emit SSE
    let _ = state.notifier.send(format!("student_verification:{}:{}", req.user_id, status));
    Ok(Json(ApiMessage { message: "student verification updated".into() }))
}

pub async fn fund_student(
    State(_state): State<crate::state::AppState>
) -> Json<ApiMessage> {
    Json(ApiMessage { 
        message: "admin fund student (stub)".into() 
    })
}

/// Approve a student verification
#[utoipa::path(
    post,
    path = "/api/admin/approve-student/{verification_id}",
    request_body = ApproveVerificationRequest,
    responses(
        (status = 200, description = "Student verification approved successfully", body = VerificationResponse),
        (status = 404, description = "Verification not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn approve_student_verification(
    State(state): State<crate::state::AppState>,
    Path(verification_id): Path<Uuid>,
    Json(payload): Json<ApproveVerificationRequest>,
) -> Result<Json<VerificationResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Get verification details
    let verification = sqlx::query_as!(
        StudentVerification,
        r#"
        SELECT id, user_id, school_email, status as "status!: VerificationStatus", admin_message, approved_at, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM student_verifications
        WHERE id = $1
        "#,
        verification_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Verification not found"})),
        )
    })?;

    if verification.status != crate::models::VerificationStatus::Pending {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Verification is not pending"})),
        ));
    }

    // Update verification status
    let _ = sqlx::query!(
        r#"
        UPDATE student_verifications 
        SET status = 'verified', admin_message = $2, approved_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
        verification_id,
        payload.message
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to approve verification"})),
        )
    })?;

    // Update user role to student
    let _ = sqlx::query!(
        r#"
        UPDATE users 
        SET base_role = 'student', is_verified = true
        WHERE id = $1
        "#,
        verification.user_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to update user role"})),
        )
    })?;

    // Create or update student record
    let _ = sqlx::query!(
        r#"
        INSERT INTO students (user_id, school_email, verification_status, verified_at, verified_by)
        VALUES ($1, $2, 'verified', CURRENT_TIMESTAMP, $3)
        ON CONFLICT (user_id) 
        DO UPDATE SET 
            school_email = $2,
            verification_status = 'verified',
            verified_at = CURRENT_TIMESTAMP,
            verified_by = $3
        "#,
        verification.user_id,
        verification.school_email,
        payload.admin_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create student record"})),
        )
    })?;

    // Log activity
    let _ = sqlx::query!(
        r#"
        INSERT INTO activity_logs (user_id, action, target_id, target_type, metadata)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        payload.admin_id,
        "student_verification_approved",
        verification_id,
        "student_verification",
        serde_json::json!({
            "student_user_id": verification.user_id,
            "school_email": verification.school_email
        })
    )
    .execute(&state.pool)
    .await;

    Ok(Json(VerificationResponse {
        verification_id,
        status: "verified".to_string(),
        verified_at: Some(Utc::now()),
    }))
}

/// Get activity logs for admin oversight
#[utoipa::path(
    get,
    path = "/api/admin/logs",
    responses(
        (status = 200, description = "Activity logs retrieved successfully", body = Vec<ActivityLog>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_activity_logs(
    State(state): State<crate::state::AppState>,
) -> Result<Json<Vec<crate::models::ActivityLog>>, (StatusCode, Json<serde_json::Value>)> {
    let logs = sqlx::query_as!(
        crate::models::ActivityLog,
        r#"
        SELECT id, user_id, action, target_id, target_type, metadata, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM activity_logs
        ORDER BY created_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch activity logs"})),
        )
    })?;

    Ok(Json(logs))
}

/// Get admin overview statistics
#[utoipa::path(
    get,
    path = "/api/admin/overview",
    responses(
        (status = 200, description = "Admin overview retrieved successfully", body = serde_json::Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_admin_overview(
    State(state): State<crate::state::AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get various statistics
    let total_users = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    let verified_students = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM students WHERE verification_status = 'verified'"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    let pending_verifications = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM student_verifications WHERE status = 'pending'"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    let total_projects = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM projects"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    let total_donations = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(amount), 0) FROM donations WHERE status = 'confirmed'"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    let overview = serde_json::json!({
        "total_users": total_users,
        "verified_students": verified_students,
        "pending_verifications": pending_verifications,
        "total_projects": total_projects,
        "total_donations": total_donations
    });

    Ok(Json(overview))
}


