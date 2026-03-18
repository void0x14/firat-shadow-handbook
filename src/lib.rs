//! Fırat Shadow Handbook - Core Library
//!
//! This library provides the core HTTP server functionality
//! for the Fırat Shadow Handbook application.

pub mod application;
pub mod config;
pub mod crypto;
pub mod domain;
pub mod handler;
pub mod http;
pub mod infrastructure;

// Re-export commonly used types
pub use config::Config;
pub use handler::Router;
pub use http::{Method, Request, Response};
