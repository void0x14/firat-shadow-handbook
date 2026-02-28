//! Fırat Shadow Handbook - Core Library
//!
//! This library provides the core HTTP server functionality
//! for the Fırat Shadow Handbook application.

pub mod http;
pub mod handler;
pub mod config;
pub mod domain;
pub mod application;
pub mod infrastructure;

// Re-export commonly used types
pub use http::{Request, Response, Method};
pub use handler::Router;
pub use config::Config;