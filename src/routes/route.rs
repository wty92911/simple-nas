use std::sync::Arc;

use anyhow::Result;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
};
use serde_json::{Value, json};

use crate::handlers::{
    AppState,
    auth::{get_profile, login_user, logout_user, register_user},
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        // Public routes
        .route("/", get(root))
        .route("/health", get(health_check_handler))
        .route("/health/db", get(database_health_handler))
        // API v1 routes
        .nest("/api/v1", create_api_v1_routes())
        // Add application state
        .with_state(app_state)
}

fn create_api_v1_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Authentication routes (public)
        .nest("/auth", create_auth_routes())
        // File management routes (protected) - placeholder for Task 1.5
        .nest("/files", create_file_routes())
        // Share management routes (protected) - placeholder for Task 1.5
        .nest("/shares", create_share_routes())
        // Admin routes (admin protected) - placeholder for future
        .nest("/admin", create_admin_routes())
}

fn create_auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/profile", get(get_profile))
        .route("/logout", post(logout_user))
}

fn create_file_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(placeholder_files_list))
        .route("/upload", post(placeholder_files_upload))
        .route("/:file_id", get(placeholder_files_get))
        .route("/:file_id", post(placeholder_files_update))
        .route("/:file_id", delete(placeholder_files_delete))
}

fn create_share_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(placeholder_shares_list))
        .route("/", post(placeholder_shares_create))
        .route("/:share_id", get(placeholder_shares_get))
        .route("/:share_id", delete(placeholder_shares_delete))
}

fn create_admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users", get(placeholder_admin_users))
        .route("/stats", get(placeholder_admin_stats))
}

// Basic handlers
async fn root() -> Json<Value> {
    Json(json!({
        "message": "Simple Home NAS API",
        "version": "0.1.0",
        "status": "running",
        "database": "PostgreSQL",
        "security": "JWT + Middleware",
        "endpoints": {
            "auth": "/api/v1/auth/*",
            "files": "/api/v1/files/*",
            "shares": "/api/v1/shares/*",
            "admin": "/api/v1/admin/*"
        }
    }))
}

async fn health_check_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.1.0",
        "database": "PostgreSQL",
        "security": "enabled"
    }))
}

async fn database_health_handler(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Value>, StatusCode> {
    match app_state.db_service.health_check().await {
        Ok(_) => Ok(Json(json!({
            "database_status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "database_type": "PostgreSQL",
            "security": "enabled"
        }))),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

// Placeholder handlers for future implementation (Task 1.5)
async fn placeholder_files_list() -> Json<Value> {
    Json(json!({
        "message": "File list endpoint - implementation coming in Task 1.5 (File Management)",
        "status": "placeholder"
    }))
}

async fn placeholder_files_upload() -> Json<Value> {
    Json(json!({
        "message": "File upload endpoint - implementation coming in Task 1.5 (File Management)",
        "status": "placeholder"
    }))
}

async fn placeholder_files_get() -> Json<Value> {
    Json(json!({
        "message": "File get endpoint - implementation coming in Task 1.5 (File Management)",
        "status": "placeholder"
    }))
}

async fn placeholder_files_update() -> Json<Value> {
    Json(json!({
        "message": "File update endpoint - implementation coming in Task 1.5 (File Management)",
        "status": "placeholder"
    }))
}

async fn placeholder_files_delete() -> Json<Value> {
    Json(json!({
        "message": "File delete endpoint - implementation coming in Task 1.5 (File Management)",
        "status": "placeholder"
    }))
}

async fn placeholder_shares_list() -> Json<Value> {
    Json(json!({
        "message": "Share list endpoint - implementation coming in Task 1.5 (File Management)",
        "status": "placeholder"
    }))
}

async fn placeholder_shares_create() -> Json<Value> {
    Json(json!({
        "message": "Share create endpoint - implementation coming in Task 1.5 (File Management)",
        "status": "placeholder"
    }))
}

async fn placeholder_shares_get() -> Json<Value> {
    Json(json!({
        "message": "Share get endpoint - implementation coming in Task 1.5 (File Management)",
        "status": "placeholder"
    }))
}

async fn placeholder_shares_delete() -> Json<Value> {
    Json(json!({
        "message": "Share delete endpoint - implementation coming in Task 1.5 (File Management)",
        "status": "placeholder"
    }))
}

async fn placeholder_admin_users() -> Json<Value> {
    Json(json!({
        "message": "Admin users endpoint - implementation coming in future tasks",
        "status": "placeholder"
    }))
}

async fn placeholder_admin_stats() -> Json<Value> {
    Json(json!({
        "message": "Admin stats endpoint - implementation coming in future tasks",
        "status": "placeholder"
    }))
}
