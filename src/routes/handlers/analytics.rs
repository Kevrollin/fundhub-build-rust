use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use sqlx::types::BigDecimal;
use num_traits::cast::ToPrimitive;

#[derive(Serialize)]
pub struct ApiMessage { 
    pub message: String 
}

#[derive(Deserialize)]
pub struct DateRangeQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct ProjectAnalytics {
    pub project_id: Uuid,
    pub title: String,
    pub total_donations: f64,
    pub donation_count: i64,
    pub funding_goal: f64,
    pub funding_percentage: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct StudentAnalytics {
    pub student_id: Uuid,
    pub username: String,
    pub total_donations_received: f64,
    pub project_count: i64,
    pub active_projects: i64,
    pub verification_status: String,
}

#[derive(Serialize)]
pub struct CampaignAnalytics {
    pub campaign_id: Uuid,
    pub name: String,
    pub reward_pool_xlm: f64,
    pub distributed_amount: f64,
    pub recipient_count: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct DonationTrend {
    pub date: String,
    pub count: i64,
    pub total_amount: f64,
}

#[derive(Serialize)]
pub struct PlatformStats {
    pub total_users: i64,
    pub verified_students: i64,
    pub total_projects: i64,
    pub active_projects: i64,
    pub total_donations: f64,
    pub total_campaigns: i64,
    pub active_campaigns: i64,
    pub total_reward_pool: f64,
}

pub async fn top_projects(
    State(state): State<crate::state::AppState>, 
    Query(params): Query<DateRangeQuery>
) -> Result<Json<Vec<ProjectAnalytics>>, StatusCode> {
    let limit = params.limit.unwrap_or(10);
    let start_date = params.start_date.unwrap_or(Utc::now() - Duration::days(30));
    let end_date = params.end_date.unwrap_or(Utc::now());

    let rows = sqlx::query!(
        r#"
        SELECT 
            p.id as project_id,
            p.title,
            p.funding_goal,
            p.created_at,
            COALESCE(SUM(d.amount), 0) as total_donations,
            COUNT(d.id) as donation_count
        FROM projects p
        LEFT JOIN donations d ON p.id = d.project_id 
            AND d.status = 'confirmed'
            AND d.created_at >= $1 
            AND d.created_at <= $2
        GROUP BY p.id, p.title, p.funding_goal, p.created_at
        ORDER BY total_donations DESC
        LIMIT $3
        "#,
        start_date, end_date, limit
    ).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let analytics: Vec<ProjectAnalytics> = rows.into_iter().map(|r| {
        let total_donations = r.total_donations.unwrap_or(BigDecimal::from(0));
        let funding_goal = r.funding_goal.clone();
        
        let funding_percentage = if funding_goal > BigDecimal::from(0) {
            let percentage = (total_donations.clone() / funding_goal.clone()) * BigDecimal::from(100);
            percentage.to_f64().unwrap_or(0.0)
        } else {
            0.0
        };

        ProjectAnalytics {
            project_id: r.project_id,
            title: r.title,
            total_donations: total_donations.to_f64().unwrap_or(0.0),
            donation_count: r.donation_count.unwrap_or(0),
            funding_goal: funding_goal.to_f64().unwrap_or(0.0),
            funding_percentage,
            created_at: r.created_at,
        }
    }).collect();

    Ok(Json(analytics))
}

pub async fn top_students(
    State(state): State<crate::state::AppState>,
    Query(params): Query<DateRangeQuery>
) -> Result<Json<Vec<StudentAnalytics>>, StatusCode> {
    let limit = params.limit.unwrap_or(10);
    let start_date = params.start_date.unwrap_or(Utc::now() - Duration::days(30));
    let end_date = params.end_date.unwrap_or(Utc::now());

    let rows = sqlx::query!(
        r#"
        SELECT 
            s.id as student_id,
            u.username,
            s.verification_status,
            COALESCE(SUM(d.amount), 0) as total_donations_received,
            COUNT(DISTINCT p.id) as project_count,
            COUNT(DISTINCT CASE WHEN p.created_at >= NOW() - INTERVAL '30 days' THEN p.id END) as active_projects
        FROM students s
        JOIN users u ON s.user_id = u.id
        LEFT JOIN projects p ON s.id = p.student_id
        LEFT JOIN donations d ON p.id = d.project_id 
            AND d.status = 'confirmed'
            AND d.created_at >= $1 
            AND d.created_at <= $2
        GROUP BY s.id, u.username, s.verification_status
        ORDER BY total_donations_received DESC
        LIMIT $3
        "#,
        start_date, end_date, limit
    ).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let analytics: Vec<StudentAnalytics> = rows.into_iter().map(|r| {
        StudentAnalytics {
            student_id: r.student_id,
            username: r.username,
            total_donations_received: r.total_donations_received.unwrap_or(BigDecimal::from(0)).to_f64().unwrap_or(0.0),
            project_count: r.project_count.unwrap_or(0),
            active_projects: r.active_projects.unwrap_or(0),
            verification_status: r.verification_status,
        }
    }).collect();

    Ok(Json(analytics))
}

pub async fn campaign_performance(
    State(state): State<crate::state::AppState>,
    Query(params): Query<DateRangeQuery>
) -> Result<Json<Vec<CampaignAnalytics>>, StatusCode> {
    let limit = params.limit.unwrap_or(10);
    let start_date = params.start_date.unwrap_or(Utc::now() - Duration::days(30));
    let end_date = params.end_date.unwrap_or(Utc::now());

    let rows = sqlx::query!(
        r#"
        SELECT 
            c.id as campaign_id,
            c.name,
            c.reward_pool_xlm,
            c.status,
            c.created_at,
            COALESCE(SUM(cd.amount), 0) as distributed_amount,
            COUNT(DISTINCT cd.recipient_id) as recipient_count
        FROM campaigns c
        LEFT JOIN campaign_distributions cd ON c.id = cd.campaign_id
            AND cd.created_at >= $1 
            AND cd.created_at <= $2
        WHERE c.created_at >= $1 AND c.created_at <= $2
        GROUP BY c.id, c.name, c.reward_pool_xlm, c.status, c.created_at
        ORDER BY distributed_amount DESC
        LIMIT $3
        "#,
        start_date, end_date, limit
    ).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let analytics: Vec<CampaignAnalytics> = rows.into_iter().map(|r| {
        CampaignAnalytics {
            campaign_id: r.campaign_id,
            name: r.name,
            reward_pool_xlm: r.reward_pool_xlm,
            distributed_amount: r.distributed_amount.unwrap_or(0.0),
            recipient_count: r.recipient_count.unwrap_or(0),
            status: r.status,
            created_at: r.created_at,
        }
    }).collect();

    Ok(Json(analytics))
}

pub async fn donation_trends(
    State(state): State<crate::state::AppState>,
    Query(params): Query<DateRangeQuery>
) -> Result<Json<Vec<DonationTrend>>, StatusCode> {
    let start_date = params.start_date.unwrap_or(Utc::now() - Duration::days(30));
    let end_date = params.end_date.unwrap_or(Utc::now());

    let rows = sqlx::query!(
        r#"
        SELECT 
            DATE(created_at) as donation_date,
            COUNT(*) as count,
            SUM(amount) as total_amount
        FROM donations 
        WHERE status = 'confirmed'
            AND created_at >= $1 
            AND created_at <= $2
        GROUP BY DATE(created_at)
        ORDER BY donation_date ASC
        "#,
        start_date, end_date
    ).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let trends: Vec<DonationTrend> = rows.into_iter().map(|r| {
        DonationTrend {
            date: r.donation_date.unwrap_or(chrono::Utc::now().date_naive()).format("%Y-%m-%d").to_string(),
            count: r.count.unwrap_or(0),
            total_amount: r.total_amount.unwrap_or(BigDecimal::from(0)).to_f64().unwrap_or(0.0),
        }
    }).collect();

    Ok(Json(trends))
}

pub async fn platform_stats(
    State(state): State<crate::state::AppState>
) -> Result<Json<PlatformStats>, StatusCode> {
    let stats = sqlx::query!(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM users) as total_users,
            (SELECT COUNT(*) FROM students WHERE verification_status = 'verified') as verified_students,
            (SELECT COUNT(*) FROM projects) as total_projects,
            (SELECT COUNT(*) FROM projects WHERE created_at >= NOW() - INTERVAL '30 days') as active_projects,
            (SELECT COALESCE(SUM(amount), 0) FROM donations WHERE status = 'confirmed') as total_donations,
            (SELECT COUNT(*) FROM campaigns WHERE status != 'deleted') as total_campaigns,
            (SELECT COUNT(*) FROM campaigns WHERE status = 'active') as active_campaigns,
            (SELECT COALESCE(SUM(reward_pool_xlm), 0) FROM campaigns WHERE status = 'active') as total_reward_pool
        "#
    ).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PlatformStats {
        total_users: stats.total_users.unwrap_or(0),
        verified_students: stats.verified_students.unwrap_or(0),
        total_projects: stats.total_projects.unwrap_or(0),
        active_projects: stats.active_projects.unwrap_or(0),
        total_donations: stats.total_donations.unwrap_or(BigDecimal::from(0)).to_f64().unwrap_or(0.0),
        total_campaigns: stats.total_campaigns.unwrap_or(0),
        active_campaigns: stats.active_campaigns.unwrap_or(0),
        total_reward_pool: stats.total_reward_pool.unwrap_or(0.0),
    }))
}

