//! HTTP middleware for rustchat
//!
//! Provides cross-cutting concerns like rate limiting, authentication,
//! security headers, and reliability patterns.

pub mod rate_limit;
pub mod reliability;
pub mod security_headers;
pub mod terms_enforcement;

pub use rate_limit::*;
