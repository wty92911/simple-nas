use std::sync::Arc;

use axum::{Extension, extract::State, http::StatusCode, response::Json};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::database::models::{
    CreateUserRequest, ErrorResponse, LoginRequest, LoginResponse, UserInfo,
};
use crate::handlers::AppState;
use crate::middleware::auth::AuthMiddleware;

// User registration endpoint
pub async fn register_user(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate required fields
    if request.username.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Validation Error".to_string(),
                message: "Username is required".to_string(),
                code: Some("400".to_string()),
            }),
        ));
    }

    if request.email.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Validation Error".to_string(),
                message: "Email is required".to_string(),
                code: Some("400".to_string()),
            }),
        ));
    }

    if request.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Validation Error".to_string(),
                message: "Password must be at least 8 characters long".to_string(),
                code: Some("400".to_string()),
            }),
        ));
    }

    // Create user
    match app_state.db_service.create_user(request).await {
        Ok(user) => {
            // Generate JWT token
            match app_state.jwt_service.generate_token(&user) {
                Ok((token, expires_at)) => {
                    // Create session in database
                    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

                    match app_state
                        .db_service
                        .create_session(user.id, token_hash, expires_at)
                        .await
                    {
                        Ok(_) => Ok(Json(LoginResponse {
                            token,
                            user,
                            expires_at,
                        })),
                        Err(_) => {
                            // If session creation fails, still return the token (stateless JWT)
                            Ok(Json(LoginResponse {
                                token,
                                user,
                                expires_at,
                            }))
                        }
                    }
                }
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Authentication Error".to_string(),
                        message: "Failed to generate authentication token".to_string(),
                        code: Some("500".to_string()),
                    }),
                )),
            }
        }
        Err(e) => Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Registration Error".to_string(),
                message: e.to_string(),
                code: Some("409".to_string()),
            }),
        )),
    }
}

// User login endpoint
pub async fn login_user(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate credentials
    match app_state
        .db_service
        .authenticate_user(&request.username, &request.password)
        .await
    {
        Ok(Some(user)) => {
            // Generate JWT token
            match app_state.jwt_service.generate_token(&user) {
                Ok((token, expires_at)) => {
                    // Create session in database
                    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

                    match app_state
                        .db_service
                        .create_session(user.id, token_hash, expires_at)
                        .await
                    {
                        Ok(_) => Ok(Json(LoginResponse {
                            token,
                            user,
                            expires_at,
                        })),
                        Err(_) => {
                            // If session creation fails, still return the token (stateless JWT)
                            Ok(Json(LoginResponse {
                                token,
                                user,
                                expires_at,
                            }))
                        }
                    }
                }
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Authentication Error".to_string(),
                        message: "Failed to generate authentication token".to_string(),
                        code: Some("500".to_string()),
                    }),
                )),
            }
        }
        Ok(None) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Authentication Error".to_string(),
                message: "Invalid username or password".to_string(),
                code: Some("401".to_string()),
            }),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Authentication Error".to_string(),
                message: "Failed to authenticate user".to_string(),
                code: Some("500".to_string()),
            }),
        )),
    }
}

// Get current user profile
pub async fn get_profile(Extension(auth): Extension<AuthMiddleware>) -> Json<UserInfo> {
    Json(auth.user)
}

// User logout handler
pub async fn logout_user(
    Extension(auth): Extension<AuthMiddleware>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // In a JWT-based system, logout is typically handled client-side by removing the token
    // However, we can log the logout action or potentially add token blacklisting in the future
    tracing::info!("User {} logged out", auth.user.username);

    Ok(Json(json!({
        "message": "Successfully logged out",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
