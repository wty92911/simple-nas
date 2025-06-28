use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

// Placeholder login handler - will be properly implemented in Task 1.3
pub async fn login_placeholder() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "message": "Login endpoint - implementation coming in Task 1.3 (Security Infrastructure)",
        "status": "placeholder"
    })))
}
