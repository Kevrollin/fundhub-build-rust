use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tracing::info;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{CorsLayer, Any, AllowOrigin};

mod config;
mod models;
mod routes;
pub mod services;
mod utils;
mod state;
mod workers;
mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize CLI interface
    let cli = cli::FundHubCLI::new();
    cli.show_banner();

    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Show startup progress
    let startup_pb = cli.show_startup_progress();
    
    // Load config
    startup_pb.set_message("Loading configuration...");
    startup_pb.inc(10);
    let config = config::init()?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Set up database connection
    startup_pb.set_message("Connecting to database...");
    startup_pb.inc(20);
    cli.initialize_database().await?;
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    
    // Initialize Stellar service
    startup_pb.set_message("Initializing Stellar service...");
    startup_pb.inc(20);
    cli.initialize_stellar().await?;
    
    let stellar_service = services::stellar::StellarService::new(&config)?;
    let new_stellar_service = services::NewStellarService::new(
        &config.stellar_horizon_url,
        &config.platform_wallet_secret_key,
        &config.platform_wallet_public_key,
    )?;
    
    // Start background workers
    startup_pb.set_message("Starting background workers...");
    startup_pb.inc(20);
    cli.start_workers().await?;
    
    let worker = workers::Worker::new(pool.clone(), stellar_service.clone());
    worker.start().await?;
    
    // Start analytics worker
    let analytics_worker = workers::analytics::AnalyticsWorker::new(pool.clone());
    analytics_worker.start().await?;
    
    // Start payment reconciler worker
    let payment_reconciler = workers::payment_reconciler::PaymentReconciler::new(pool.clone());
    tokio::spawn(async move {
        if let Err(e) = payment_reconciler.start().await {
            eprintln!("Payment reconciler error: {}", e);
        }
    });
    
    // Build our application
    startup_pb.set_message("Building application...");
    startup_pb.inc(20);
    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(100);

    let app = Router::new()
        .route("/health", get(health_check))
        // Mount API routes
        .nest("/api/auth", routes::auth_routes())
        .nest("/api/students", routes::student_routes())
        .nest("/api/wallets", routes::wallet_routes())
        .nest("/api/projects", routes::project_routes())
        .nest("/api/donations", routes::donation_routes())
        .nest("/api/campaigns", routes::campaign_routes())
        .nest("/api/admin", routes::admin_routes())
        .nest("/api/analytics", routes::analytics_routes())
        .nest("/api/guest", routes::guest_routes())
        .nest("/api/milestones", routes::milestone_routes())
        .nest("/api/contracts", routes::contract_routes())
        .nest("/api/payments", routes::payment_routes())
        .nest("/api/notifications", routes::notification_routes())
        .route("/api/notifications/sse", get(routes::sse_notifications))
        // Documentation routes
        .nest("/api/docs", routes::docs_routes())
        // Add CORS middleware
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:8080".parse().unwrap(),
                    "https://localhost:8080".parse().unwrap(),
                    "http://localhost:3000".parse().unwrap(),
                    "https://localhost:3000".parse().unwrap(),
                    "http://127.0.0.1:8080".parse().unwrap(),
                    "https://127.0.0.1:8080".parse().unwrap(),
                    "http://127.0.0.1:3000".parse().unwrap(),
                    "https://127.0.0.1:3000".parse().unwrap(),
                    // Add your Vercel frontend domain here
                    "https://your-app-name.vercel.app".parse().unwrap(),
                ])
                .allow_methods([
                    "GET".parse().unwrap(),
                    "POST".parse().unwrap(),
                    "PUT".parse().unwrap(),
                    "DELETE".parse().unwrap(),
                    "PATCH".parse().unwrap(),
                    "OPTIONS".parse().unwrap(),
                ])
                .allow_headers([
                    "content-type".parse().unwrap(),
                    "authorization".parse().unwrap(),
                    "accept".parse().unwrap(),
                    "origin".parse().unwrap(),
                    "x-requested-with".parse().unwrap(),
                ])
                .allow_credentials(true)
        )
        // Add middleware
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // Add state
        .with_state(state::AppState { 
            pool, 
            stellar: stellar_service, 
            stellar_service: new_stellar_service,
            notifier: tx 
        });

    // Complete startup
    startup_pb.set_message("Starting HTTP server...");
    startup_pb.inc(10);
    startup_pb.finish_with_message("✅ FundHub Backend started successfully!");

    // Show server information
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    cli.show_server_info(port);

    // Set up graceful shutdown
    let start_time = std::time::Instant::now();
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    // Run the server - bind to 0.0.0.0 for production
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let server = axum::serve(tokio::net::TcpListener::bind(addr).await?, app.into_make_service());

    // Handle shutdown
    tokio::select! {
        _ = server => {
            cli.show_shutdown_message();
        }
        _ = shutdown_signal => {
            cli.show_shutdown_message();
        }
    }

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}