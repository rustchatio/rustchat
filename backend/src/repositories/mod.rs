//! Repository module for database query patterns
//!
//! Centralizes common SQL queries to reduce duplication across the codebase.

pub mod post_repository;

pub use post_repository::{PostRepository, PostWithUser};
