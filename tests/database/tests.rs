use anyhow::Result;
use chrono::{Duration, Utc};
use serde_json::json;
use simple_nas::database::models::{CreateShareRequest, CreateUserRequest, FileSearchRequest};
use simple_nas::database::service::DatabaseService;
use sqlx_db_tester::TestPg;
use uuid::Uuid;

// Helper function to create test database
pub async fn setup_test_db() -> Result<(TestPg, DatabaseService)> {
    let tdb = TestPg::new(
        "postgres://postgres:postgres@localhost:5432".to_string(),
        std::path::Path::new("./migrations"),
    );
    let pool = tdb.get_pool().await;
    let service = DatabaseService::new(pool);
    Ok((tdb, service))
}

// Helper function to create a test user
pub async fn create_test_user(service: &DatabaseService, username: &str) -> Result<Uuid> {
    let request = CreateUserRequest {
        username: username.to_string(),
        email: format!("{username}@example.com"),
        password: "test_password123".to_string(),
        metadata: json!({"test": true}),
    };

    let user_info = service.create_user(request).await?;
    Ok(user_info.id)
}

#[tokio::test]
async fn test_health_check() -> Result<()> {
    let (_tdb, service) = setup_test_db().await?;

    // Health check should pass
    service.health_check().await?;

    Ok(())
}

#[tokio::test]
async fn test_user_creation_and_authentication() -> Result<()> {
    let (_tdb, service) = setup_test_db().await?;

    let request = CreateUserRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "secure_password123".to_string(),
        metadata: json!({"role": "user", "preferences": {"theme": "dark"}}),
    };

    // Create user
    let user_info = service.create_user(request).await?;
    assert_eq!(user_info.username, "testuser");
    assert_eq!(user_info.email, "test@example.com");
    assert!(!user_info.is_admin);

    // Test authentication with correct password
    let auth_result = service
        .authenticate_user("testuser", "secure_password123")
        .await?;
    assert!(auth_result.is_some());
    let authenticated_user = auth_result.unwrap();
    assert_eq!(authenticated_user.id, user_info.id);
    assert_eq!(authenticated_user.username, "testuser");

    // Test authentication with wrong password
    let auth_fail = service
        .authenticate_user("testuser", "wrong_password")
        .await?;
    assert!(auth_fail.is_none());

    // Test authentication with non-existent user
    let auth_nonexistent = service.authenticate_user("nonexistent", "password").await?;
    assert!(auth_nonexistent.is_none());

    Ok(())
}

#[tokio::test]
async fn test_get_user_by_id() -> Result<()> {
    let (_tdb, service) = setup_test_db().await?;

    let user_id = create_test_user(&service, "testuser").await?;

    // Get existing user
    let user_info = service.get_user_by_id(user_id).await?;
    assert!(user_info.is_some());
    let user = user_info.unwrap();
    assert_eq!(user.id, user_id);
    assert_eq!(user.username, "testuser");

    // Get non-existent user
    let non_existent = service.get_user_by_id(Uuid::new_v4()).await?;
    assert!(non_existent.is_none());

    Ok(())
}

#[tokio::test]
async fn test_session_management() -> Result<()> {
    let (_tdb, service) = setup_test_db().await?;

    let user_id = create_test_user(&service, "sessionuser").await?;
    let token_hash = "test_token_hash_123".to_string();
    let expires_at = Utc::now() + Duration::hours(24);

    // Create session
    let session_id = service
        .create_session(user_id, token_hash.clone(), expires_at)
        .await?;
    assert!(!session_id.is_nil());

    // Validate session
    let session_user = service.validate_session(&token_hash).await?;
    assert!(session_user.is_some());
    let user = session_user.unwrap();
    assert_eq!(user.id, user_id);
    assert_eq!(user.username, "sessionuser");

    // Revoke session
    service.revoke_session(&token_hash).await?;

    // Validate revoked session should fail
    let revoked_session = service.validate_session(&token_hash).await?;
    assert!(revoked_session.is_none());

    Ok(())
}

