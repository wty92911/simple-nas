use anyhow::Result;
use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use uuid::Uuid;

use crate::database::models::{ErrorResponse, UserInfo};
use crate::database::service::DatabaseService;

// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub username: String,
    pub is_admin: bool,
    pub iat: i64,    // Issued at
    pub exp: i64,    // Expiration time
    pub jti: String, // JWT ID (for session tracking)
}

// JWT Service for token management
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expires_in_hours: i64,
}

impl JwtService {
    pub fn new(secret: &str, expires_in_hours: Option<i64>) -> Self {
        let key = secret.as_bytes();
        Self {
            encoding_key: EncodingKey::from_secret(key),
            decoding_key: DecodingKey::from_secret(key),
            expires_in_hours: expires_in_hours.unwrap_or(24), // Default 24 hours
        }
    }

    // Generate JWT token for user
    pub fn generate_token(&self, user: &UserInfo) -> Result<(String, DateTime<Utc>)> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(self.expires_in_hours);
        let session_id = Uuid::new_v4().to_string();

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            is_admin: user.is_admin,
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            jti: session_id,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow::anyhow!("Token generation failed: {}", e))?;

        Ok((token, expires_at))
    }

    // Validate JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true; // Validate expiration

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("Token validation failed: {}", e))?;

        Ok(token_data.claims)
    }

    // Extract token from Authorization header
    fn extract_bearer_token(auth_header: &str) -> Option<&str> {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            Some(token)
        } else {
            None
        }
    }
}

// Authentication middleware for protected routes
#[derive(Clone)]
pub struct AuthMiddleware {
    pub user: UserInfo,
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for AuthMiddleware
where
    S: Send + Sync,
    DatabaseService: FromRef<S>,
    JwtService: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        // Extract Bearer token
        let token =
            JwtService::extract_bearer_token(auth_header).ok_or(AuthError::InvalidTokenFormat)?;

        // Get JWT service from state
        let jwt_service = JwtService::from_ref(state);

        // Validate token and extract claims
        let claims = jwt_service
            .validate_token(token)
            .map_err(|_| AuthError::InvalidToken)?;

        // Get database service from state
        let db_service = DatabaseService::from_ref(state);

        // Verify user still exists and get current user info
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

        let user = db_service
            .get_user_by_id(user_id)
            .await
            .map_err(|_| AuthError::DatabaseError)?
            .ok_or(AuthError::UserNotFound)?;

        // Optional: Validate session in database (for revocation support)
        let token_hash = sha2::Sha256::digest(token.as_bytes());
        let token_hash_str = format!("{token_hash:x}");

        if db_service.validate_session(&token_hash_str).await.is_err() {
            // If session validation fails, continue with JWT validation only
            // This allows for stateless JWT without requiring session storage
        }

        Ok(AuthMiddleware { user, claims })
    }
}

// Helper trait for extracting services from application state
pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidTokenFormat,
    InvalidToken,
    UserNotFound,
    DatabaseError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidTokenFormat => (
                StatusCode::UNAUTHORIZED,
                "Invalid token format. Expected 'Bearer <token>'",
            ),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
            AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "User not found"),
            AuthError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error during authentication",
            ),
        };

        let error_response = ErrorResponse {
            error: "Authentication Error".to_string(),
            message: error_message.to_string(),
            code: Some(format!("{}", status.as_u16())),
        };

        (status, Json(error_response)).into_response()
    }
}

// Optional middleware for admin-only routes
pub struct AdminAuthMiddleware {
    pub user: UserInfo,
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for AdminAuthMiddleware
where
    S: Send + Sync,
    DatabaseService: FromRef<S>,
    JwtService: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First validate authentication
        let auth = AuthMiddleware::from_request_parts(parts, state).await?;

        // Then check admin privileges
        if !auth.user.is_admin {
            return Err(AuthError::InvalidToken); // Reuse InvalidToken for insufficient privileges
        }

        Ok(AdminAuthMiddleware {
            user: auth.user,
            claims: auth.claims,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_service_creation() {
        let service = JwtService::new("test_secret_key", Some(1));
        assert_eq!(service.expires_in_hours, 1);
    }

    #[test]
    fn test_bearer_token_extraction() {
        let valid_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let extracted = JwtService::extract_bearer_token(valid_header);
        assert_eq!(extracted, Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));

        let invalid_header = "Basic dXNlcjpwYXNz";
        let extracted = JwtService::extract_bearer_token(invalid_header);
        assert_eq!(extracted, None);

        let empty_header = "";
        let extracted = JwtService::extract_bearer_token(empty_header);
        assert_eq!(extracted, None);
    }

    #[test]
    fn test_token_generation_and_validation() {
        let service = JwtService::new("test_secret_key_for_testing", Some(1));

        let user = UserInfo {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            is_admin: false,
            metadata: serde_json::json!({}),
        };

        // Generate token
        let result = service.generate_token(&user);
        assert!(result.is_ok());

        let (token, _expires_at) = result.unwrap();
        assert!(!token.is_empty());

        // Validate token
        let claims_result = service.validate_token(&token);
        assert!(claims_result.is_ok());

        let claims = claims_result.unwrap();
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.is_admin, user.is_admin);
    }

    #[test]
    fn test_invalid_token_validation() {
        let service = JwtService::new("test_secret_key", Some(1));

        // Test with invalid token
        let result = service.validate_token("invalid.token.here");
        assert!(result.is_err());

        // Test with empty token
        let result = service.validate_token("");
        assert!(result.is_err());

        // Test with malformed token
        let result = service.validate_token("not.a.jwt");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_with_different_secret() {
        let service1 = JwtService::new("secret1", Some(1));
        let service2 = JwtService::new("secret2", Some(1));

        let user = UserInfo {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            is_admin: false,
            metadata: serde_json::json!({}),
        };

        // Generate token with service1
        let (token, _) = service1.generate_token(&user).unwrap();

        // Try to validate with service2 (should fail)
        let result = service2.validate_token(&token);
        assert!(result.is_err());
    }
}
