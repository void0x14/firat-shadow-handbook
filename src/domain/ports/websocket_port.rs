//! WebSocket Port - Domain boundary for WebSocket operations

use crate::domain::websocket::ConnectionState;
use crate::domain::websocket::WebSocketMessage;
use std::io::{Read, Write};

/// WebSocket Port trait - defines the contract for WebSocket operations
pub trait WebSocketPort {
    type Stream: Read + Write;

    /// Send a WebSocket message
    fn send(
        &self,
        stream: &mut Self::Stream,
        message: WebSocketMessage,
    ) -> Result<(), WebSocketError>;

    /// Receive a WebSocket message
    fn receive(&self, stream: &mut Self::Stream) -> Result<WebSocketMessage, WebSocketError>;

    /// Close the WebSocket connection
    fn close(
        &self,
        stream: &mut Self::Stream,
        code: u16,
        reason: &str,
    ) -> Result<(), WebSocketError>;

    /// Send a ping frame
    fn ping(&self, stream: &mut Self::Stream, data: &[u8]) -> Result<(), WebSocketError>;

    /// Send a pong frame
    fn pong(&self, stream: &mut Self::Stream, data: &[u8]) -> Result<(), WebSocketError>;
}

// Blanket implementation for Box<dyn WebSocketPort>
// This enables runtime polymorphic adapter selection via CompositionRoot
impl<T: WebSocketPort + ?Sized> WebSocketPort for Box<T> {
    type Stream = T::Stream;

    fn send(
        &self,
        stream: &mut Self::Stream,
        message: WebSocketMessage,
    ) -> Result<(), WebSocketError> {
        (**self).send(stream, message)
    }

    fn receive(&self, stream: &mut Self::Stream) -> Result<WebSocketMessage, WebSocketError> {
        (**self).receive(stream)
    }

    fn close(
        &self,
        stream: &mut Self::Stream,
        code: u16,
        reason: &str,
    ) -> Result<(), WebSocketError> {
        (**self).close(stream, code, reason)
    }

    fn ping(&self, stream: &mut Self::Stream, data: &[u8]) -> Result<(), WebSocketError> {
        (**self).ping(stream, data)
    }

    fn pong(&self, stream: &mut Self::Stream, data: &[u8]) -> Result<(), WebSocketError> {
        (**self).pong(stream, data)
    }
}

/// WebSocket error types
#[derive(Debug)]
pub enum WebSocketError {
    Io(std::io::Error),
    InvalidHandshake(String),
    InvalidFrame(String),
    ConnectionClosed(String),
    ProtocolError(String),
}

impl std::fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::InvalidHandshake(msg) => write!(f, "Invalid handshake: {}", msg),
            Self::InvalidFrame(msg) => write!(f, "Invalid frame: {}", msg),
            Self::ConnectionClosed(msg) => write!(f, "Connection closed: {}", msg),
            Self::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

impl std::error::Error for WebSocketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for WebSocketError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for WebSocketError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::InvalidFrame(format!("UTF-8 error: {}", err)) // Folding Utf8Error into InvalidFrame
    }
}

/// Connection state tracker
#[derive(Debug, Clone)]
pub struct ConnectionTracker {
    pub state: ConnectionState,
    pub remote_addr: String,
    pub path: String,
}

impl ConnectionTracker {
    pub fn new(remote_addr: String, path: String) -> Self {
        Self {
            state: ConnectionState::Connecting,
            remote_addr,
            path,
        }
    }

    pub fn transition_to(&mut self, state: ConnectionState) {
        self.state = state;
    }

    pub fn is_open(&self) -> bool {
        self.state == ConnectionState::Open
    }

    pub fn is_closed(&self) -> bool {
        self.state == ConnectionState::Closed
    }
}
