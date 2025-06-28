use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub password_min_length: usize,
    pub rate_limit_requests_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_path: PathBuf,
    pub max_file_size_mb: u64,
    pub allowed_extensions: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                cors_origins: vec!["http://localhost:5173".to_string()],
            },
            database: DatabaseConfig {
                url: "postgres://nas_user:nas_password@localhost:5432/simple_nas".to_string(),
                max_connections: 20,
                min_connections: 5,
                acquire_timeout_secs: 30,
                idle_timeout_secs: 600,
                max_lifetime_secs: 1800,
            },
            security: SecurityConfig {
                jwt_secret: "your-secret-key-change-in-production".to_string(),
                jwt_expiration_hours: 24,
                password_min_length: 8,
                rate_limit_requests_per_minute: 60,
            },
            storage: StorageConfig {
                base_path: PathBuf::from("./storage"),
                max_file_size_mb: 1024, // 1GB
                allowed_extensions: vec![
                    "txt".to_string(),
                    "pdf".to_string(),
                    "jpg".to_string(),
                    "jpeg".to_string(),
                    "png".to_string(),
                    "gif".to_string(),
                    "mp4".to_string(),
                    "mp3".to_string(),
                    "zip".to_string(),
                ],
            },
        }
    }
}