#[tokio::test]
async fn test_expired_session() -> Result<()> {
    let (_tdb, service) = setup_test_db().await?;

    let user_id = create_test_user(&service, "expireduser").await?;
    let token_hash = "expired_token_hash".to_string();
    let expires_at = Utc::now() - Duration::hours(1); // Expired 1 hour ago

    // Create expired session
    service
        .create_session(user_id, token_hash.clone(), expires_at)
        .await?;

    // Validate expired session should fail
    let expired_session = service.validate_session(&token_hash).await?;
    assert!(expired_session.is_none());

    Ok(())
}

#[tokio::test]
async fn test_file_metadata_operations() -> Result<()> {
    let (_tdb, service) = setup_test_db().await?;

    let user_id = create_test_user(&service, "fileuser").await?;

    // Create file metadata
    let file_info = service
        .create_file_metadata(
            "test_document.pdf".to_string(),
            "/uploads/test_document.pdf".to_string(),
            1024000, // 1MB
            "application/pdf".to_string(),
            "sha256:abcd1234".to_string(),
            user_id,
            vec!["document".to_string(), "pdf".to_string()],
            json!({"description": "Test PDF document", "category": "work"}),
        )
        .await?;

    assert_eq!(file_info.name, "test_document.pdf");
    assert_eq!(file_info.size, 1024000);
    assert_eq!(file_info.mime_type, "application/pdf");
    assert_eq!(file_info.owner_id, user_id);
    assert_eq!(file_info.tags, vec!["document", "pdf"]);

    // Get file by ID
    let retrieved_file = service.get_file_by_id(file_info.id).await?;
    assert!(retrieved_file.is_some());
    let file = retrieved_file.unwrap();
    assert_eq!(file.id, file_info.id);
    assert_eq!(file.name, "test_document.pdf");

    // Delete file
    let deleted = service.delete_file(file_info.id, user_id).await?;
    assert!(deleted);

    // Try to get deleted file
    let deleted_file = service.get_file_by_id(file_info.id).await?;
    assert!(deleted_file.is_none());

    // Try to delete non-existent file
    let not_deleted = service.delete_file(Uuid::new_v4(), user_id).await?;
    assert!(!not_deleted);

    Ok(())
}

#[tokio::test]
async fn test_file_search() -> Result<()> {
    let (_tdb, service) = setup_test_db().await?;

    let user_id = create_test_user(&service, "searchuser").await?;

    // Create multiple test files
    service
        .create_file_metadata(
            "document1.pdf".to_string(),
            "/uploads/document1.pdf".to_string(),
            1000,
            "application/pdf".to_string(),
            "sha256:doc1".to_string(),
            user_id,
            vec!["document".to_string(), "work".to_string()],
            json!({"type": "report"}),
        )
        .await?;

    service
        .create_file_metadata(
            "image1.jpg".to_string(),
            "/uploads/image1.jpg".to_string(),
            2000,
            "image/jpeg".to_string(),
            "sha256:img1".to_string(),
            user_id,
            vec!["image".to_string(), "personal".to_string()],
            json!({"type": "photo"}),
        )
        .await?;

    service
        .create_file_metadata(
            "document2.docx".to_string(),
            "/uploads/document2.docx".to_string(),
            1500,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            "sha256:doc2".to_string(),
            user_id,
            vec!["document".to_string(), "personal".to_string()],
            json!({"type": "letter"}),
        )
        .await?;

    // Search all files for user
    let search_request = FileSearchRequest {
        query: None,
        tags: None,
        mime_type: None,
        owner_id: Some(user_id),
        limit: Some(10),
        offset: Some(0),
    };

    let search_result = service.search_files(search_request).await?;
    assert_eq!(search_result.files.len(), 3);
    assert_eq!(search_result.total, 3);

    // Search by tags
    let tag_search = FileSearchRequest {
        query: None,
        tags: Some(vec!["document".to_string()]),
        mime_type: None,
        owner_id: Some(user_id),
        limit: Some(10),
        offset: Some(0),
    };

    let tag_result = service.search_files(tag_search).await?;
    assert_eq!(tag_result.files.len(), 2); // document1.pdf and document2.docx

    // Search by mime type
    let mime_search = FileSearchRequest {
        query: None,
        tags: None,
        mime_type: Some("application/pdf".to_string()),
        owner_id: Some(user_id),
        limit: Some(10),
        offset: Some(0),
    };

    let mime_result = service.search_files(mime_search).await?;
    assert_eq!(mime_result.files.len(), 1); // Only document1.pdf
    assert_eq!(mime_result.files[0].name, "document1.pdf");

    Ok(())
}

