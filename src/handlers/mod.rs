pub mod auth;
pub mod files;
pub mod shares;
pub mod system;

use tracing::error;

use crate::config::AppConfig;
use crate::database::create_connection_pool;
use crate::database::service::DatabaseService;
use crate::middleware::auth::JwtService;

use anyhow::Result;
/// Application state that will be shared across all handlers
/// This is wrapped in Arc<> in main.rs for efficient sharing across threads
pub struct AppState {
    pub db_service: DatabaseService,
    pub jwt_service: JwtService,
}

impl AppState {
    pub async fn new(app_config: &AppConfig) -> Result<Self> {
        // Create database connection pool
        let db_pool = create_connection_pool(&app_config.database_url)
            .await
            .map_err(|e| {
                error!("Failed to create database connection pool: {}", e);
                e
            })?;

        let db_service = DatabaseService::new(db_pool);
        let jwt_service =
            JwtService::new(&app_config.jwt_secret, Some(app_config.jwt_expires_hours));
        Ok(Self {
            db_service,
            jwt_service,
        })
    }
}
//  implement for AppState for flexibility
impl crate::middleware::auth::FromRef<AppState> for DatabaseService {
    fn from_ref(app_state: &AppState) -> DatabaseService {
        app_state.db_service.clone()
    }
}

impl crate::middleware::auth::FromRef<AppState> for JwtService {
    fn from_ref(app_state: &AppState) -> JwtService {
        app_state.jwt_service.clone()
    }
}
