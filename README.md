# Simple Home NAS

A simple, secure home NAS system built with Rust and React.

## 🚀 Features

- **File Management**: Upload, download, organize files with metadata and tags
- **Secure Sharing**: Share files with expiring links and download limits
- **Authentication**: JWT-based authentication with session management
- **PostgreSQL Database**: Advanced features including full-text search and JSONB metadata
- **Cross-Platform**: Supports macOS, Linux, and Windows

## 🛠️ Technology Stack

- **Backend**: Rust + Axum + PostgreSQL + SQLx
- **Database**: PostgreSQL 16+ with advanced features
- **Authentication**: JWT tokens with Argon2 password hashing
- **Frontend**: React + TypeScript (planned)

## 📋 Prerequisites

- Rust 1.70+ (with Cargo)
- PostgreSQL 16+ or Docker for development
- SQLx CLI for database migrations

## 🔧 Development Setup

### Option 1: Using Docker (Recommended)

1. **Start PostgreSQL database**:
   ```bash
   docker compose up -d postgres
   ```

2. **Set environment variables**:
   ```bash
   export DATABASE_URL="postgres://nas_user:nas_password@localhost:5432/simple_nas"
   ```

3. **Run migrations**:
   ```bash
   sqlx migrate run
   ```

4. **Start the application**:
   ```bash
   cargo run
   ```

### Option 2: Using Local PostgreSQL

1. **Install PostgreSQL 16+**

2. **Create database and user**:
   ```sql
   CREATE DATABASE simple_nas;
   CREATE USER nas_user WITH ENCRYPTED PASSWORD 'nas_password';
   GRANT ALL PRIVILEGES ON DATABASE simple_nas TO nas_user;
   ```

3. **Set environment variables**:
   ```bash
   export DATABASE_URL="postgres://nas_user:nas_password@localhost:5432/simple_nas"
   ```

4. **Run migrations**:
   ```bash
   sqlx migrate run
   ```

5. **Start the application**:
   ```bash
   cargo run
   ```

## 🗄️ Database Features

Our PostgreSQL implementation includes:

- **Native UUID Support**: Primary keys using UUID v4
- **JSONB Metadata**: Flexible metadata storage for files and users
- **Full-Text Search**: Advanced search capabilities with ranking
- **Array Support**: Tag storage using PostgreSQL arrays
- **Advanced Indexing**: GIN indexes for JSONB and full-text search
- **Automatic Triggers**: Auto-updating timestamps and search vectors

## 📦 Database Migrations

### Running Migrations

```bash
# Run all pending migrations
sqlx migrate run

# Check migration status
sqlx migrate info

# Revert last migration
sqlx migrate revert
```

### Creating New Migrations

```bash
# Create a new migration file
sqlx migrate add migration_name
```

## 🔍 API Endpoints

### Health Checks
- `GET /` - Basic server information
- `GET /health` - Application health status
- `GET /health/db` - Database connectivity check

### Authentication (Planned)
- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/logout` - User logout
- `POST /api/v1/auth/register` - User registration

### File Management (Planned)
- `GET /api/v1/files` - List files with search and filtering
- `POST /api/v1/files` - Upload new file
- `GET /api/v1/files/:id` - Download file
- `DELETE /api/v1/files/:id` - Delete file

### Sharing (Planned)
- `POST /api/v1/shares` - Create share link
- `GET /api/v1/shares/:hash` - Access shared file

## 🧪 Testing

### Unit Tests
```bash
cargo test
```

### Database Testing

Tests use `testcontainers` to spin up isolated PostgreSQL instances:

```bash
# Run tests with database integration
cargo test --features test-integration
```

## 🐳 Docker Environment

### Services

- **PostgreSQL**: Main database (port 5432)
- **pgAdmin**: Database management UI (port 8080, optional)

### Commands

```bash
# Start all services
docker compose up -d

# Start only PostgreSQL
docker compose up -d postgres

# Start with pgAdmin for database management
docker compose --profile admin up -d

# View logs
docker compose logs postgres

# Stop services
docker compose down

# Clean up volumes (WARNING: deletes data)
docker compose down -v
```

## ⚙️ Configuration

Configuration is handled through environment variables and defaults:

### Database Configuration
- `DATABASE_URL`: PostgreSQL connection string
- `DATABASE_MAX_CONNECTIONS`: Maximum connection pool size (default: 20)
- `DATABASE_MIN_CONNECTIONS`: Minimum connection pool size (default: 5)

### Server Configuration
- `SERVER_HOST`: Server bind address (default: 127.0.0.1)
- `SERVER_PORT`: Server port (default: 3000)

### Security Configuration
- `JWT_SECRET`: JWT signing secret (change in production!)
- `JWT_EXPIRATION_HOURS`: Token expiration time (default: 24)

## 🚀 Production Deployment

### Environment Setup

1. **Set production environment variables**:
   ```bash
   export DATABASE_URL="postgres://user:password@db-host:5432/simple_nas"
   export JWT_SECRET="your-secure-secret-key"
   export RUST_LOG="info"
   ```

2. **Run migrations**:
   ```bash
   sqlx migrate run
   ```

3. **Build and run**:
   ```bash
   cargo build --release
   ./target/release/simple-nas
   ```

### PostgreSQL Production Setup

- Use connection pooling
- Enable SSL/TLS connections
- Configure backup strategies
- Monitor performance and connections
- Set up read replicas for scaling

## 🔐 Security Notes

- Change default JWT secret in production
- Use strong passwords for database users
- Enable SSL/TLS for database connections
- Regularly update dependencies
- Monitor for security advisories

## 📈 Monitoring

### Database Statistics

Access `/health/db` endpoint for database connectivity status.

The application provides built-in database statistics including:
- User count
- File count
- Share count
- Active session count
- Total file storage size

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🗺️ Roadmap

- [x] PostgreSQL migration from SQLite
- [x] Enhanced database schema with JSONB and full-text search
- [ ] Security implementation with JWT authentication
- [ ] File upload and management
- [ ] Secure sharing system
- [ ] Frontend React application
- [ ] Media processing and thumbnails
- [ ] Performance optimizations