pub async fn project_analytics(
    State(state): State<crate::state::AppState>,
    Path(project_id): Path<Uuid>
) -> Result<Json<ProjectAnalytics>, StatusCode> {
    let row = sqlx::query!(
        r#"
        SELECT 
            p.id as project_id,
            p.title,
            p.funding_goal,
            p.created_at,
            COALESCE(SUM(d.amount), 0) as total_donations,
            COUNT(d.id) as donation_count
        FROM projects p
        LEFT JOIN donations d ON p.id = d.project_id AND d.status = 'confirmed'
        WHERE p.id = $1
        GROUP BY p.id, p.title, p.funding_goal, p.created_at
        "#,
        project_id
    ).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(r) => {
            let total_donations = r.total_donations.unwrap_or(BigDecimal::from(0));
            let funding_goal = r.funding_goal.clone();
            
            let funding_percentage = if funding_goal > BigDecimal::from(0) {
                let percentage = (total_donations.clone() / funding_goal.clone()) * BigDecimal::from(100);
                percentage.to_f64().unwrap_or(0.0)
            } else {
                0.0
            };

            Ok(Json(ProjectAnalytics {
                project_id: r.project_id,
                title: r.title,
                total_donations: total_donations.to_f64().unwrap_or(0.0),
                donation_count: r.donation_count.unwrap_or(0),
                funding_goal: funding_goal.to_f64().unwrap_or(0.0),
                funding_percentage,
                created_at: r.created_at,
            }))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn student_analytics(
    State(state): State<crate::state::AppState>,
    Path(student_id): Path<Uuid>
) -> Result<Json<StudentAnalytics>, StatusCode> {
    let row = sqlx::query!(
        r#"
        SELECT 
            s.id as student_id,
            u.username,
            s.verification_status,
            COALESCE(SUM(d.amount), 0) as total_donations_received,
            COUNT(DISTINCT p.id) as project_count,
            COUNT(DISTINCT CASE WHEN p.created_at >= NOW() - INTERVAL '30 days' THEN p.id END) as active_projects
        FROM students s
        JOIN users u ON s.user_id = u.id
        LEFT JOIN projects p ON s.id = p.student_id
        LEFT JOIN donations d ON p.id = d.project_id AND d.status = 'confirmed'
        WHERE s.id = $1
        GROUP BY s.id, u.username, s.verification_status
        "#,
        student_id
    ).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(r) => {
            Ok(Json(StudentAnalytics {
                student_id: r.student_id,
                username: r.username,
                total_donations_received: r.total_donations_received.unwrap_or(BigDecimal::from(0)).to_f64().unwrap_or(0.0),
                project_count: r.project_count.unwrap_or(0),
                active_projects: r.active_projects.unwrap_or(0),
                verification_status: r.verification_status,
            }))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}
