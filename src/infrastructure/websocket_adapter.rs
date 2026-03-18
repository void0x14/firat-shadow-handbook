//! WebSocket Adapter - RFC 6455 compliant implementation
//!
//! Zero external dependencies - uses only std::net

use crate::domain::ports::websocket_port::{WebSocketError, WebSocketPort};
use crate::domain::websocket::{CloseCode, OpCode, WebSocketFrame, WebSocketMessage};
use ring::digest::{Context, SHA1_FOR_LEGACY_USE_ONLY};
use std::io::{Read, Write};

/// Magic string for WebSocket handshake (RFC 6455)
const WS_MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Maximum frame payload size (64MB - DoS protection)
const MAX_PAYLOAD_SIZE: usize = 64 * 1024 * 1024;

/// WebSocket Adapter - implements WebSocketPort trait
pub struct WebSocketAdapter;

impl WebSocketAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Calculate Sec-WebSocket-Accept value
    ///
    /// Algorithm:
    /// 1. Concatenate client key with magic string
    /// 2. SHA-1 hash
    /// 3. Base64 encode
    pub fn calculate_accept(key: &str) -> String {
        let mut context = Context::new(&SHA1_FOR_LEGACY_USE_ONLY);
        context.update(key.as_bytes());
        context.update(WS_MAGIC_STRING.as_bytes());
        let digest = context.finish();
        base64_encode(digest.as_ref())
    }

    /// Parse HTTP upgrade request for WebSocket handshake
    pub fn parse_handshake_request(
        headers: &std::collections::HashMap<String, String>,
        path: &str,
    ) -> Result<crate::domain::websocket::HandshakeRequest, WebSocketError> {
        // Validate Upgrade header
        let upgrade = headers
            .get("Upgrade")
            .ok_or_else(|| WebSocketError::InvalidHandshake("Missing Upgrade header".into()))?;

        if upgrade.to_lowercase() != "websocket" {
            return Err(WebSocketError::InvalidHandshake(
                "Invalid Upgrade header".into(),
            ));
        }

        // Validate Connection header
        let connection = headers
            .get("Connection")
            .ok_or_else(|| WebSocketError::InvalidHandshake("Missing Connection header".into()))?;

        if !connection.to_lowercase().contains("upgrade") {
            return Err(WebSocketError::InvalidHandshake(
                "Invalid Connection header".into(),
            ));
        }

        // Get and validate Sec-WebSocket-Key
        let key = headers
            .get("Sec-WebSocket-Key")
            .ok_or_else(|| WebSocketError::InvalidHandshake("Missing Sec-WebSocket-Key".into()))?;

        // Key must be 24 bytes base64-encoded
        if key.len() != 24 {
            return Err(WebSocketError::InvalidHandshake(
                "Invalid Sec-WebSocket-Key length".into(),
            ));
        }

        // Validate Sec-WebSocket-Version
        let version = headers
            .get("Sec-WebSocket-Version")
            .ok_or_else(|| WebSocketError::InvalidHandshake("Missing Sec-WebSocket-Version".into()))?;

        if version != "13" {
            return Err(WebSocketError::InvalidHandshake(
                "Unsupported WebSocket version".into(),
            ));
        }

        // Optional headers
        let protocols = headers.get("Sec-WebSocket-Protocol").cloned();
        let extensions = headers.get("Sec-WebSocket-Extensions").cloned();

        Ok(crate::domain::websocket::HandshakeRequest {
            path: path.to_string(),
            key: key.clone(),
            version: version.clone(),
            protocols,
            extensions,
        })
    }

    /// Build handshake response headers
    pub fn build_handshake_response(
        request: &crate::domain::websocket::HandshakeRequest,
    ) -> crate::domain::websocket::HandshakeResponse {
        let accept = Self::calculate_accept(&request.key);

        crate::domain::websocket::HandshakeResponse {
            accept,
            protocol: request.protocols.clone(),
            extensions: request.extensions.clone(),
        }
    }

    /// Format handshake response as HTTP headers
    pub fn format_handshake_http_response(
        response: &crate::domain::websocket::HandshakeResponse,
    ) -> String {
        let mut headers = String::from("HTTP/1.1 101 Switching Protocols\r\n");
        headers.push_str("Upgrade: websocket\r\n");
        headers.push_str("Connection: Upgrade\r\n");
        headers.push_str(&format!("Sec-WebSocket-Accept: {}\r\n", response.accept));

        if let Some(protocol) = &response.protocol {
            headers.push_str(&format!("Sec-WebSocket-Protocol: {}\r\n", protocol));
        }

        headers.push_str("\r\n");
        headers
    }

    /// Encode a WebSocket frame to bytes
    pub fn encode_frame(frame: &WebSocketFrame) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Byte 0: FIN, RSV1-3, Opcode
        let mut byte0 = 0u8;
        if frame.fin {
            byte0 |= 0x80;
        }
        if frame.rsv1 {
            byte0 |= 0x40;
        }
        if frame.rsv2 {
            byte0 |= 0x20;
        }
        if frame.rsv3 {
            byte0 |= 0x10;
        }
        byte0 |= frame.opcode as u8;
        buffer.push(byte0);

        // Byte 1: MASK, Payload length
        let payload_len = frame.payload.len();
        let mut byte1 = 0u8;
        if frame.masked {
            byte1 |= 0x80;
        }

        // Extended payload length
        if payload_len <= 125 {
            byte1 |= payload_len as u8;
            buffer.push(byte1);
        } else if payload_len <= 65535 {
            byte1 |= 126;
            buffer.push(byte1);
            buffer.extend_from_slice(&(payload_len as u16).to_be_bytes());
        } else {
            byte1 |= 127;
            buffer.push(byte1);
            buffer.extend_from_slice(&(payload_len as u64).to_be_bytes());
        }

        // Mask key (if masked)
        if let Some(mask_key) = frame.mask_key {
            buffer.extend_from_slice(&mask_key);
        }

        // Payload (apply masking if needed)
        if frame.masked {
            if let Some(mask_key) = frame.mask_key {
                for (i, byte) in frame.payload.iter().enumerate() {
                    buffer.push(byte ^ mask_key[i % 4]);
                }
            }
        } else {
            buffer.extend_from_slice(&frame.payload);
        }

        buffer
    }

    /// Decode a WebSocket frame from bytes
    pub fn decode_frame(buffer: &[u8]) -> Result<(WebSocketFrame, usize), WebSocketError> {
        if buffer.is_empty() {
            return Err(WebSocketError::InvalidFrame("Empty buffer".into()));
        }

        let mut offset = 0;

        // Byte 0: FIN, RSV1-3, Opcode
        let byte0 = buffer[offset];
        let fin = (byte0 & 0x80) != 0;
        let rsv1 = (byte0 & 0x40) != 0;
        let rsv2 = (byte0 & 0x20) != 0;
        let rsv3 = (byte0 & 0x10) != 0;
        let opcode = OpCode::from_u8(byte0 & 0x0F)
            .ok_or_else(|| WebSocketError::InvalidFrame("Invalid opcode".into()))?;

        offset += 1;

        if offset >= buffer.len() {
            return Err(WebSocketError::InvalidFrame("Incomplete frame".into()));
        }

        // Byte 1: MASK, Payload length
        let byte1 = buffer[offset];
        let masked = (byte1 & 0x80) != 0;
        let mut payload_len = (byte1 & 0x7F) as usize;

        offset += 1;

        // Extended payload length
        if payload_len == 126 {
            if offset + 2 > buffer.len() {
                return Err(WebSocketError::InvalidFrame("Incomplete frame length".into()));
            }
            payload_len = u16::from_be_bytes([buffer[offset], buffer[offset + 1]]) as usize;
            offset += 2;
        } else if payload_len == 127 {
            if offset + 8 > buffer.len() {
                return Err(WebSocketError::InvalidFrame("Incomplete frame length".into()));
            }
            payload_len = u64::from_be_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
                buffer[offset + 4],
                buffer[offset + 5],
                buffer[offset + 6],
                buffer[offset + 7],
            ]) as usize;
            offset += 8;
        }

        // Validate payload size (DoS protection)
        if payload_len > MAX_PAYLOAD_SIZE {
            return Err(WebSocketError::MessageTooLarge(payload_len));
        }

        // Mask key (if masked)
        let mut mask_key = None;
        if masked {
            if offset + 4 > buffer.len() {
                return Err(WebSocketError::InvalidFrame("Incomplete mask key".into()));
            }
            let key = [
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ];
            mask_key = Some(key);
            offset += 4;
        }

        // Payload
        if offset + payload_len > buffer.len() {
            return Err(WebSocketError::InvalidFrame("Incomplete payload".into()));
        }

        let mut payload = buffer[offset..offset + payload_len].to_vec();

        // Unmask payload if masked (client -> server messages are always masked)
        if masked {
            if let Some(key) = mask_key {
                for (i, byte) in payload.iter_mut().enumerate() {
                    *byte ^= key[i % 4];
                }
            }
        }

        offset += payload_len;

        let frame = WebSocketFrame {
            fin,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            masked,
            mask_key,
            payload,
        };

        Ok((frame, offset))
    }

    /// Parse WebSocket message from frame
    pub fn frame_to_message(frame: WebSocketFrame) -> Result<WebSocketMessage, WebSocketError> {
        match frame.opcode {
            OpCode::Text => {
                let text = String::from_utf8(frame.payload)?;
                Ok(WebSocketMessage::Text(text))
            }
            OpCode::Binary => Ok(WebSocketMessage::Binary(frame.payload)),
            OpCode::Close => {
                if frame.payload.is_empty() {
                    Ok(WebSocketMessage::Close(CloseCode::Normal, String::new()))
                } else if frame.payload.len() >= 2 {
                    let code = u16::from_be_bytes([frame.payload[0], frame.payload[1]]);
                    let reason = String::from_utf8_lossy(&frame.payload[2..]).to_string();
                    Ok(WebSocketMessage::Close(CloseCode::from_u16(code), reason))
                } else {
                    Err(WebSocketError::InvalidFrame(
                        "Invalid close frame payload".into(),
                    ))
                }
            }
            OpCode::Ping => Ok(WebSocketMessage::Ping(frame.payload)),
            OpCode::Pong => Ok(WebSocketMessage::Pong(frame.payload)),
            OpCode::Continuation => Err(WebSocketError::InvalidFrame(
                "Continuation frames not supported".into(),
            )),
        }
    }

    /// Convert message to frame
    fn message_to_frame(message: WebSocketMessage) -> WebSocketFrame {
        match message {
            WebSocketMessage::Text(text) => WebSocketFrame::text(&text),
            WebSocketMessage::Binary(data) => WebSocketFrame::binary(&data),
            WebSocketMessage::Close(code, reason) => WebSocketFrame::close(code, &reason),
            WebSocketMessage::Ping(data) => WebSocketFrame::ping(&data),
            WebSocketMessage::Pong(data) => WebSocketFrame::pong(&data),
        }
    }
}