#[tokio::test]
async fn test_share_operations() -> Result<()> {
    let (_tdb, service) = setup_test_db().await?;

    let user_id = create_test_user(&service, "shareuser").await?;

    // Create a file to share
    let file_info = service
        .create_file_metadata(
            "shared_document.pdf".to_string(),
            "/uploads/shared_document.pdf".to_string(),
            500000,
            "application/pdf".to_string(),
            "sha256:shared123".to_string(),
            user_id,
            vec!["shared".to_string()],
            json!({"shared": true}),
        )
        .await?;

    // Create share
    let share_request = CreateShareRequest {
        file_id: file_info.id,
        expires_at: Some(Utc::now() + Duration::days(7)),
        max_downloads: Some(5),
        metadata: json!({"description": "Shared test document"}),
    };

    let share_info = service.create_share(share_request, user_id).await?;
    assert_eq!(share_info.file_id, file_info.id);
    assert_eq!(share_info.max_downloads, Some(5));
    assert_eq!(share_info.download_count, 0);
    assert!(!share_info.share_hash.is_empty());

    // Get share by hash
    let retrieved_share = service.get_share_by_hash(&share_info.share_hash).await?;
    assert!(retrieved_share.is_some());
    let (share, file) = retrieved_share.unwrap();
    assert_eq!(share.id, share_info.id);
    assert_eq!(file.id, file_info.id);
    assert_eq!(file.name, "shared_document.pdf");

    // Increment download count
    service
        .increment_share_download(&share_info.share_hash)
        .await?;

    // Get user shares
    let user_shares = service.get_user_shares(user_id).await?;
    assert_eq!(user_shares.shares.len(), 1);
    assert_eq!(user_shares.total, 1);
    assert_eq!(user_shares.shares[0].download_count, 1); // Should be incremented

    Ok(())
}

#[tokio::test]
async fn test_expired_share() -> Result<()> {
    let (_tdb, service) = setup_test_db().await?;

    let user_id = create_test_user(&service, "expiredshareuser").await?;

    // Create a file
    let file_info = service
        .create_file_metadata(
            "expired_share.pdf".to_string(),
            "/uploads/expired_share.pdf".to_string(),
            100000,
            "application/pdf".to_string(),
            "sha256:expired".to_string(),
            user_id,
            vec!["expired".to_string()],
            json!({}),
        )
        .await?;

    // Create expired share
    let share_request = CreateShareRequest {
        file_id: file_info.id,
        expires_at: Some(Utc::now() - Duration::hours(1)), // Expired 1 hour ago
        max_downloads: None,
        metadata: json!({}),
    };

    let share_info = service.create_share(share_request, user_id).await?;

    // Try to get expired share - should return None
    let expired_share = service.get_share_by_hash(&share_info.share_hash).await?;
    assert!(expired_share.is_none());

    Ok(())
}

#[tokio::test]
async fn test_max_downloads_exceeded() -> Result<()> {
    let (_tdb, service) = setup_test_db().await?;

    let user_id = create_test_user(&service, "maxdownloaduser").await?;

    // Create a file
    let file_info = service
        .create_file_metadata(
            "limited_download.pdf".to_string(),
            "/uploads/limited_download.pdf".to_string(),
            200000,
            "application/pdf".to_string(),
            "sha256:limited".to_string(),
            user_id,
            vec!["limited".to_string()],
            json!({}),
        )
        .await?;

    // Create share with max 2 downloads
    let share_request = CreateShareRequest {
        file_id: file_info.id,
        expires_at: None,
        max_downloads: Some(2),
        metadata: json!({}),
    };

    let share_info = service.create_share(share_request, user_id).await?;

    // First download - should work
    let share1 = service.get_share_by_hash(&share_info.share_hash).await?;
    assert!(share1.is_some());
    service
        .increment_share_download(&share_info.share_hash)
        .await?;

    // Second download - should work
    let share2 = service.get_share_by_hash(&share_info.share_hash).await?;
    assert!(share2.is_some());
    service
        .increment_share_download(&share_info.share_hash)
        .await?;

    // Third download - should fail (exceeded max downloads)
    let share3 = service.get_share_by_hash(&share_info.share_hash).await?;
    assert!(share3.is_none());

    Ok(())
}
