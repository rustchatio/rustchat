//! Real-time WebSocket module for rustchat
//!
//! Provides WebSocket hub for presence, typing indicators, and event fan-out.

pub mod connection_store;
pub mod events;
pub mod hub;
pub mod websocket_actor;

pub use connection_store::*;
pub use events::*;
pub use hub::*;
pub use websocket_actor::*;
