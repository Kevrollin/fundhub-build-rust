use axum::{extract::{Json, State, Path, Multipart}, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use crate::models::{
    Student, StudentVerification, StudentVerificationRequest, EnhancedStudentVerificationRequest,
    VerificationStatus, VerificationStatusResponse, StudentProfile, VerificationHistory,
    ApproveVerificationRequest, RejectVerificationRequest, VerificationResponse
};

#[derive(Serialize)]
pub struct ApiMessage { 
    pub message: String 
}

#[derive(Deserialize)]
pub struct RegisterRequest { 
    pub user_id: Uuid,
    pub school_email: String,
    pub admission_number: String,
    pub document_urls: Option<Vec<String>>, // For pre-uploaded S3/MinIO URLs
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub status: String,
    pub verification_id: Uuid,
}

#[derive(Serialize)]
pub struct StudentStatusResponse {
    pub verification_status: String,
    pub progress: i32,
    pub verified_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct UploadDocumentsRequest {
    pub student_id: Uuid,
    pub document_type: String,
}

pub async fn register(
    State(state): State<crate::state::AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), StatusCode> {
    // Check if user exists
    let user = sqlx::query!(
        r#"SELECT id FROM users WHERE id = $1"#,
        req.user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Check if student already exists
    let existing = sqlx::query!(
        r#"SELECT id FROM students WHERE user_id = $1"#,
        req.user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let student_id = if let Some(existing_student) = existing {
        // Update existing student
        sqlx::query!(
            r#"
            UPDATE students 
            SET school_email = $2, admission_number = $3
            WHERE user_id = $1
            "#,
            req.user_id,
            req.school_email,
            req.admission_number
        )
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        existing_student.id
    } else {
        // Create new student record
        let new_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO students (id, user_id, school_email, admission_number, verification_status, verification_progress)
            VALUES ($1, $2, $3, $4, 'pending', 0)
            "#,
            new_id,
            req.user_id,
            req.school_email,
            req.admission_number
        )
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        new_id
    };

    // If document URLs provided, create file records
    if let Some(urls) = req.document_urls {
        for url in urls {
            let _ = sqlx::query!(
                r#"
                INSERT INTO files (owner_id, entity_type, entity_id, path, filename)
                VALUES ($1, 'student_verification', $2, $3, $4)
                "#,
                user.id,
                student_id,
                url,
                url.split('/').last().unwrap_or("document")
            )
            .execute(&state.pool)
            .await;
        }
    }

    Ok((StatusCode::ACCEPTED, Json(RegisterResponse {
        status: "pending".to_string(),
        verification_id: student_id,
    })))
}

pub async fn get_status(
    State(state): State<crate::state::AppState>,
    Path(user_id): Path<Uuid>
) -> Result<Json<StudentStatusResponse>, StatusCode> {
    let student = sqlx::query!(
        r#"
        SELECT verification_status, verification_progress, verified_at
        FROM students 
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(StudentStatusResponse {
        verification_status: student.verification_status,
        progress: student.verification_progress.unwrap_or(0),
        verified_at: student.verified_at,
    }))
}

pub async fn update(
    State(_state): State<crate::state::AppState>
) -> Json<ApiMessage> {
    Json(ApiMessage { 
        message: "student updated (stub)".into() 
    })
}

// Upload document endpoint (multipart form data)
pub async fn upload_document(
    State(state): State<crate::state::AppState>,
    mut multipart: Multipart,
) -> Result<Json<ApiMessage>, StatusCode> {
    let mut student_id: Option<Uuid> = None;
    let mut document_type: Option<String> = None;
    let mut file_data: Vec<u8> = Vec::new();
    let mut filename: Option<String> = None;

    // Parse multipart form
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let field_name = field.name().unwrap_or("").to_string();
        
        match field_name.as_str() {
            "student_id" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                student_id = Some(text.parse().map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            "document_type" => {
                document_type = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                file_data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec();
            }
            _ => {}
        }
    }

    let student_id = student_id.ok_or(StatusCode::BAD_REQUEST)?;
    let document_type = document_type.ok_or(StatusCode::BAD_REQUEST)?;
    let filename = filename.ok_or(StatusCode::BAD_REQUEST)?;

    // TODO: Upload to MinIO/S3
    // For now, store file path as local reference
    let file_path = format!("/uploads/students/{}/{}", student_id, filename);
    
    // Get student user_id
    let student = sqlx::query!(
        r#"SELECT user_id FROM students WHERE id = $1"#,
        student_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Save file record
    sqlx::query!(
        r#"
        INSERT INTO files (owner_id, entity_type, entity_id, path, filename, size_bytes)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        student.user_id,
        document_type,
        student_id,
        file_path,
        filename,
        file_data.len() as i64
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiMessage {
        message: "Document uploaded successfully".to_string(),
    }))
}

/// Apply for enhanced student verification
#[utoipa::path(
    post,
    path = "/api/students/apply-verification",
    request_body = EnhancedStudentVerificationRequest,
    responses(
        (status = 201, description = "Verification application submitted successfully", body = StudentVerification),
        (status = 400, description = "Invalid request data"),
        (status = 409, description = "Verification already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Students"
)]
pub async fn apply_verification(
    State(state): State<crate::state::AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<EnhancedStudentVerificationRequest>,
) -> Result<(StatusCode, Json<StudentVerification>), (StatusCode, Json<serde_json::Value>)> {
    // Get user ID from the authenticated user (extracted from JWT token)
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid or missing authentication token"})),
            )
        })?;
    // Validate school email format
    if !payload.school_email.ends_with(".edu") && !payload.school_email.ends_with(".ac.ke") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "School email must end with .edu or .ac.ke"})),
        ));
    }

    // user_id is already fetched above

    // Check if verification already exists for this user
    let existing = sqlx::query_scalar!(
        "SELECT id FROM student_verifications WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "Verification already exists for this user"})),
        ));
    }

    // Create verification request with enhanced fields
    let verification = sqlx::query_as!(
        StudentVerification,
        r#"
        INSERT INTO student_verifications (user_id, school_email, full_name, school_name, student_bio, motivation_text, status, submitted_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'pending', CURRENT_TIMESTAMP)
        RETURNING id, user_id, school_email, full_name, school_name, student_bio, motivation_text, status as "status!: VerificationStatus", admin_message, approved_at, submitted_at, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        "#,
        user_id,
        payload.school_email,
        payload.full_name,
        payload.school_name,
        payload.student_bio,
        payload.motivation_text
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create verification request"})),
        )
    })?;

    // Update user verification status
    let _ = sqlx::query!(
        r#"
        UPDATE users 
        SET verification_status = 'pending', verification_submitted_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
        user_id
    )
    .execute(&state.pool)
    .await;

    // Create student profile
    let _ = sqlx::query!(
        r#"
        INSERT INTO student_profiles (user_id, full_name, school_name, school_email, student_bio, motivation_text)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id) DO UPDATE SET
            full_name = $2,
            school_name = $3,
            school_email = $4,
            student_bio = $5,
            motivation_text = $6,
            updated_at = CURRENT_TIMESTAMP
        "#,
        user_id,
        payload.full_name,
        payload.school_name,
        payload.school_email,
        payload.student_bio,
        payload.motivation_text
    )
    .execute(&state.pool)
    .await;

    // Log activity
    let _ = sqlx::query!(
        r#"
        INSERT INTO activity_logs (user_id, action, target_id, target_type, metadata)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        user_id,
        "verification_applied",
        verification.id,
        "student_verification",
        serde_json::json!({
            "school_email": payload.school_email,
            "full_name": payload.full_name,
            "school_name": payload.school_name
        })
    )
    .execute(&state.pool)
    .await;

    // Add to verification history
    let _ = sqlx::query!(
        r#"
        INSERT INTO verification_history (user_id, verification_id, status, admin_message)
        VALUES ($1, $2, 'pending', NULL)
        "#,
        user_id,
        verification.id
    )
    .execute(&state.pool)
    .await;

    Ok((StatusCode::CREATED, Json(verification)))
}

