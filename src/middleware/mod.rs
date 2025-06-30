// Middleware modules - will be implemented in Task 1.3 (Security Infrastructure)
// pub mod auth;         // Authentication middleware
// pub mod rate_limit;   // Rate limiting middleware
// pub mod cors;         // CORS middleware (basic version in main.rs)

// Middleware modules for the Simple NAS application
pub mod auth;

pub use auth::*;
