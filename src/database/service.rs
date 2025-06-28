use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::database::models::{
    CreateShareRequest, CreateUserRequest, FileInfo, FileListResponse, FileSearchRequest,
    ShareInfo, ShareListResponse, UserInfo,
};

use crate::utils::{hash_password, verify_password};

#[allow(dead_code)]
pub struct DatabaseService {
    pool: PgPool,
}

#[allow(dead_code)]
impl DatabaseService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // User management
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserInfo> {
        // Hash password with Argon2
        let password_hash = hash_password(&request.password)?;
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, is_admin, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(user_id)
        .bind(&request.username)
        .bind(&request.email)
        .bind(password_hash)
        .bind(false) // Default to non-admin
        .bind(&request.metadata)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(UserInfo {
            id: user_id,
            username: request.username,
            email: request.email,
            is_admin: false,
            metadata: request.metadata,
        })
    }

    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<UserInfo>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, is_admin, metadata FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let stored_hash: String = row.get("password_hash");
            if verify_password(&password, &stored_hash)? {
                return Ok(Some(UserInfo {
                    id: row.get("id"),
                    username: row.get("username"),
                    email: row.get("email"),
                    is_admin: row.get("is_admin"),
                    metadata: row.get("metadata"),
                }));
            }
        }

        Ok(None)
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<UserInfo>> {
        let row =
            sqlx::query("SELECT id, username, email, is_admin, metadata FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row.map(|row| UserInfo {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            is_admin: row.get("is_admin"),
            metadata: row.get("metadata"),
        }))
    }

    // Session management
    pub async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: DateTime<Utc>,
    ) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, token_hash, expires_at, created_at, last_used_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(session_id)
    }

    pub async fn validate_session(&self, token_hash: &str) -> Result<Option<UserInfo>> {
        let row = sqlx::query(
            r#"
            SELECT u.id, u.username, u.email, u.is_admin, u.metadata
            FROM users u
            INNER JOIN user_sessions s ON u.id = s.user_id
            WHERE s.token_hash = $1 AND s.expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            // Update last_used_at
            sqlx::query("UPDATE user_sessions SET last_used_at = NOW() WHERE token_hash = $1")
                .bind(token_hash)
                .execute(&self.pool)
                .await?;

            return Ok(Some(UserInfo {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                is_admin: row.get("is_admin"),
                metadata: row.get("metadata"),
            }));
        }

        Ok(None)
    }

    pub async fn revoke_session(&self, token_hash: &str) -> Result<()> {
        sqlx::query("DELETE FROM user_sessions WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // File management
    pub async fn create_file_metadata(
        &self,
        name: String,
        path: String,
        size: i64,
        mime_type: String,
        checksum: String,
        owner_id: Uuid,
        tags: Vec<String>,
        metadata: JsonValue,
    ) -> Result<FileInfo> {
        let file_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO files (id, name, path, size, mime_type, checksum, owner_id, tags, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(file_id)
        .bind(&name)
        .bind(&path)
        .bind(size)
        .bind(&mime_type)
        .bind(checksum)
        .bind(owner_id)
        .bind(&tags)
        .bind(&metadata)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(FileInfo {
            id: file_id,
            name,
            path,
            size,
            mime_type,
            owner_id,
            tags,
            metadata,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_file_by_id(&self, file_id: Uuid) -> Result<Option<FileInfo>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, path, size, mime_type, owner_id, tags, metadata, created_at, updated_at
            FROM files WHERE id = $1
            "#,
        )
        .bind(file_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| FileInfo {
            id: row.get("id"),
            name: row.get("name"),
            path: row.get("path"),
            size: row.get("size"),
            mime_type: row.get("mime_type"),
            owner_id: row.get("owner_id"),
            tags: row.get("tags"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    pub async fn search_files(&self, request: FileSearchRequest) -> Result<FileListResponse> {
        let limit = request.limit.unwrap_or(50).min(100); // Max 100 results
        let offset = request.offset.unwrap_or(0);

        // Use QueryBuilder for safe parameter binding
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT id, name, path, size, mime_type, owner_id, tags, metadata, created_at, updated_at FROM files WHERE 1=1"
        );

        // Add conditions using QueryBuilder
        if let Some(owner_id) = request.owner_id {
            query_builder.push(" AND owner_id = ");
            query_builder.push_bind(owner_id);
        }

        if let Some(mime_type) = &request.mime_type {
            query_builder.push(" AND mime_type = ");
            query_builder.push_bind(mime_type);
        }

        if let Some(tags) = &request.tags {
            if !tags.is_empty() {
                query_builder.push(" AND tags && ");
                query_builder.push_bind(tags);
            }
        }

        if let Some(search_query) = &request.query {
            query_builder.push(" AND search_vector @@ plainto_tsquery('english', ");
            query_builder.push_bind(search_query);
            query_builder.push(")");
        }

        // Add ordering
        if request.query.is_some() {
            query_builder.push(" ORDER BY ts_rank(search_vector, plainto_tsquery('english', ");
            query_builder.push_bind(request.query.as_ref().unwrap());
            query_builder.push(")) DESC");
        } else {
            query_builder.push(" ORDER BY created_at DESC");
        }

        // Add pagination
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        // Execute query
        let query = query_builder.build();
        let rows = query.fetch_all(&self.pool).await?;

        let files: Vec<FileInfo> = rows
            .into_iter()
            .map(|row| FileInfo {
                id: row.get("id"),
                name: row.get("name"),
                path: row.get("path"),
                size: row.get("size"),
                mime_type: row.get("mime_type"),
                owner_id: row.get("owner_id"),
                tags: row.get("tags"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        // Get total count with similar approach
        let mut count_builder =
            sqlx::QueryBuilder::new("SELECT COUNT(*) as total FROM files WHERE 1=1");

        if let Some(owner_id) = request.owner_id {
            count_builder.push(" AND owner_id = ");
            count_builder.push_bind(owner_id);
        }

        if let Some(mime_type) = &request.mime_type {
            count_builder.push(" AND mime_type = ");
            count_builder.push_bind(mime_type);
        }

        if let Some(tags) = &request.tags {
            if !tags.is_empty() {
                count_builder.push(" AND tags && ");
                count_builder.push_bind(tags);
            }
        }

        if let Some(search_query) = &request.query {
            count_builder.push(" AND search_vector @@ plainto_tsquery('english', ");
            count_builder.push_bind(search_query);
            count_builder.push(")");
        }

        let count_query = count_builder.build();
        let total_row = count_query.fetch_one(&self.pool).await?;
        let total: i64 = total_row.get("total");

        Ok(FileListResponse {
            files,
            total,
            page: offset / limit,
            per_page: limit,
        })
    }

    pub async fn delete_file(&self, file_id: Uuid, owner_id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM files WHERE id = $1 AND owner_id = $2")
            .bind(file_id)
            .bind(owner_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Share management
    pub async fn create_share(
        &self,
        request: CreateShareRequest,
        created_by: Uuid,
    ) -> Result<ShareInfo> {
        let share_id = Uuid::new_v4();
        let share_hash = self.generate_secure_hash();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO shares (id, file_id, share_hash, expires_at, max_downloads, download_count, created_by, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(share_id)
        .bind(request.file_id)
        .bind(&share_hash)
        .bind(request.expires_at)
        .bind(request.max_downloads)
        .bind(0) // Initial download count
        .bind(created_by)
        .bind(&request.metadata)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ShareInfo {
            id: share_id,
            file_id: request.file_id,
            share_hash,
            expires_at: request.expires_at,
            max_downloads: request.max_downloads,
            download_count: 0,
            metadata: request.metadata,
            created_at: now,
        })
    }

    pub async fn get_share_by_hash(
        &self,
        share_hash: &str,
    ) -> Result<Option<(ShareInfo, FileInfo)>> {
        let row = sqlx::query(
            r#"
            SELECT 
                s.id as share_id, s.file_id, s.share_hash, s.expires_at, s.max_downloads, 
                s.download_count, s.metadata as share_metadata, s.created_at as share_created_at,
                f.name, f.path, f.size, f.mime_type, f.owner_id, f.tags, 
                f.metadata as file_metadata, f.created_at as file_created_at, f.updated_at
            FROM shares s
            INNER JOIN files f ON s.file_id = f.id
            WHERE s.share_hash = $1 
            AND (s.expires_at IS NULL OR s.expires_at > NOW())
            AND (s.max_downloads IS NULL OR s.download_count < s.max_downloads)
            "#,
        )
        .bind(share_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let share_info = ShareInfo {
                id: row.get("share_id"),
                file_id: row.get("file_id"),
                share_hash: row.get("share_hash"),
                expires_at: row.get("expires_at"),
                max_downloads: row.get("max_downloads"),
                download_count: row.get("download_count"),
                metadata: row.get("share_metadata"),
                created_at: row.get("share_created_at"),
            };

            let file_info = FileInfo {
                id: row.get("file_id"),
                name: row.get("name"),
                path: row.get("path"),
                size: row.get("size"),
                mime_type: row.get("mime_type"),
                owner_id: row.get("owner_id"),
                tags: row.get("tags"),
                metadata: row.get("file_metadata"),
                created_at: row.get("file_created_at"),
                updated_at: row.get("updated_at"),
            };

            (share_info, file_info)
        }))
    }

    pub async fn increment_share_download(&self, share_hash: &str) -> Result<()> {
        sqlx::query("UPDATE shares SET download_count = download_count + 1 WHERE share_hash = $1")
            .bind(share_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_user_shares(&self, user_id: Uuid) -> Result<ShareListResponse> {
        let rows = sqlx::query(
            r#"
            SELECT id, file_id, share_hash, expires_at, max_downloads, download_count, metadata, created_at
            FROM shares 
            WHERE created_by = $1 
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let shares: Vec<ShareInfo> = rows
            .into_iter()
            .map(|row| ShareInfo {
                id: row.get("id"),
                file_id: row.get("file_id"),
                share_hash: row.get("share_hash"),
                expires_at: row.get("expires_at"),
                max_downloads: row.get("max_downloads"),
                download_count: row.get("download_count"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
            })
            .collect();

        let total = shares.len() as i64;

        Ok(ShareListResponse { shares, total })
    }

    // Utility functions
    fn generate_secure_hash(&self) -> String {
        use rand::Rng;
        use sha2::{Digest, Sha256};

        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();
        let mut hasher = Sha256::new();
        hasher.update(random_bytes);
        let result = hasher.finalize();
        format!("{:x}", result)[..16].to_string() // Take first 16 chars
    }

    // Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }
}