/// Get enhanced verification status for a user
#[utoipa::path(
    get,
    path = "/api/students/verification-status/{user_id}",
    responses(
        (status = 200, description = "Verification status retrieved successfully", body = VerificationStatusResponse),
        (status = 404, description = "Verification not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Students"
)]
pub async fn get_verification_status(
    State(state): State<crate::state::AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<VerificationStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx::query!(
        r#"
        SELECT verification_status, verification_submitted_at, verification_approved_at
        FROM users
        WHERE id = $1
        "#,
        user_id
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
            Json(serde_json::json!({"error": "User not found"})),
        )
    })?;

    let verification = sqlx::query!(
        r#"
        SELECT admin_message, status
        FROM student_verifications
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database error"})),
        )
    })?;

    let status = user.verification_status.unwrap_or_else(|| "not_applied".to_string());
    let message = match status.as_str() {
        "pending" => Some("Your verification is under review. This usually takes 1-2 business days.".to_string()),
        "verified" => Some("Congratulations! Your student verification has been approved.".to_string()),
        "rejected" => verification.as_ref().and_then(|v| v.admin_message.clone()),
        _ => None,
    };

    Ok(Json(VerificationStatusResponse {
        status,
        message,
        submitted_at: user.verification_submitted_at,
        approved_at: user.verification_approved_at,
        admin_message: verification.and_then(|v| v.admin_message),
    }))
}

