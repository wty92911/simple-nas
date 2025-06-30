// PostgreSQL schema definitions for Simple Home NAS
// These constants reference the migration files and provide query helpers
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

// Database connection and migration utilities
pub async fn create_connection_pool(database_url: &str) -> Result<PgPool> {
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
        .connect(database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Database connection failed: {}", e))
}

// // Run database migrations
// pub async fn run_migrations(pool: &PgPool) -> Result<()> {
//     sqlx::migrate!("./migrations")
//         .run(pool)
//         .await
//         .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))
// }

// // Database health check
// pub async fn health_check(pool: &PgPool) -> Result<()> {
//     sqlx::query("SELECT 1")
//         .fetch_one(pool)
//         .await
//         .map_err(|e| anyhow::anyhow!("Database health check failed: {}", e))?;
//     Ok(())
// }

// Search functionality using PostgreSQL full-text search
pub async fn search_files(
    pool: &PgPool,
    search_query: &str,
    owner_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> Result<Vec<sqlx::postgres::PgRow>> {
    let query = if let Some(owner_id) = owner_id {
        sqlx::query(
            r#"
            SELECT id, name, path, size, mime_type, checksum, owner_id, tags, metadata, created_at, updated_at
            FROM files
            WHERE search_vector @@ plainto_tsquery('english', $1)
              AND owner_id = $2
            ORDER BY ts_rank(search_vector, plainto_tsquery('english', $1)) DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(search_query)
        .bind(owner_id)
        .bind(limit)
        .bind(offset)
    } else {
        sqlx::query(
            r#"
            SELECT id, name, path, size, mime_type, checksum, owner_id, tags, metadata, created_at, updated_at
            FROM files
            WHERE search_vector @@ plainto_tsquery('english', $1)
            ORDER BY ts_rank(search_vector, plainto_tsquery('english', $1)) DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(search_query)
        .bind(limit)
        .bind(offset)
    };

    query
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow::anyhow!("File search failed: {}", e))
}

// Tag-based file filtering
pub async fn find_files_by_tags(
    pool: &PgPool,
    tags: &[String],
    owner_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> Result<Vec<sqlx::postgres::PgRow>> {
    let query = if let Some(owner_id) = owner_id {
        sqlx::query(
            r#"
            SELECT id, name, path, size, mime_type, checksum, owner_id, tags, metadata, created_at, updated_at
            FROM files
            WHERE tags && $1 AND owner_id = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tags)
        .bind(owner_id)
        .bind(limit)
        .bind(offset)
    } else {
        sqlx::query(
            r#"
            SELECT id, name, path, size, mime_type, checksum, owner_id, tags, metadata, created_at, updated_at
            FROM files
            WHERE tags && $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tags)
        .bind(limit)
        .bind(offset)
    };

    query
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Tag-based file search failed: {}", e))
}

// Cleanup expired sessions
pub async fn cleanup_expired_sessions(pool: &PgPool) -> Result<u64> {
    let result = sqlx::query("DELETE FROM user_sessions WHERE expires_at < NOW()")
        .execute(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Session cleanup failed: {}", e))?;

    Ok(result.rows_affected())
}

// // Get database statistics
// pub async fn get_database_stats(pool: &PgPool) -> Result<DatabaseStats> {
//     let row = sqlx::query(
//         r#"
//         SELECT
//             (SELECT COUNT(*) FROM users) as user_count,
//             (SELECT COUNT(*) FROM files) as file_count,
//             (SELECT COUNT(*) FROM shares) as share_count,
//             (SELECT COUNT(*) FROM user_sessions WHERE expires_at > NOW()) as active_session_count,
//             (SELECT COALESCE(SUM(size), 0) FROM files) as total_file_size
//         "#,
//     )
//     .fetch_one(pool)
//     .await
//     .map_err(|e| anyhow::anyhow!("Failed to get database stats: {}", e))?;

//     Ok(DatabaseStats {
//         user_count: row.get("user_count"),
//         file_count: row.get("file_count"),
//         share_count: row.get("share_count"),
//         active_session_count: row.get("active_session_count"),
//         total_file_size: row.get("total_file_size"),
//     })
// }

#[derive(Debug, serde::Serialize)]
pub struct DatabaseStats {
    pub user_count: i64,
    pub file_count: i64,
    pub share_count: i64,
    pub active_session_count: i64,
    pub total_file_size: i64,
}
