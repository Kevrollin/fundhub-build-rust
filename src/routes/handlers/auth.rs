use axum::{
    extract::{Json, State, Path, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use argon2::password_hash::{PasswordVerifier, PasswordHash};
use chrono::{Duration, Utc};
use rand::Rng;

use crate::models::{User, UserRole, UserStatus, BaseRole};

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SignupResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: chrono::DateTime<Utc>,
}

pub async fn signup(
    State(state): State<crate::state::AppState>,
    Json(payload): Json<SignupRequest>,
) -> Result<(StatusCode, Json<SignupResponse>), StatusCode> {
    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    // Create user with active status (skip email verification for now)
    let user = sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash, role, base_role, is_verified, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, username, email
        "#,
        payload.username,
        payload.email,
        password_hash,
        "user",
        "base_user",
        true,
        "active",
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Skip email verification for now
    tracing::info!("User created successfully: {}", user.email);

    Ok((StatusCode::CREATED, Json(SignupResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        status: "active".to_string(),
    })))
}

pub async fn login(
    State(state): State<crate::state::AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    tracing::info!("Login attempt for email: {}", payload.email);
    
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash,
               role as "role: UserRole",
               base_role as "base_role!: BaseRole",
               is_verified as "is_verified!: bool",
               status as "status: UserStatus",
               last_login,
               created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM users WHERE email = $1
        "#,
        payload.email
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error during login: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        tracing::info!("User not found for email: {}", payload.email);
        StatusCode::UNAUTHORIZED
    })?;

    // Verify password
    let argon2 = Argon2::default();
    let parsed = PasswordHash::new(&user.password_hash)
        .map_err(|e| {
            tracing::error!("Password hash parsing error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let is_valid = argon2.verify_password(payload.password.as_bytes(), &parsed).is_ok();

    if !is_valid {
        tracing::info!("Invalid password for user: {}", payload.email);
        return Err(StatusCode::UNAUTHORIZED);
    }

    tracing::info!("Password verified for user: {}", user.id);

    // Generate JWT access token
    let access_token = crate::utils::jwt::create_token(&user.id)
        .map_err(|e| {
            tracing::error!("JWT token creation error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Generate refresh token
    let refresh_token = generate_random_token();
    let refresh_token_hash = hash_token(&refresh_token);
    let refresh_expires_at = Utc::now() + Duration::days(30);

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user.id,
        refresh_token_hash,
        refresh_expires_at,
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Refresh token insertion error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        expires_in: 3600, // 1 hour
    }))
}

pub async fn logout() -> StatusCode {
    // Since we're using JWTs, logout is handled client-side
    StatusCode::OK
}

pub async fn get_me(
    State(state): State<crate::state::AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ProfileResponse>, StatusCode> {
    // Extract user ID from JWT token
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash,
               role as "role: UserRole",
               base_role as "base_role!: BaseRole",
               is_verified as "is_verified!: bool",
               status as "status: UserStatus",
               last_login,
               created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM users WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ProfileResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        status: user.status,
        created_at: user.created_at,
    }))
}

pub async fn get_profile(
    State(state): State<crate::state::AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ProfileResponse>, StatusCode> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash,
               role as "role: UserRole",
               base_role as "base_role!: BaseRole",
               is_verified as "is_verified!: bool",
               status as "status: UserStatus",
               last_login,
               created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM users WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ProfileResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        status: user.status,
        created_at: user.created_at,
    }))
}

pub async fn refresh(
    State(state): State<crate::state::AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let token_hash = hash_token(&payload.refresh_token);
    
    // Verify refresh token exists and is not expired
    let token_record = sqlx::query!(
        r#"
        SELECT user_id, expires_at
        FROM refresh_tokens
        WHERE token_hash = $1 AND expires_at > NOW()
        "#,
        token_hash
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    // Generate new access token
    let access_token = crate::utils::jwt::create_token(&token_record.user_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate new refresh token
    let new_refresh_token = generate_random_token();
    let new_refresh_token_hash = hash_token(&new_refresh_token);
    let refresh_expires_at = Utc::now() + Duration::days(30);

    // Delete old refresh token
    sqlx::query!(
        r#"
        DELETE FROM refresh_tokens WHERE token_hash = $1
        "#,
        token_hash
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert new refresh token
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        token_record.user_id,
        new_refresh_token_hash,
        refresh_expires_at,
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        expires_in: 3600,
    }))
}

pub async fn verify_email(
    State(state): State<crate::state::AppState>,
    Query(query): Query<VerifyEmailQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Find verification token
    let token_record = sqlx::query!(
        r#"
        SELECT user_id, expires_at, verified_at
        FROM email_verification_tokens
        WHERE token = $1
        "#,
        query.token
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Check if already verified
    if token_record.verified_at.is_some() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if expired
    if token_record.expires_at < Utc::now() {
        return Err(StatusCode::GONE);
    }

    // Mark token as verified
    sqlx::query!(
        r#"
        UPDATE email_verification_tokens
        SET verified_at = NOW()
        WHERE token = $1
        "#,
        query.token
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update user status to active
    sqlx::query!(
        r#"
        UPDATE users
        SET status = 'active'
        WHERE id = $1
        "#,
        token_record.user_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "message": "Email verified successfully",
        "user_id": token_record.user_id
    })))
}

// Helper functions
fn generate_random_token() -> String {
    let mut rng = rand::thread_rng();
    let token: String = (0..32)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect();
    token
}

fn hash_token(token: &str) -> String {
    use argon2::Argon2;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(token.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

#[derive(Debug, Serialize)]
pub struct StudentStatusResponse {
    pub has_student_account: bool,
    pub student_status: String,
}

pub async fn get_student_status(
    State(state): State<crate::state::AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<StudentStatusResponse>, StatusCode> {
    let user_id = crate::utils::jwt::extract_user_id_from_headers(&headers)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check if user has a student record
    let student_record = sqlx::query!(
        r#"
        SELECT verification_status
        FROM students
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match student_record {
        Some(student) => {
            let status = match student.verification_status.as_str() {
                "verified" => "verified",
                "pending" => "pending",
                _ => "pending"
            };
            Ok(Json(StudentStatusResponse {
                has_student_account: true,
                student_status: status.to_string(),
            }))
        }
        None => {
            Ok(Json(StudentStatusResponse {
                has_student_account: false,
                student_status: "none".to_string(),
            }))
        }
    }
}