/// Get student profile information
#[utoipa::path(
    get,
    path = "/api/students/profile/{user_id}",
    responses(
        (status = 200, description = "Student profile retrieved successfully", body = StudentProfile),
        (status = 404, description = "Profile not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Students"
)]
pub async fn get_student_profile(
    State(state): State<crate::state::AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<StudentProfile>, (StatusCode, Json<serde_json::Value>)> {
    let profile = sqlx::query_as!(
        StudentProfile,
        r#"
        SELECT id, user_id, full_name, school_name, school_email, student_bio, motivation_text,
               profile_picture_url, linkedin_url, github_url, portfolio_url,
               created_at as "created_at!: chrono::DateTime<chrono::Utc>",
               updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
        FROM student_profiles
        WHERE user_id = $1
        "#,
        user_id
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
            Json(serde_json::json!({"error": "Student profile not found"})),
        )
    })?;

    Ok(Json(profile))
}

/// Update student profile
#[utoipa::path(
    put,
    path = "/api/students/profile/{user_id}",
    request_body = StudentProfile,
    responses(
        (status = 200, description = "Profile updated successfully", body = StudentProfile),
        (status = 404, description = "Profile not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Students"
)]
pub async fn update_student_profile(
    State(state): State<crate::state::AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<StudentProfile>,
) -> Result<Json<StudentProfile>, (StatusCode, Json<serde_json::Value>)> {
    let profile = sqlx::query_as!(
        StudentProfile,
        r#"
        UPDATE student_profiles 
        SET full_name = $2, school_name = $3, school_email = $4, student_bio = $5, 
            motivation_text = $6, profile_picture_url = $7, linkedin_url = $8, 
            github_url = $9, portfolio_url = $10, updated_at = CURRENT_TIMESTAMP
        WHERE user_id = $1
        RETURNING id, user_id, full_name, school_name, school_email, student_bio, motivation_text,
                  profile_picture_url, linkedin_url, github_url, portfolio_url,
                  created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                  updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
        "#,
        user_id,
        payload.full_name,
        payload.school_name,
        payload.school_email,
        payload.student_bio,
        payload.motivation_text,
        payload.profile_picture_url,
        payload.linkedin_url,
        payload.github_url,
        payload.portfolio_url
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
            Json(serde_json::json!({"error": "Student profile not found"})),
        )
    })?;

    Ok(Json(profile))
}
