use axum::{
    extract::Request,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{error, info, Level};
use tracing_subscriber;

mod config;
mod database;
mod handlers;
mod middleware;
mod services;
mod utils;

use config::Settings;
use database::{create_connection_pool, health_check, run_migrations};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Simple Home NAS server...");

    // Load configuration
    let settings = Settings::default();
    info!(
        "Loaded configuration with database URL: {}",
        settings.database.url.replace(
            &settings.database.url.split('@').nth(1).unwrap_or(""),
            "@[REDACTED]"
        )
    );

    // Create database connection pool
    info!("Connecting to PostgreSQL database...");
    let db_pool = create_connection_pool(&settings.database.url)
        .await
        .map_err(|e| {
            error!("Failed to create database connection pool: {}", e);
            e
        })?;

    // Run database migrations
    info!("Running database migrations...");
    run_migrations(&db_pool).await.map_err(|e| {
        error!("Database migration failed: {}", e);
        e
    })?;

    // Verify database connection
    info!("Verifying database connection...");
    health_check(&db_pool).await.map_err(|e| {
        error!("Database health check failed: {}", e);
        e
    })?;

    info!("Database connection established and migrations completed successfully");

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check_handler))
        .route("/health/db", get(database_health_handler))
        .route(
            "/api/v1/auth/login",
            post(handlers::auth::login_placeholder),
        )
        .with_state(db_pool)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::default().include_headers(true)),
                )
                .layer(CorsLayer::permissive()),
        );

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], settings.server.port));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> Json<Value> {
    Json(json!({
        "message": "Simple Home NAS API",
        "version": "0.1.0",
        "status": "running",
        "database": "PostgreSQL"
    }))
}

async fn health_check_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.1.0",
        "database": "PostgreSQL"
    }))
}

async fn database_health_handler(
    axum::extract::State(pool): axum::extract::State<sqlx::PgPool>,
) -> Result<Json<Value>, StatusCode> {
    match health_check(&pool).await {
        Ok(_) => Ok(Json(json!({
            "database_status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "database_type": "PostgreSQL"
        }))),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}
