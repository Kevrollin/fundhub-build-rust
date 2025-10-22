use axum::{extract::{State, Path}, Json, http::StatusCode};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::workers::distribute_campaign_funds;

#[derive(Serialize)]
pub struct ApiMessage { pub message: String }

#[derive(Deserialize)]
pub struct CreateCampaignRequest { 
    pub name: String, 
    pub criteria: String, 
    pub reward_pool_xlm: f64 
}

#[derive(Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub criteria: Option<String>,
    pub reward_pool_xlm: Option<f64>,
    pub status: Option<String>,
}

#[derive(Serialize)]
pub struct CampaignResponse {
    pub id: Uuid,
    pub name: String,
    pub criteria: String,
    pub reward_pool_xlm: f64,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct CampaignStats {
    pub total_campaigns: i64,
    pub active_campaigns: i64,
    pub total_reward_pool: f64,
    pub distributed_amount: f64,
}

pub async fn create(State(state): State<crate::state::AppState>, Json(req): Json<CreateCampaignRequest>) -> Json<ApiMessage> {
    let _ = sqlx::query!(
        r#"INSERT INTO campaigns (id, name, criteria, reward_pool_xlm, status, created_at)
           VALUES ($1, $2, $3, $4, 'active', NOW())"#,
        Uuid::new_v4(), req.name, req.criteria, req.reward_pool_xlm
    ).execute(&state.pool).await;
    Json(ApiMessage { message: "campaign created".into() })
}
pub async fn execute(State(state): State<crate::state::AppState>) -> Json<ApiMessage> {
    let _ = distribute_campaign_funds(&state.pool, &state.stellar).await;
    Json(ApiMessage { message: "campaign distribution triggered".into() })
}
pub async fn list(State(state): State<crate::state::AppState>) -> Json<serde_json::Value> {
    let rows = sqlx::query!(
        r#"SELECT id, name, criteria, reward_pool_xlm, status, created_at FROM campaigns WHERE status = 'active' ORDER BY created_at DESC"#
    ).fetch_all(&state.pool).await.unwrap_or_default();
    let json: Vec<_> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id,
        "name": r.name,
        "criteria": r.criteria,
        "reward_pool_xlm": r.reward_pool_xlm,
        "status": r.status,
        "created_at": r.created_at,
    })).collect();
    Json(serde_json::json!(json))
}

