//! rustchat - Self-hosted team collaboration platform
//!
//! This crate provides the core functionality for rustchat,
//! a messaging platform built in Rust.

pub mod a2a;
pub mod api;
pub mod auth;
pub mod config;
pub mod crypto;
pub mod db;
pub mod error;
pub mod jobs;
pub mod mattermost_compat;
pub mod mcp;
pub mod middleware;
pub mod models;
pub mod realtime;
pub mod search;
pub mod services;
pub mod storage;
pub mod telemetry;
