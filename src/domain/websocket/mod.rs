//! WebSocket domain types
//!
//! RFC 6455 compliant WebSocket types for the domain layer

/// WebSocket opcode values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl OpCode {
    /// Check if opcode is a control frame
    pub fn is_control(self) -> bool {
        matches!(self, OpCode::Close | OpCode::Ping | OpCode::Pong)
    }

    /// Create OpCode from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(OpCode::Continuation),
            0x1 => Some(OpCode::Text),
            0x2 => Some(OpCode::Binary),
            0x8 => Some(OpCode::Close),
            0x9 => Some(OpCode::Ping),
            0xA => Some(OpCode::Pong),
            _ => None,
        }
    }
}

/// WebSocket connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Open,
    Closing,
    Closed,
}

/// WebSocket close codes (RFC 6455)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CloseCode {
    Normal = 1000,
    GoingAway = 1001,
    ProtocolError = 1002,
    UnsupportedData = 1003,
    InvalidFramePayloadData = 1007,
    PolicyViolation = 1008,
    MessageTooBig = 1009,
    MandatoryExtension = 1010,
    InternalServerError = 1011,
    ServiceRestart = 1012,
    TryAgainLater = 1013,
    TlsHandshakeFailure = 1015,
}

impl CloseCode {
    /// Create CloseCode from u16
    pub fn from_u16(value: u16) -> Self {
        match value {
            1000 => CloseCode::Normal,
            1001 => CloseCode::GoingAway,
            1002 => CloseCode::ProtocolError,
            1003 => CloseCode::UnsupportedData,
            1007 => CloseCode::InvalidFramePayloadData,
            1008 => CloseCode::PolicyViolation,
            1009 => CloseCode::MessageTooBig,
            1010 => CloseCode::MandatoryExtension,
            1011 => CloseCode::InternalServerError,
            1012 => CloseCode::ServiceRestart,
            1013 => CloseCode::TryAgainLater,
            1015 => CloseCode::TlsHandshakeFailure,
            _ => CloseCode::ProtocolError,
        }
    }

    /// Convert to u16
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// WebSocket message type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Close(CloseCode, String),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

/// WebSocket frame structure
#[derive(Debug, Clone)]
pub struct WebSocketFrame {
    pub fin: bool,
    pub rsv1: bool,
    pub rsv2: bool,
    pub rsv3: bool,
    pub opcode: OpCode,
    pub masked: bool,
    pub mask_key: Option<[u8; 4]>,
    pub payload: Vec<u8>,
}

impl WebSocketFrame {
    /// Create a new text frame
    pub fn text(data: &str) -> Self {
        Self {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: OpCode::Text,
            masked: false,
            mask_key: None,
            payload: data.as_bytes().to_vec(),
        }
    }

    /// Create a new binary frame
    pub fn binary(data: &[u8]) -> Self {
        Self {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: OpCode::Binary,
            masked: false,
            mask_key: None,
            payload: data.to_vec(),
        }
    }

    /// Create a pong frame
    pub fn pong(data: &[u8]) -> Self {
        Self {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: OpCode::Pong,
            masked: false,
            mask_key: None,
            payload: data.to_vec(),
        }
    }

    /// Create a close frame
    pub fn close(code: CloseCode, reason: &str) -> Self {
        let mut payload = Vec::with_capacity(2 + reason.len());
        payload.extend_from_slice(&code.as_u16().to_be_bytes());
        payload.extend_from_slice(reason.as_bytes());

        Self {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: OpCode::Close,
            masked: false,
            mask_key: None,
            payload,
        }
    }

    /// Create a ping frame
    pub fn ping(data: &[u8]) -> Self {
        Self {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: OpCode::Ping,
            masked: false,
            mask_key: None,
            payload: data.to_vec(),
        }
    }
}

/// WebSocket handshake request
#[derive(Debug, Clone)]
pub struct HandshakeRequest {
    pub path: String,
    pub key: String,
    pub version: String,
    pub protocols: Option<String>,
    pub extensions: Option<String>,
}

/// WebSocket handshake response
#[derive(Debug, Clone)]
pub struct HandshakeResponse {
    pub accept: String,
    pub protocol: Option<String>,
    pub extensions: Option<String>,
}
