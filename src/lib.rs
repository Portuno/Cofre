// Cofre Vault Platform - Core Library
// Provides authentication, vault management, content storage, and semantic graph operations

pub mod error;
pub mod models;
pub mod db;
pub mod services;

pub use error::{Error, Result};
pub use models::*;
pub use db::Database;
pub use services::AuthService;

// Re-export commonly used types
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};
