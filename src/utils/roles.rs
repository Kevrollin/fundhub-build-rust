use axum::{http::{StatusCode, Request, Response}, middleware::Next};
use crate::{state::AppState, utils::jwt};

fn bearer_from_auth(header: Option<&str>) -> Option<&str> {
    header.and_then(|h| h.strip_prefix("Bearer "))
}

pub async fn require_admin_mw(mut req: Request<axum::body::Body>, next: Next) -> Result<Response<axum::body::Body>, StatusCode> {
    let auth = req.headers().get("authorization").and_then(|v| v.to_str().ok());
    let token = bearer_from_auth(auth).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = jwt::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Extract state from request extensions
    let state = req.extensions().get::<AppState>().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let rec = sqlx::query!("SELECT role FROM users WHERE id = $1", claims.sub)
        .fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    if rec.role.to_lowercase() != "admin" { return Err(StatusCode::FORBIDDEN); }
    Ok(next.run(req).await)
}

pub async fn require_verified_student_mw(mut req: Request<axum::body::Body>, next: Next) -> Result<Response<axum::body::Body>, StatusCode> {
    let auth = req.headers().get("authorization").and_then(|v| v.to_str().ok());
    let token = bearer_from_auth(auth).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = jwt::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Extract state from request extensions
    let state = req.extensions().get::<AppState>().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let rec = sqlx::query!("SELECT verification_status FROM students WHERE user_id = $1", claims.sub)
        .fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    if rec.verification_status.to_lowercase() != "verified" { return Err(StatusCode::FORBIDDEN); }
    Ok(next.run(req).await)
}


