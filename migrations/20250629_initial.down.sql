-- Revert migration: 20240101000000_initial
-- Description: Drop all tables, functions, triggers, and data created in the initial migration

-- Remove initial admin user
DELETE FROM users WHERE username = 'admin' AND email = 'admin@localhost';

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_files_updated_at ON files;
DROP TRIGGER IF EXISTS trigger_users_updated_at ON users;
DROP TRIGGER IF EXISTS files_search_vector_update ON files;

-- Drop functions
DROP FUNCTION IF EXISTS update_updated_at_column();
DROP FUNCTION IF EXISTS update_files_search_vector();

-- Drop indexes (explicit drops for clarity, though they'll be dropped with tables)
DROP INDEX IF EXISTS idx_shares_expires_at;
DROP INDEX IF EXISTS idx_shares_created_by;
DROP INDEX IF EXISTS idx_shares_hash;
DROP INDEX IF EXISTS idx_shares_file_id;
DROP INDEX IF EXISTS idx_files_created_at;
DROP INDEX IF EXISTS idx_files_search_vector;
DROP INDEX IF EXISTS idx_files_tags;
DROP INDEX IF EXISTS idx_files_mime_type;
DROP INDEX IF EXISTS idx_files_owner_id;
DROP INDEX IF EXISTS idx_user_sessions_expires_at;
DROP INDEX IF EXISTS idx_user_sessions_token_hash;
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_username;

-- Drop tables (in reverse order due to foreign key constraints)
DROP TABLE IF EXISTS shares;
DROP TABLE IF EXISTS files;
DROP TABLE IF EXISTS user_sessions;
DROP TABLE IF EXISTS users;

-- Note: Extensions are not dropped as they might be used by other parts of the system
-- If you need to drop them, uncomment the following lines:
-- DROP EXTENSION IF EXISTS "pg_trgm";
-- DROP EXTENSION IF EXISTS "uuid-ossp";