impl WebSocketPort for WebSocketAdapter {
    type Stream = std::net::TcpStream;

    fn send(&self, stream: &mut Self::Stream, message: WebSocketMessage) -> Result<(), WebSocketError> {
        let frame = Self::message_to_frame(message);
        let buffer = Self::encode_frame(&frame);
        stream.write_all(&buffer)?;
        stream.flush()?;
        Ok(())
    }

    fn receive(&self, stream: &mut Self::Stream) -> Result<WebSocketMessage, WebSocketError> {
        let mut buffer = Vec::with_capacity(4096);
        let mut temp = [0u8; 1024];

        // Read until we have a complete frame
        loop {
            let n = stream.read(&mut temp)?;
            if n == 0 {
                return Err(WebSocketError::ConnectionClosed("Connection closed".into()));
            }
            buffer.extend_from_slice(&temp[..n]);

            // Try to decode frame
            match Self::decode_frame(&buffer) {
                Ok((frame, _)) => return Self::frame_to_message(frame),
                Err(WebSocketError::InvalidFrame(_)) if buffer.len() < MAX_PAYLOAD_SIZE => {
                    // Need more data, continue reading
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn close(&self, stream: &mut Self::Stream, code: u16, reason: &str) -> Result<(), WebSocketError> {
        let close_frame = WebSocketFrame::close(CloseCode::from_u16(code), reason);
        let buffer = Self::encode_frame(&close_frame);
        stream.write_all(&buffer)?;
        stream.flush()?;
        
        // Read close response
        let mut response_buffer = Vec::with_capacity(4096);
        let mut temp = [0u8; 1024];
        stream.set_read_timeout(Some(std::time::Duration::from_secs(1))).ok();
        
        loop {
            match stream.read(&mut temp) {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    response_buffer.extend_from_slice(&temp[..n]);
                    if Self::decode_frame(&response_buffer).is_ok() {
                        break;
                    }
                }
                Err(_) => break, // Timeout or error
            }
        }

        Ok(())
    }

    fn ping(&self, stream: &mut Self::Stream, data: &[u8]) -> Result<(), WebSocketError> {
        let ping_frame = WebSocketFrame::ping(data);
        let buffer = Self::encode_frame(&ping_frame);
        stream.write_all(&buffer)?;
        stream.flush()?;
        Ok(())
    }

    fn pong(&self, stream: &mut Self::Stream, data: &[u8]) -> Result<(), WebSocketError> {
        let pong_frame = WebSocketFrame::pong(data);
        let buffer = Self::encode_frame(&pong_frame);
        stream.write_all(&buffer)?;
        stream.flush()?;
        Ok(())
    }
}

impl Default for WebSocketAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Base64 Encoding (Zero Dependency Implementation)
// ============================================================================

/// Base64 encode bytes to string
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b0 = data[i] as usize;
        let b1 = if i + 1 < data.len() { data[i + 1] as usize } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as usize } else { 0 };

        result.push(ALPHABET[(b0 >> 2) & 0x3F] as char);
        result.push(ALPHABET[((b0 << 4) | (b1 >> 4)) & 0x3F] as char);

        if i + 1 < data.len() {
            result.push(ALPHABET[((b1 << 2) | (b2 >> 6)) & 0x3F] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(ALPHABET[b2 & 0x3F] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::websocket_port::ConnectionTracker;
    use crate::domain::websocket::ConnectionState;

    #[test]
    fn test_calculate_accept() {
        // RFC 6455 example
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept = WebSocketAdapter::calculate_accept(key);
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn test_encode_decode_text_frame() {
        let frame = WebSocketFrame::text("Hello, WebSocket!");
        let encoded = WebSocketAdapter::encode_frame(&frame);
        let (decoded, _) = WebSocketAdapter::decode_frame(&encoded).unwrap();
        
        assert_eq!(decoded.opcode, OpCode::Text);
        assert_eq!(decoded.fin, true);
        assert_eq!(decoded.masked, false);
        assert_eq!(decoded.payload, b"Hello, WebSocket!");
    }

    #[test]
    fn test_encode_decode_binary_frame() {
        let data = vec![0x00, 0x01, 0x02, 0x03, 0xFF];
        let frame = WebSocketFrame::binary(&data);
        let encoded = WebSocketAdapter::encode_frame(&frame);
        let (decoded, _) = WebSocketAdapter::decode_frame(&encoded).unwrap();
        
        assert_eq!(decoded.opcode, OpCode::Binary);
        assert_eq!(decoded.payload, data);
    }

    #[test]
    fn test_encode_decode_ping_frame() {
        let data = vec![0x42, 0x43];
        let frame = WebSocketFrame::ping(&data);
        let encoded = WebSocketAdapter::encode_frame(&frame);
        let (decoded, _) = WebSocketAdapter::decode_frame(&encoded).unwrap();
        
        assert_eq!(decoded.opcode, OpCode::Ping);
        assert_eq!(decoded.payload, data);
    }

    #[test]
    fn test_encode_decode_pong_frame() {
        let data = vec![0x42, 0x43];
        let frame = WebSocketFrame::pong(&data);
        let encoded = WebSocketAdapter::encode_frame(&frame);
        let (decoded, _) = WebSocketAdapter::decode_frame(&encoded).unwrap();
        
        assert_eq!(decoded.opcode, OpCode::Pong);
        assert_eq!(decoded.payload, data);
    }

    #[test]
    fn test_encode_decode_close_frame() {
        let frame = WebSocketFrame::close(CloseCode::Normal, "Goodbye");
        let encoded = WebSocketAdapter::encode_frame(&frame);
        let (decoded, _) = WebSocketAdapter::decode_frame(&encoded).unwrap();
        
        assert_eq!(decoded.opcode, OpCode::Close);
        // First 2 bytes are close code
        let code = u16::from_be_bytes([decoded.payload[0], decoded.payload[1]]);
        assert_eq!(code, 1000);
        // Rest is reason
        assert_eq!(&decoded.payload[2..], b"Goodbye");
    }

    #[test]
    fn test_masking_unmasking() {
        let payload = b"Hello";
        let mask_key = [0x12, 0x34, 0x56, 0x78];
        
        // Manual masking
        let mut masked = Vec::new();
        for (i, byte) in payload.iter().enumerate() {
            masked.push(byte ^ mask_key[i % 4]);
        }
        
        // Manual unmasking
        let mut unmasked = Vec::new();
        for (i, byte) in masked.iter().enumerate() {
            unmasked.push(byte ^ mask_key[i % 4]);
        }
        
        assert_eq!(unmasked, payload.to_vec());
    }

    #[test]
    fn test_op_code_is_control() {
        assert!(!OpCode::Text.is_control());
        assert!(!OpCode::Binary.is_control());
        assert!(OpCode::Close.is_control());
        assert!(OpCode::Ping.is_control());
        assert!(OpCode::Pong.is_control());
    }

    #[test]
    fn test_close_code_from_u16() {
        assert_eq!(CloseCode::from_u16(1000), CloseCode::Normal);
        assert_eq!(CloseCode::from_u16(1001), CloseCode::GoingAway);
        assert_eq!(CloseCode::from_u16(1002), CloseCode::ProtocolError);
        assert_eq!(CloseCode::from_u16(9999), CloseCode::ProtocolError);
    }

    #[test]
    fn test_connection_state_transitions() {
        let mut tracker = ConnectionTracker::new("127.0.0.1:8080".into(), "/ws".into());
        assert_eq!(tracker.state, ConnectionState::Connecting);
        
        tracker.transition_to(ConnectionState::Open);
        assert_eq!(tracker.state, ConnectionState::Open);
        assert!(tracker.is_open());
        
        tracker.transition_to(ConnectionState::Closing);
        assert_eq!(tracker.state, ConnectionState::Closing);
        
        tracker.transition_to(ConnectionState::Closed);
        assert_eq!(tracker.state, ConnectionState::Closed);
        assert!(tracker.is_closed());
    }
}
