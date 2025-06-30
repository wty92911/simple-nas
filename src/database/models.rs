use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub metadata: JsonValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadRequest {
    pub name: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: JsonValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSearchRequest {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub mime_type: Option<String>,
    pub owner_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShareRequest {
    pub file_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_downloads: Option<i32>,
    #[serde(default)]
    pub metadata: JsonValue,
}

// Response DTOs for API endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub metadata: JsonValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub size: i64,
    pub mime_type: String,
    pub owner_id: Uuid,
    pub tags: Vec<String>,
    pub metadata: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareInfo {
    pub id: Uuid,
    pub file_id: Uuid,
    pub share_hash: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_downloads: Option<i32>,
    pub download_count: i32,
    pub metadata: JsonValue,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListResponse {
    pub files: Vec<FileInfo>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareListResponse {
    pub shares: Vec<ShareInfo>,
    pub total: i64,
}

// Error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: Option<String>,
}
