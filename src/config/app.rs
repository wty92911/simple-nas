use std::{fs::File, path::Path};

use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub jwt_expires_hours: i64,

    pub concurrency_limit: usize,
    pub rate_limit_per_second: u64,

    pub database_url: String,
    pub security_config: SecurityConfig,
    pub port: u16,
}

impl AppConfig {
    pub fn from_yml(path: impl AsRef<Path>) -> Result<Self> {
        let file =
            File::open(path).map_err(|e| anyhow::anyhow!("Failed to open config file: {}", e))?;
        let config = serde_yaml::from_reader(file)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;
        Ok(config)
    }
}

// Security configuration
#[derive(Clone, Deserialize)]
pub struct SecurityConfig {
    pub cors_enabled: bool,
    pub rate_limiting_enabled: bool,
    pub requests_per_minute: u32,
    pub allowed_origins: Vec<String>,
    pub security_headers_enabled: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            cors_enabled: true,
            rate_limiting_enabled: true,
            requests_per_minute: 60,
            allowed_origins: vec!["http://localhost:3000".to_string()],
            security_headers_enabled: true,
        }
    }
}
