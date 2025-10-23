use axum::{
    routing::{get, post},
    Router,
    extract::State,
    response::sse::{Sse, Event},
    response::IntoResponse,
    middleware,
};
use crate::state::AppState;
pub mod handlers; // expose handlers module in this module tree
pub mod payments; // expose payments module
use tokio_stream::wrappers::BroadcastStream;
use futures::StreamExt;
use crate::utils::roles::{require_admin_mw, require_verified_student_mw, require_auth_mw};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/signup", post(handlers::auth::signup))
        .route("/login", post(handlers::auth::login))
        .route("/logout", post(handlers::auth::logout))
        .route("/refresh", post(handlers::auth::refresh))
        .route("/verify-email", get(handlers::auth::verify_email))
        .route("/me", get(handlers::auth::get_me))
        .route("/profile/:user_id", get(handlers::auth::get_profile))
        .route("/student-status", get(handlers::auth::get_student_status))
}

pub fn student_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(self::handlers::students::register))
        .route("/status/:user_id", get(self::handlers::students::get_status))
        .route("/update", post(self::handlers::students::update))
        .route("/apply-verification", post(self::handlers::students::apply_verification).layer(middleware::from_fn(require_auth_mw)))
        .route("/verification-status/:user_id", get(self::handlers::students::get_verification_status))
        .route("/profile/:user_id", get(self::handlers::students::get_student_profile))
        .route("/profile/:user_id", axum::routing::put(self::handlers::students::update_student_profile))
}

pub fn wallet_routes() -> Router<AppState> {
    Router::new()
        .route("/test", get(self::handlers::wallets::test_connection))
        .route("/connect", post(self::handlers::wallets::connect))
        .route("/user/:user_id", get(self::handlers::wallets::get_user_wallet))
        .route("/details/:wallet_id", get(self::handlers::wallets::get_wallet_details))
        .route("/balance/:wallet_id", get(self::handlers::wallets::get_balance))
        .route("/transactions/:wallet_id", get(self::handlers::wallets::get_transactions))
        // Removed student-only restriction - all authenticated users can connect wallets
}

pub fn project_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(self::handlers::projects::create_project))
        .route("/", get(self::handlers::projects::list_projects))
        .route("/public", get(self::handlers::projects::get_public_projects))
        .route("/:id", get(self::handlers::projects::get_project))
        .route("/:id", axum::routing::put(self::handlers::projects::update_project))
        .route("/:id", axum::routing::delete(self::handlers::projects::delete_project))
        .route("/:id/publish", post(self::handlers::projects::publish_project))
        .route("/:id/reject", post(self::handlers::projects::reject_project))
}

pub fn donation_routes() -> Router<AppState> {
    Router::new()
        .route("/initiate", post(self::handlers::donations::initiate))
        .route("/verify", post(self::handlers::donations::verify))
        .route("/platform/initiate", post(self::handlers::donations::initiate_platform_donation))
        .route("/project/:project_id", get(self::handlers::donations::get_project_donations))
        .route("/student/:student_id", get(self::handlers::donations::get_student_donations))
}

pub fn campaign_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(self::handlers::campaigns::list))
        .route("/create", post(self::handlers::campaigns::create))
        .route("/execute", post(self::handlers::campaigns::execute))
        .route("/active", get(self::handlers::campaigns::list))
        .route("/stats", get(self::handlers::campaigns::stats))
        .route("/:id", get(self::handlers::campaigns::get_by_id))
        .route("/:id", axum::routing::put(self::handlers::campaigns::update))
        .route("/:id", axum::routing::delete(self::handlers::campaigns::delete))
        .route("/:id/pause", post(self::handlers::campaigns::pause))
        .route("/:id/resume", post(self::handlers::campaigns::resume))
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/students", get(self::handlers::admin::list_students))
        .route("/verifications", get(self::handlers::admin::list_pending_verifications))
        .route("/verifications/all", get(self::handlers::admin::list_all_verifications))
        .route("/verifications/approved", get(self::handlers::admin::list_approved_verifications))
        .route("/verifications/rejected", get(self::handlers::admin::list_rejected_verifications))
        .route("/verifications/enhanced", get(self::handlers::admin::get_enhanced_verifications))
        .route("/verifications/:id/details", get(self::handlers::admin::get_verification_details))
        .route("/verifications/:id/approve", post(self::handlers::admin::approve_verification))
        .route("/verifications/:id/reject", post(self::handlers::admin::reject_verification))
        .route("/verifications/:id/approve-enhanced", post(self::handlers::admin::approve_verification_enhanced))
        .route("/verifications/:id/reject-enhanced", post(self::handlers::admin::reject_verification_enhanced))
        .route("/approve-student/:verification_id", post(self::handlers::admin::approve_student_verification))
        .route("/verify-student", post(self::handlers::admin::verify_student))
        .route("/fund-student", post(self::handlers::admin::fund_student))
        .route("/logs", get(self::handlers::admin::get_activity_logs))
        .route("/overview", get(self::handlers::admin::get_admin_overview))
        .route_layer(middleware::from_fn(require_admin_mw))
}

