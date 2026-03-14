//! Agent-to-Agent (A2A) Protocol Module
//!
//! Provides secure communication between AI agents in the RustChat platform.
//! Implements Constitution XIII: Zero-Trust extensibility for agent collaboration.
//!
//! # Architecture
//!
//! - **Protocol**: Core message types and definitions (AgentId, A2AMessage, etc.)
//! - **Registry**: Agent discovery and capability advertisement
//! - **Bus**: Redis-backed pub/sub for message distribution
//! - **Tasks**: Task lifecycle management (create, update, complete)
//! - **Security**: Message signing, authentication, and audit logging

pub mod bus;
pub mod protocol;
pub mod registry;
pub mod security;
pub mod tasks;

pub use bus::A2AMessageBus;
pub use protocol::{
    AgentAdvertisement, AgentCapability, AgentId, A2AMessage, TaskRequest, TaskResponse,
    TaskStatus, TaskStatusUpdate,
};
pub use registry::AgentRegistry;
pub use security::A2ASecurityContext;
pub use tasks::TaskManager;