pub async fn get_by_id(State(state): State<crate::state::AppState>, Path(id): Path<Uuid>) -> Result<Json<CampaignResponse>, StatusCode> {
    let row = sqlx::query!(
        r#"SELECT id, name, criteria, reward_pool_xlm, status, created_at, updated_at FROM campaigns WHERE id = $1"#,
        id
    ).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match row {
        Some(r) => Ok(Json(CampaignResponse {
            id: r.id,
            name: r.name,
            criteria: r.criteria,
            reward_pool_xlm: r.reward_pool_xlm,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update(State(state): State<crate::state::AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdateCampaignRequest>) -> Result<Json<ApiMessage>, StatusCode> {
    let mut query = String::from("UPDATE campaigns SET ");
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 1;
    let mut updates = Vec::new();

    if let Some(name) = &req.name {
        updates.push(format!("name = ${}", param_count));
        params.push(Box::new(name.clone()));
        param_count += 1;
    }
    if let Some(criteria) = &req.criteria {
        updates.push(format!("criteria = ${}", param_count));
        params.push(Box::new(criteria.clone()));
        param_count += 1;
    }
    if let Some(reward_pool) = req.reward_pool_xlm {
        updates.push(format!("reward_pool_xlm = ${}", param_count));
        params.push(Box::new(reward_pool));
        param_count += 1;
    }
    if let Some(status) = &req.status {
        updates.push(format!("status = ${}", param_count));
        params.push(Box::new(status.clone()));
        param_count += 1;
    }

    if updates.is_empty() {
        return Ok(Json(ApiMessage { message: "No updates provided".into() }));
    }

    updates.push(format!("updated_at = NOW()"));
    query.push_str(&updates.join(", "));
    query.push_str(&format!(" WHERE id = ${}", param_count));
    params.push(Box::new(id));

    // For simplicity, using a direct query approach
    let result = if let Some(name) = req.name {
        if let Some(criteria) = req.criteria {
            if let Some(reward_pool) = req.reward_pool_xlm {
                if let Some(status) = req.status {
                    sqlx::query!(
                        r#"UPDATE campaigns SET name = $1, criteria = $2, reward_pool_xlm = $3, status = $4, updated_at = NOW() WHERE id = $5"#,
                        name, criteria, reward_pool, status, id
                    ).execute(&state.pool).await
                } else {
                    sqlx::query!(
                        r#"UPDATE campaigns SET name = $1, criteria = $2, reward_pool_xlm = $3, updated_at = NOW() WHERE id = $4"#,
                        name, criteria, reward_pool, id
                    ).execute(&state.pool).await
                }
            } else {
                sqlx::query!(
                    r#"UPDATE campaigns SET name = $1, criteria = $2, updated_at = NOW() WHERE id = $3"#,
                    name, criteria, id
                ).execute(&state.pool).await
            }
        } else {
            sqlx::query!(
                r#"UPDATE campaigns SET name = $1, updated_at = NOW() WHERE id = $2"#,
                name, id
            ).execute(&state.pool).await
        }
    } else {
        return Ok(Json(ApiMessage { message: "No valid updates provided".into() }));
    };

    match result {
        Ok(_) => Ok(Json(ApiMessage { message: "Campaign updated successfully".into() })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete(State(state): State<crate::state::AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiMessage>, StatusCode> {
    let result = sqlx::query!(
        r#"UPDATE campaigns SET status = 'deleted', updated_at = NOW() WHERE id = $1"#,
        id
    ).execute(&state.pool).await;

    match result {
        Ok(rows) => {
            if rows.rows_affected() > 0 {
                Ok(Json(ApiMessage { message: "Campaign deleted successfully".into() }))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn pause(State(state): State<crate::state::AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiMessage>, StatusCode> {
    let result = sqlx::query!(
        r#"UPDATE campaigns SET status = 'paused', updated_at = NOW() WHERE id = $1 AND status = 'active'"#,
        id
    ).execute(&state.pool).await;

    match result {
        Ok(rows) => {
            if rows.rows_affected() > 0 {
                Ok(Json(ApiMessage { message: "Campaign paused successfully".into() }))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn resume(State(state): State<crate::state::AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiMessage>, StatusCode> {
    let result = sqlx::query!(
        r#"UPDATE campaigns SET status = 'active', updated_at = NOW() WHERE id = $1 AND status = 'paused'"#,
        id
    ).execute(&state.pool).await;

    match result {
        Ok(rows) => {
            if rows.rows_affected() > 0 {
                Ok(Json(ApiMessage { message: "Campaign resumed successfully".into() }))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn stats(State(state): State<crate::state::AppState>) -> Result<Json<CampaignStats>, StatusCode> {
    let stats = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_campaigns,
            COUNT(CASE WHEN status = 'active' THEN 1 END) as active_campaigns,
            COALESCE(SUM(reward_pool_xlm), 0) as total_reward_pool,
            COALESCE(SUM(CASE WHEN status = 'completed' THEN reward_pool_xlm ELSE 0 END), 0) as distributed_amount
        FROM campaigns 
        WHERE status != 'deleted'
        "#
    ).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CampaignStats {
        total_campaigns: stats.total_campaigns.unwrap_or(0),
        active_campaigns: stats.active_campaigns.unwrap_or(0),
        total_reward_pool: stats.total_reward_pool.unwrap_or(0.0),
        distributed_amount: stats.distributed_amount.unwrap_or(0.0),
    }))
}


