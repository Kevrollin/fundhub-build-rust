use axum::{http::{StatusCode, Request, Response}, middleware::Next, extract::FromRequestParts};
use crate::utils::jwt;

fn bearer_from_auth(header: Option<&str>) -> Option<&str> {
    header.and_then(|h| h.strip_prefix("Bearer "))
}

pub async fn require_admin_mw(
    mut req: Request<axum::body::Body>, 
    next: Next
) -> Result<Response<axum::body::Body>, StatusCode> {
    let auth = req.headers().get("authorization").and_then(|v| v.to_str().ok());
    let token = bearer_from_auth(auth).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = jwt::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // For now, just check if the user ID is the admin user ID
    // This is a temporary fix to test the endpoints
    if claims.sub.to_string() != "00000000-0000-0000-0000-000000000001" {
        tracing::error!("User {} is not admin", claims.sub);
        return Err(StatusCode::FORBIDDEN); 
    }
    
    Ok(next.run(req).await)
}

pub async fn require_verified_student_mw(mut req: Request<axum::body::Body>, next: Next) -> Result<Response<axum::body::Body>, StatusCode> {
    let auth = req.headers().get("authorization").and_then(|v| v.to_str().ok());
    let token = bearer_from_auth(auth).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = jwt::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| {
            tracing::error!("DATABASE_URL not set");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Create a temporary connection for this check
    let pool = sqlx::PgPool::connect(&database_url).await
        .map_err(|e| {
            tracing::error!("Failed to connect to database: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let rec = sqlx::query!("SELECT verification_status FROM students WHERE user_id = $1", claims.sub)
        .fetch_optional(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    if rec.verification_status.to_lowercase() != "verified" { return Err(StatusCode::FORBIDDEN); }
    
    // Close the temporary connection
    pool.close().await;
    
    Ok(next.run(req).await)
}

pub async fn require_auth_mw(
    mut req: Request<axum::body::Body>, 
    next: Next
) -> Result<Response<axum::body::Body>, StatusCode> {
    let auth = req.headers().get("authorization").and_then(|v| v.to_str().ok());
    let token = bearer_from_auth(auth).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = jwt::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Store the claims in the request extensions for the handler to use
    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
}


