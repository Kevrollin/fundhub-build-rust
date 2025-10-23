use axum::{extract::{State, Json, Path}, http::StatusCode};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::{
    Student, StudentVerification, VerificationStatus, StudentProfile, VerificationHistory,
    EnhancedStudentVerificationRequest, ApproveVerificationRequest, RejectVerificationRequest, VerificationResponse
};

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

// These structs are now imported from the models module

#[derive(Serialize)]
pub struct PendingVerification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
    pub school_email: Option<String>,
    pub full_name: Option<String>,
    pub school_name: Option<String>,
    pub student_bio: Option<String>,
    pub motivation_text: Option<String>,
    pub admission_number: Option<String>,
    pub verification_status: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct EnhancedVerificationList {
    pub verifications: Vec<PendingVerification>,
    pub total_count: i64,
    pub pending_count: i64,
    pub verified_count: i64,
    pub rejected_count: i64,
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
    // Get all pending verifications with user details from the new student_verifications table
    let rows = sqlx::query!(
        r#"
        SELECT sv.id, sv.user_id, u.username, u.email, sv.school_email,
               sv.full_name, sv.school_name, sv.student_bio, sv.motivation_text,
               NULL::text as admission_number, sv.status as verification_status, 
               sv.submitted_at, sv.created_at
        FROM student_verifications sv
        JOIN users u ON u.id = sv.user_id
        WHERE sv.status = 'pending'
        ORDER BY sv.created_at ASC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching pending verifications: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let verifications = rows.into_iter().map(|row| PendingVerification {
        id: row.id,
        user_id: row.user_id,
        username: Some(row.username),
        email: Some(row.email),
        school_email: Some(row.school_email),
        full_name: row.full_name,
        school_name: row.school_name,
        student_bio: row.student_bio,
        motivation_text: row.motivation_text,
        admission_number: row.admission_number,
        verification_status: row.verification_status,
        submitted_at: row.submitted_at,
        created_at: row.created_at,
    }).collect();

    Ok(Json(verifications))
}

pub async fn list_all_verifications(
    State(state): State<crate::state::AppState>
) -> Result<Json<Vec<PendingVerification>>, StatusCode> {
    // Get all verifications (pending, approved, rejected) with user details from the new student_verifications table
    let rows = sqlx::query!(
        r#"
        SELECT sv.id, sv.user_id, u.username, u.email, sv.school_email,
               sv.full_name, sv.school_name, sv.student_bio, sv.motivation_text,
               NULL::text as admission_number, sv.status as verification_status, 
               sv.submitted_at, sv.created_at
        FROM student_verifications sv
        JOIN users u ON u.id = sv.user_id
        ORDER BY sv.created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching all verifications: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let verifications = rows.into_iter().map(|row| PendingVerification {
        id: row.id,
        user_id: row.user_id,
        username: Some(row.username),
        email: Some(row.email),
        school_email: Some(row.school_email),
        full_name: row.full_name,
        school_name: row.school_name,
        student_bio: row.student_bio,
        motivation_text: row.motivation_text,
        admission_number: row.admission_number,
        verification_status: row.verification_status,
        submitted_at: row.submitted_at,
        created_at: row.created_at,
    }).collect();

    Ok(Json(verifications))
}

pub async fn list_approved_verifications(
    State(state): State<crate::state::AppState>
) -> Result<Json<Vec<PendingVerification>>, StatusCode> {
    // Get all approved verifications from the new student_verifications table
    let rows = sqlx::query!(
        r#"
        SELECT sv.id, sv.user_id, u.username, u.email, sv.school_email,
               sv.full_name, sv.school_name, sv.student_bio, sv.motivation_text,
               NULL::text as admission_number, sv.status as verification_status, 
               sv.submitted_at, sv.created_at
        FROM student_verifications sv
        JOIN users u ON u.id = sv.user_id
        WHERE sv.status = 'verified'
        ORDER BY sv.created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching approved verifications: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let verifications = rows.into_iter().map(|row| PendingVerification {
        id: row.id,
        user_id: row.user_id,
        username: Some(row.username),
        email: Some(row.email),
        school_email: Some(row.school_email),
        full_name: row.full_name,
        school_name: row.school_name,
        student_bio: row.student_bio,
        motivation_text: row.motivation_text,
        admission_number: row.admission_number,
        verification_status: row.verification_status,
        submitted_at: row.submitted_at,
        created_at: row.created_at,
    }).collect();

    Ok(Json(verifications))
}

pub async fn list_rejected_verifications(
    State(state): State<crate::state::AppState>
) -> Result<Json<Vec<PendingVerification>>, StatusCode> {
    // Get all rejected verifications from the new student_verifications table
    let rows = sqlx::query!(
        r#"
        SELECT sv.id, sv.user_id, u.username, u.email, sv.school_email,
               sv.full_name, sv.school_name, sv.student_bio, sv.motivation_text,
               NULL::text as admission_number, sv.status as verification_status, 
               sv.submitted_at, sv.created_at
        FROM student_verifications sv
        JOIN users u ON u.id = sv.user_id
        WHERE sv.status = 'rejected'
        ORDER BY sv.created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching rejected verifications: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let verifications = rows.into_iter().map(|row| PendingVerification {
        id: row.id,
        user_id: row.user_id,
        username: Some(row.username),
        email: Some(row.email),
        school_email: Some(row.school_email),
        full_name: row.full_name,
        school_name: row.school_name,
        student_bio: row.student_bio,
        motivation_text: row.motivation_text,
        admission_number: row.admission_number,
        verification_status: row.verification_status,
        submitted_at: row.submitted_at,
        created_at: row.created_at,
    }).collect();

    Ok(Json(verifications))
}

pub async fn approve_verification(
    State(state): State<crate::state::AppState>,
    Path(verification_id): Path<Uuid>,
    Json(req): Json<ApproveVerificationRequest>,
) -> Result<Json<VerificationResponse>, StatusCode> {
    // First get the verification details
    let verification = sqlx::query!(
        r#"
        SELECT user_id, school_email FROM student_verifications WHERE id = $1
        "#,
        verification_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Update student verification status in the new student_verifications table
    let result = sqlx::query!(
        r#"
        UPDATE student_verifications
        SET status = 'verified',
            admin_message = $2,
            approved_at = NOW()
        WHERE id = $1
        RETURNING id, user_id, approved_at
        "#,
        verification_id,
        req.message
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update user role to student
    sqlx::query!(
        r#"
        UPDATE users
        SET base_role = 'student', is_verified = true
        WHERE id = $1
        "#,
        result.user_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create or update student record
    sqlx::query!(
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
        result.user_id,
        verification.school_email,
        req.admin_id
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
        verified_at: result.approved_at,
    }))
}

pub async fn reject_verification(
    State(state): State<crate::state::AppState>,
    Path(verification_id): Path<Uuid>,
    Json(req): Json<RejectVerificationRequest>,
) -> Result<Json<VerificationResponse>, StatusCode> {
    // Update student verification status in the new student_verifications table
    let result = sqlx::query!(
        r#"
        UPDATE student_verifications
        SET status = 'rejected',
            admin_message = $2
        WHERE id = $1
        RETURNING id, user_id
        "#,
        verification_id,
        req.reason
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Emit SSE notification with rejection message
    let _ = state.notifier.send(format!(
        "verification_status:{}:rejected:{}", 
        result.user_id,
        req.reason
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
        SELECT id, user_id, school_email, full_name, school_name, student_bio, motivation_text,
               status as "status!: VerificationStatus", admin_message, approved_at, submitted_at, 
               created_at as "created_at!: chrono::DateTime<chrono::Utc>"
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
    // Get comprehensive platform statistics
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
        "SELECT COUNT(*) FROM students WHERE verification_status = 'pending'"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    let rejected_verifications = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM students WHERE verification_status = 'rejected'"
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
        "rejected_verifications": rejected_verifications,
        "total_projects": total_projects,
        "total_donations": total_donations
    });

    Ok(Json(overview))
}

/// Get enhanced verification list with detailed information
#[utoipa::path(
    get,
    path = "/api/admin/verifications/enhanced",
    responses(
        (status = 200, description = "Enhanced verification list retrieved successfully", body = EnhancedVerificationList),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_enhanced_verifications(
    State(state): State<crate::state::AppState>,
) -> Result<Json<EnhancedVerificationList>, (StatusCode, Json<serde_json::Value>)> {
    let verifications = sqlx::query_as!(
        PendingVerification,
        r#"
        SELECT 
            sv.id, sv.user_id, u.username, u.email, sv.school_email,
            sv.full_name, sv.school_name, sv.student_bio, sv.motivation_text,
            s.admission_number, sv.status as verification_status,
            sv.submitted_at, sv.created_at
        FROM student_verifications sv
        JOIN users u ON sv.user_id = u.id
        LEFT JOIN students s ON sv.user_id = s.user_id
        ORDER BY sv.created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    let total_count = verifications.len() as i64;
    let pending_count = verifications.iter().filter(|v| v.verification_status.as_deref() == Some("pending")).count() as i64;
    let verified_count = verifications.iter().filter(|v| v.verification_status.as_deref() == Some("verified")).count() as i64;
    let rejected_count = verifications.iter().filter(|v| v.verification_status.as_deref() == Some("rejected")).count() as i64;

    Ok(Json(EnhancedVerificationList {
        verifications,
        total_count,
        pending_count,
        verified_count,
        rejected_count,
    }))
}

/// Get detailed verification information
#[utoipa::path(
    get,
    path = "/api/admin/verifications/{verification_id}/details",
    responses(
        (status = 200, description = "Verification details retrieved successfully", body = PendingVerification),
        (status = 404, description = "Verification not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_verification_details(
    State(state): State<crate::state::AppState>,
    Path(verification_id): Path<Uuid>,
) -> Result<Json<PendingVerification>, (StatusCode, Json<serde_json::Value>)> {
    let verification = sqlx::query_as!(
        PendingVerification,
        r#"
        SELECT 
            sv.id, sv.user_id, u.username, u.email, sv.school_email,
            sv.full_name, sv.school_name, sv.student_bio, sv.motivation_text,
            s.admission_number, sv.status as verification_status,
            sv.submitted_at, sv.created_at
        FROM student_verifications sv
        JOIN users u ON sv.user_id = u.id
        LEFT JOIN students s ON sv.user_id = s.user_id
        WHERE sv.id = $1
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

    Ok(Json(verification))
}

/// Enhanced approve verification with detailed tracking
#[utoipa::path(
    post,
    path = "/api/admin/verifications/{verification_id}/approve-enhanced",
    request_body = ApproveVerificationRequest,
    responses(
        (status = 200, description = "Verification approved successfully", body = VerificationResponse),
        (status = 404, description = "Verification not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn approve_verification_enhanced(
    State(state): State<crate::state::AppState>,
    Path(verification_id): Path<Uuid>,
    Json(payload): Json<ApproveVerificationRequest>,
) -> Result<Json<VerificationResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Get verification details
    let verification = sqlx::query!(
        r#"
        SELECT user_id, school_email, full_name, school_name, student_bio, motivation_text FROM student_verifications WHERE id = $1
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

    // Update student verification status
    let result = sqlx::query!(
        r#"
        UPDATE student_verifications
        SET status = 'verified',
            admin_message = $2,
            approved_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING id, user_id, approved_at
        "#,
        verification_id,
        payload.message
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to update verification"})),
        )
    })?;

    // Update user role and verification status
    sqlx::query!(
        r#"
        UPDATE users
        SET role = 'student', base_role = 'student', is_verified = true, verification_status = 'verified',
            verification_approved_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
        result.user_id
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
    sqlx::query!(
        r#"
        INSERT INTO students (user_id, school_email, verification_status)
        VALUES ($1, $2, 'verified')
        ON CONFLICT (school_email) 
        DO UPDATE SET 
            user_id = $1,
            verification_status = 'verified'
        "#,
        result.user_id,
        verification.school_email
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        eprintln!("Database error creating student record: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to create student record: {}", e)})),
        )
    })?;

    // Create or update student profile
    let profile_result = sqlx::query!(
        r#"
        INSERT INTO student_profiles (user_id, full_name, school_name, school_email, student_bio, motivation_text)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        result.user_id,
        verification.full_name,
        verification.school_name,
        verification.school_email,
        verification.student_bio,
        verification.motivation_text
    )
    .execute(&state.pool)
    .await;

    match profile_result {
        Ok(result) => eprintln!("Student profile created/updated: {} rows affected", result.rows_affected()),
        Err(e) => eprintln!("Error creating student profile: {:?}", e),
    }

    // Add to verification history
    let _ = sqlx::query!(
        r#"
        INSERT INTO verification_history (user_id, verification_id, status, admin_message, admin_id)
        VALUES ($1, $2, 'verified', $3, $4)
        "#,
        result.user_id,
        verification_id,
        payload.message,
        payload.admin_id
    )
    .execute(&state.pool)
    .await;

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
            "student_user_id": result.user_id,
            "school_email": verification.school_email,
            "full_name": verification.full_name,
            "school_name": verification.school_name
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

/// Enhanced reject verification with detailed tracking
#[utoipa::path(
    post,
    path = "/api/admin/verifications/{verification_id}/reject-enhanced",
    request_body = RejectVerificationRequest,
    responses(
        (status = 200, description = "Verification rejected successfully", body = VerificationResponse),
        (status = 404, description = "Verification not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn reject_verification_enhanced(
    State(state): State<crate::state::AppState>,
    Path(verification_id): Path<Uuid>,
    Json(payload): Json<RejectVerificationRequest>,
) -> Result<Json<VerificationResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Get verification details
    let verification = sqlx::query!(
        r#"
        SELECT user_id, school_email FROM student_verifications WHERE id = $1
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

    // Update verification status to rejected
    let _ = sqlx::query!(
        r#"
        UPDATE student_verifications
        SET status = 'rejected',
            admin_message = $2
        WHERE id = $1
        "#,
        verification_id,
        payload.reason
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to update verification"})),
        )
    })?;

    // Update user verification status
    sqlx::query!(
        r#"
        UPDATE users
        SET verification_status = 'rejected'
        WHERE id = $1
        "#,
        verification.user_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to update user status"})),
        )
    })?;

    // Add to verification history
    let _ = sqlx::query!(
        r#"
        INSERT INTO verification_history (user_id, verification_id, status, admin_message)
        VALUES ($1, $2, 'rejected', $3)
        "#,
        verification.user_id,
        verification_id,
        payload.reason
    )
    .execute(&state.pool)
    .await;

    // Log activity
    let _ = sqlx::query!(
        r#"
        INSERT INTO activity_logs (user_id, action, target_id, target_type, metadata)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        verification.user_id,
        "student_verification_rejected",
        verification_id,
        "student_verification",
        serde_json::json!({
            "school_email": verification.school_email,
            "reason": payload.reason
        })
    )
    .execute(&state.pool)
    .await;

    Ok(Json(VerificationResponse {
        verification_id,
        status: "rejected".to_string(),
        verified_at: None,
    }))
}


