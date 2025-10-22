use axum::{extract::{Json, State, Path, Query}, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::types::BigDecimal;
use chrono::{DateTime, Utc};

use crate::models::{Project, ProjectMilestone, PublicProjectInfo};

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub student_id: Uuid,
    pub title: String,
    pub description: String,
    pub repo_url: Option<String>,
    pub media_urls: Option<Vec<String>>,
    pub tags: Vec<String>,
    pub funding_goal_xlm: String,
    pub milestones: Vec<CreateMilestoneRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMilestoneRequest {
    pub title: String,
    pub description: Option<String>,
    pub amount_xlm: String,
    pub proof_type: String,
    pub order: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub repo_url: Option<String>,
    pub media_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub funding_goal_xlm: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub project: Project,
    pub milestones: Vec<ProjectMilestone>,
}

#[derive(Debug, Deserialize)]
pub struct ListProjectsQuery {
    pub status: Option<String>,
    pub student_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ProjectListItem {
    pub id: Uuid,
    pub student_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub funding_goal: BigDecimal,
    pub status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct PublishProjectRequest {
    pub admin_id: Uuid,
    pub contract_address: Option<String>,
}

pub async fn create_project(
    State(state): State<crate::state::AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), StatusCode> {
    // Verify student exists and is verified
    let student = sqlx::query!(
        r#"
        SELECT id, verification_status 
        FROM students 
        WHERE id = $1
        "#,
        req.student_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if student.verification_status != "verified" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Parse funding goal
    let funding_goal: BigDecimal = req.funding_goal_xlm
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Create project
    let project_id = Uuid::new_v4();
    let project = sqlx::query_as!(
        Project,
        r#"
        INSERT INTO projects (
            id, student_id, title, description, repo_url, 
            media_url, tags, funding_goal, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending_review')
        RETURNING id, student_id, title, description, repo_url, 
                  media_url, tags, funding_goal, status, 
                  contract_address, created_at
        "#,
        project_id,
        req.student_id,
        req.title,
        req.description,
        req.repo_url,
        req.media_urls.as_ref().and_then(|urls| urls.first()).cloned(),
        Some(&req.tags[..]),
        funding_goal,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create project: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create milestones
    let mut milestones = Vec::new();
    for milestone_req in req.milestones {
        let amount_xlm: f64 = milestone_req.amount_xlm
            .parse()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let amount_stroops = (amount_xlm * 10_000_000.0) as i64;

        let milestone_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO project_milestones (
                id, project_id, title, description, amount_stroops, 
                proof_type, position, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')
            "#,
            milestone_id,
            project_id,
            milestone_req.title,
            milestone_req.description,
            amount_stroops,
            milestone_req.proof_type,
            milestone_req.order,
        )
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let milestone = ProjectMilestone {
            id: milestone_id,
            project_id,
            title: milestone_req.title.clone(),
            description: milestone_req.description.clone(),
            amount_stroops,
            proof_type: Some(milestone_req.proof_type.clone()),
            position: Some(milestone_req.order),
            status: Some("pending".to_string()),
            proof_url: None,
            completed_at: None,
            created_at: Some(Utc::now()),
        };

        milestones.push(milestone);
    }

    Ok((StatusCode::CREATED, Json(ProjectResponse {
        project,
        milestones,
    })))
}

pub async fn list_projects(
    State(state): State<crate::state::AppState>,
    Query(query): Query<ListProjectsQuery>,
) -> Result<Json<Vec<ProjectListItem>>, StatusCode> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let projects = if let Some(status) = query.status {
        sqlx::query_as!(
            ProjectListItem,
            r#"
            SELECT id, student_id, title, description, tags, 
                   funding_goal, status, created_at
            FROM projects
            WHERE status = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            status,
            limit,
            offset
        )
        .fetch_all(&state.pool)
        .await
    } else if let Some(student_id) = query.student_id {
        sqlx::query_as!(
            ProjectListItem,
            r#"
            SELECT id, student_id, title, description, tags, 
                   funding_goal, status, created_at
            FROM projects
            WHERE student_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            student_id,
            limit,
            offset
        )
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query_as!(
            ProjectListItem,
            r#"
            SELECT id, student_id, title, description, tags, 
                   funding_goal, status, created_at
            FROM projects
            WHERE status IN ('active', 'pending_review')
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&state.pool)
        .await
    };

    projects.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_project(
    State(state): State<crate::state::AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, StatusCode> {
    let project = sqlx::query_as!(
        Project,
        r#"
        SELECT id, student_id, title, description, repo_url, 
               media_url, tags, funding_goal, status, 
               contract_address, created_at
        FROM projects
        WHERE id = $1
        "#,
        project_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let milestones = sqlx::query_as!(
        ProjectMilestone,
        r#"
        SELECT id, project_id, title, description, amount_stroops, 
               proof_type, position, status, proof_url, 
               completed_at, created_at
        FROM project_milestones
        WHERE project_id = $1
        ORDER BY position ASC
        "#,
        project_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ProjectResponse {
        project,
        milestones,
    }))
}

pub async fn update_project(
    State(state): State<crate::state::AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    // Get existing project
    let mut project = sqlx::query_as!(
        Project,
        r#"
        SELECT id, student_id, title, description, repo_url, 
               media_url, tags, funding_goal, status, 
               contract_address, created_at
        FROM projects
        WHERE id = $1
        "#,
        project_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Can only update if pending_review or active (not completed/rejected)
    if project.status == "completed" || project.status == "rejected" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Update fields
    if let Some(title) = req.title {
        project.title = title;
    }
    if let Some(description) = req.description {
        project.description = Some(description);
    }
    if let Some(repo_url) = req.repo_url {
        project.repo_url = Some(repo_url);
    }
    if let Some(media_url) = req.media_url {
        project.media_url = Some(media_url);
    }
    if let Some(tags) = req.tags {
        project.tags = tags;
    }
    if let Some(funding_goal_str) = req.funding_goal_xlm {
        project.funding_goal = funding_goal_str
            .parse()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
    }

    // Save updates
    let updated_project = sqlx::query_as!(
        Project,
        r#"
        UPDATE projects
        SET title = $2, description = $3, repo_url = $4, 
            media_url = $5, tags = $6, funding_goal = $7
        WHERE id = $1
        RETURNING id, student_id, title, description, repo_url, 
                  media_url, tags, funding_goal, status, 
                  contract_address, created_at
        "#,
        project_id,
        project.title,
        project.description.as_deref(),
        project.repo_url,
        project.media_url,
        &project.tags[..],
        project.funding_goal,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated_project))
}

pub async fn delete_project(
    State(state): State<crate::state::AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Check project exists and is deletable (only pending_review)
    let project = sqlx::query!(
        r#"SELECT status FROM projects WHERE id = $1"#,
        project_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if project.status != "pending_review" {
        return Err(StatusCode::FORBIDDEN);
    }

    sqlx::query!(
        r#"DELETE FROM projects WHERE id = $1"#,
        project_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn publish_project(
    State(state): State<crate::state::AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<PublishProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    // Verify admin exists
    let admin = sqlx::query!(
        r#"SELECT role FROM users WHERE id = $1"#,
        req.admin_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::FORBIDDEN)?;

    // Admin role check
    if admin.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Update project status to active
    let project = sqlx::query_as!(
        Project,
        r#"
        UPDATE projects
        SET status = 'active', contract_address = $2
        WHERE id = $1
        RETURNING id, student_id, title, description, repo_url, 
                  media_url, tags, funding_goal, status, 
                  contract_address, created_at
        "#,
        project_id,
        req.contract_address,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Emit SSE notification
    let _ = state.notifier.send(format!(
        "project_published:{}:{}",
        project.student_id,
        project.id
    ));

    Ok(Json(project))
}

pub async fn reject_project(
    State(state): State<crate::state::AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Project>, StatusCode> {
    let project = sqlx::query_as!(
        Project,
        r#"
        UPDATE projects
        SET status = 'rejected'
        WHERE id = $1
        RETURNING id, student_id, title, description, repo_url, 
                  media_url, tags, funding_goal, status, 
                  contract_address, created_at
        "#,
        project_id,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Emit SSE notification
    let _ = state.notifier.send(format!(
        "project_rejected:{}:{}",
        project.student_id,
        project.id
    ));

    Ok(Json(project))
}

/// Get public project information (limited for guests and non-authenticated users)
#[utoipa::path(
    get,
    path = "/api/projects/public",
    responses(
        (status = 200, description = "Public projects retrieved successfully", body = Vec<PublicProjectInfo>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Projects"
)]
pub async fn get_public_projects(
    State(state): State<crate::state::AppState>,
) -> Result<Json<Vec<PublicProjectInfo>>, (StatusCode, Json<serde_json::Value>)> {
    let projects = sqlx::query_as!(
        PublicProjectInfo,
        r#"
        SELECT 
            p.id,
            p.title,
            LEFT(p.description, 200) as short_description,
            p.media_url,
            p.funding_goal as "funding_goal!: sqlx::types::BigDecimal",
            COALESCE(SUM(d.amount), 0) as "current_funding!: sqlx::types::BigDecimal",
            p.tags,
            p.created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM projects p
        LEFT JOIN donations d ON p.id = d.project_id AND d.status = 'confirmed'
        WHERE p.visibility = 'public' AND p.status = 'active'
        GROUP BY p.id, p.title, p.description, p.media_url, p.funding_goal, p.tags, p.created_at
        ORDER BY p.created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch public projects"})),
        )
    })?;

    Ok(Json(projects))
}