pub fn analytics_routes() -> Router<AppState> {
    Router::new()
        .route("/platform/stats", get(self::handlers::analytics::platform_stats))
        .route("/projects/top", get(self::handlers::analytics::top_projects))
        .route("/students/top", get(self::handlers::analytics::top_students))
        .route("/campaigns/performance", get(self::handlers::analytics::campaign_performance))
        .route("/donations/trends", get(self::handlers::analytics::donation_trends))
        .route("/projects/:id", get(self::handlers::analytics::project_analytics))
        .route("/students/:id", get(self::handlers::analytics::student_analytics))
}

pub fn guest_routes() -> Router<AppState> {
    Router::new()
        .route("/fund", post(self::handlers::guest::create_guest_donation))
        .route("/verify", post(self::handlers::guest::verify_guest_donation))
        .route("/projects", get(self::handlers::guest::get_public_projects))
}

pub fn milestone_routes() -> Router<AppState> {
    Router::new()
        .route("/projects/:project_id/milestones", post(self::handlers::milestones::create_milestone))
        .route("/projects/:project_id/milestones", get(self::handlers::milestones::get_project_milestones))
        .route("/projects/:project_id/milestones/:milestone_id/release", post(self::handlers::milestones::release_milestone))
        .route_layer(middleware::from_fn(require_verified_student_mw))
}

pub fn contract_routes() -> Router<AppState> {
    Router::new()
        .route("/deploy", post(self::handlers::contracts::deploy_contracts))
        .route("/milestones/register", post(self::handlers::contracts::register_milestone))
        .route("/milestones/release", post(self::handlers::contracts::release_milestone))
        .route("/deposits/record", post(self::handlers::contracts::record_deposit))
        .route("/projects/:project_id/balance", get(self::handlers::contracts::get_project_balance))
        .route("/projects/:project_id/milestones", get(self::handlers::contracts::get_project_milestones))
        .route("/addresses", get(self::handlers::contracts::get_contract_addresses))
        .route_layer(middleware::from_fn(require_admin_mw))
}

pub fn payment_routes() -> Router<AppState> {
    Router::new()
        .route("/initiate", post(self::handlers::payments::initiate_payment))
        .route("/mpesa/webhook", post(self::handlers::payments::mpesa_webhook))
        .route("/stripe/webhook", post(self::handlers::payments::stripe_webhook))
        .route("/refund", post(self::handlers::payments::process_refund))
        .route("/providers", get(self::handlers::payments::get_providers))
        .route("/status", get(self::handlers::payments::get_payment_status))
}

pub fn notification_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(self::handlers::notifications::get_notifications))
        .route("/unread-count", get(self::handlers::notifications::get_unread_count))
        .route("/read-all", axum::routing::put(self::handlers::notifications::mark_all_notifications_read))
        .route("/:id/read", axum::routing::put(self::handlers::notifications::mark_notification_read))
        .route("/:id", axum::routing::delete(self::handlers::notifications::delete_notification))
        .route("/create", post(self::handlers::notifications::create_notification))
}

pub fn docs_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::docs::docs_html))
        .route("/api", get(handlers::docs::api_info))
        .route("/health", get(handlers::docs::health_check))
}

pub async fn sse_notifications(State(state): State<AppState>) -> impl IntoResponse {
    let rx = state.notifier.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
        match msg {
            Ok(s) => Some(Ok::<Event, std::convert::Infallible>(Event::default().data(s))),
            Err(_) => None,
        }
    });
    Sse::new(stream)
}

// Analytics endpoints
pub async fn analytics_top_projects(State(state): State<AppState>) -> impl IntoResponse {
    let rows = sqlx::query!(
        r#"SELECT entity_id, value FROM analytics_summary WHERE entity_type = 'project' AND metric = 'total_donations' ORDER BY value DESC LIMIT 10"#
    ).fetch_all(&state.pool).await.unwrap_or_default();
    let json: Vec<_> = rows.into_iter().map(|r| serde_json::json!({"project_id": r.entity_id, "total_donations": r.value})).collect();
    axum::Json(serde_json::json!(json))
}

pub async fn analytics_top_students(State(state): State<AppState>) -> impl IntoResponse {
    let rows = sqlx::query!(
        r#"SELECT entity_id, value FROM analytics_summary WHERE entity_type = 'student' AND metric = 'total_donations' ORDER BY value DESC LIMIT 10"#
    ).fetch_all(&state.pool).await.unwrap_or_default();
    let json: Vec<_> = rows.into_iter().map(|r| serde_json::json!({"student_id": r.entity_id, "total_donations": r.value})).collect();
    axum::Json(serde_json::json!(json))
}