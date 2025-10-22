use axum::{extract::{Json, State, Path, Multipart}, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Student, StudentVerification, StudentVerificationRequest, VerificationStatus};

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

/// Apply for student verification
#[utoipa::path(
    post,
    path = "/api/students/apply-verification",
    request_body = StudentVerificationRequest,
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
    Json(payload): Json<StudentVerificationRequest>,
) -> Result<(StatusCode, Json<StudentVerification>), (StatusCode, Json<serde_json::Value>)> {
    // Check if verification already exists for this user
    let existing = sqlx::query_scalar!(
        "SELECT id FROM student_verifications WHERE user_id = (SELECT id FROM users WHERE email = $1)",
        payload.school_email
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
            Json(serde_json::json!({"error": "Verification already exists for this email"})),
        ));
    }

    // Get user ID from email (assuming the school email is the user's email)
    let user_id = sqlx::query_scalar!(
        "SELECT id FROM users WHERE email = $1",
        payload.school_email
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

    // Create verification request
    let verification = sqlx::query_as!(
        StudentVerification,
        r#"
        INSERT INTO student_verifications (user_id, school_email, status)
        VALUES ($1, $2, 'pending')
        RETURNING id, user_id, school_email, status as "status!: VerificationStatus", admin_message, approved_at, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        "#,
        user_id,
        payload.school_email
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create verification request"})),
        )
    })?;

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
            "school_email": payload.school_email
        })
    )
    .execute(&state.pool)
    .await;

    Ok((StatusCode::CREATED, Json(verification)))
}

/// Get verification status for a user
#[utoipa::path(
    get,
    path = "/api/students/verification-status/{user_id}",
    responses(
        (status = 200, description = "Verification status retrieved successfully", body = StudentVerification),
        (status = 404, description = "Verification not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Students"
)]
pub async fn get_verification_status(
    State(state): State<crate::state::AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<StudentVerification>, (StatusCode, Json<serde_json::Value>)> {
    let verification = sqlx::query_as!(
        StudentVerification,
        r#"
        SELECT id, user_id, school_email, status as "status!: VerificationStatus", admin_message, approved_at, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
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
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Verification not found"})),
        )
    })?;

    Ok(Json(verification))
}
