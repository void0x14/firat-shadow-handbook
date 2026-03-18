//! WebSocket Message Use Case - Application layer for handling WebSocket messages

use crate::domain::ports::websocket_port::{WebSocketError, WebSocketPort};
use crate::domain::websocket::WebSocketMessage;

/// WebSocket Message Handler - processes incoming WebSocket messages
pub struct WsMessageUsecase<P: WebSocketPort> {
    websocket_port: P,
}

impl<P: WebSocketPort> WsMessageUsecase<P> {
    /// Create a new WebSocket message use case
    pub fn new(websocket_port: P) -> Self {
        Self { websocket_port }
    }

    /// Receive a message from the WebSocket
    pub fn receive_message(
        &self,
        stream: &mut <P as WebSocketPort>::Stream,
    ) -> Result<WebSocketMessage, WebSocketError> {
        self.websocket_port.receive(stream)
    }

    /// Close the WebSocket connection
    pub fn close_connection(
        &self,
        stream: &mut <P as WebSocketPort>::Stream,
        code: u16,
        reason: &str,
    ) -> Result<(), WebSocketError> {
        self.websocket_port.close(stream, code, reason)
    }

    /// Handle an incoming WebSocket message
    ///
    /// This is where application-specific message routing logic would go.
    /// For now, it's a simple echo handler as a placeholder.
    pub fn handle_message(
        &self,
        stream: &mut <P as WebSocketPort>::Stream,
        message: WebSocketMessage,
    ) -> Result<(), WebSocketError> {
        match message {
            // Echo text messages back
            WebSocketMessage::Text(text) => {
                println!("[WS] Received text: {}", text);
                // Application logic would go here
                // For now, echo back
                self.websocket_port.send(
                    stream,
                    WebSocketMessage::Text(format!("Echo: {}", text)),
                )?;
            }
            // Echo binary messages back
            WebSocketMessage::Binary(data) => {
                println!("[WS] Received binary: {} bytes", data.len());
                // Application logic would go here
                self.websocket_port.send(stream, WebSocketMessage::Binary(data))?;
            }
            // Handle ping with pong
            WebSocketMessage::Ping(data) => {
                println!("[WS] Received ping");
                self.websocket_port.pong(stream, &data)?;
            }
            // Pong is already a response, just log
            WebSocketMessage::Pong(data) => {
                println!("[WS] Received pong: {} bytes", data.len());
            }
            // Close is handled by the connection manager
            WebSocketMessage::Close(code, reason) => {
                println!("[WS] Received close: {} - {}", code.as_u16(), reason);
            }
        }

        Ok(())
    }

    /// Send a message through the WebSocket
    pub fn send_message(
        &self,
        stream: &mut <P as WebSocketPort>::Stream,
        message: WebSocketMessage,
    ) -> Result<(), WebSocketError> {
        self.websocket_port.send(stream, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::websocket_port::WebSocketPort;
    use crate::domain::websocket::WebSocketFrame;
    use std::io::{Read, Write};

    // Mock stream for testing
    struct MockStream {
        buffer: Vec<u8>,
    }

    impl MockStream {
        fn new() -> Self {
            Self { buffer: Vec::new() }
        }

        fn get_messages(&self) -> Vec<WebSocketMessage> {
            let mut messages = Vec::new();
            let mut offset = 0;

            while offset < self.buffer.len() {
                if let Ok((frame, len)) =
                    crate::infrastructure::websocket_adapter::WebSocketAdapter::decode_frame(
                        &self.buffer[offset..],
                    )
                {
                    if let Ok(msg) =
                        crate::infrastructure::websocket_adapter::WebSocketAdapter::frame_to_message(
                            frame,
                        )
                    {
                        messages.push(msg);
                    }
                    offset += len;
                } else {
                    break;
                }
            }

            messages
        }
    }

    impl Read for MockStream {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            // For testing, return 0 (no data to read)
            Ok(0)
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.buffer.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    // Mock WebSocketPort for testing
    struct MockWebSocketPort;

    impl WebSocketPort for MockWebSocketPort {
        type Stream = MockStream;

        fn send(
            &self,
            stream: &mut Self::Stream,
            message: WebSocketMessage,
        ) -> Result<(), WebSocketError> {
            // Encode the message as a frame
            let msg_frame = match message {
                WebSocketMessage::Text(text) => WebSocketFrame::text(&text),
                WebSocketMessage::Binary(data) => WebSocketFrame::binary(&data),
                WebSocketMessage::Close(code, reason) => {
                    WebSocketFrame::close(code, &reason)
                }
                WebSocketMessage::Ping(data) => WebSocketFrame::ping(&data),
                WebSocketMessage::Pong(data) => WebSocketFrame::pong(&data),
            };
            
            let buffer = crate::infrastructure::websocket_adapter::WebSocketAdapter::encode_frame(
                &msg_frame,
            );
            stream.write_all(&buffer)?;
            Ok(())
        }

        fn receive(
            &self,
            _stream: &mut Self::Stream,
        ) -> Result<WebSocketMessage, WebSocketError> {
            Err(WebSocketError::ConnectionClosed("Mock".into()))
        }

        fn close(
            &self,
            _stream: &mut Self::Stream,
            _code: u16,
            _reason: &str,
        ) -> Result<(), WebSocketError> {
            Ok(())
        }

        fn ping(&self, _stream: &mut Self::Stream, _data: &[u8]) -> Result<(), WebSocketError> {
            Ok(())
        }

        fn pong(&self, _stream: &mut Self::Stream, _data: &[u8]) -> Result<(), WebSocketError> {
            Ok(())
        }
    }

    #[test]
    fn test_usecase_echo_text() {
        let port = MockWebSocketPort;
        let usecase = WsMessageUsecase::new(port);
        let mut stream = MockStream::new();

        let result = usecase.handle_message(&mut stream, WebSocketMessage::Text("Hello".into()));
        assert!(result.is_ok());

        let messages = stream.get_messages();
        assert!(!messages.is_empty());
        
        if let WebSocketMessage::Text(text) = &messages[0] {
            assert!(text.starts_with("Echo: "));
        } else {
            panic!("Expected text message");
        }
    }

    #[test]
    fn test_usecase_echo_binary() {
        let port = MockWebSocketPort;
        let usecase = WsMessageUsecase::new(port);
        let mut stream = MockStream::new();

        let data = vec![0x01, 0x02, 0x03];
        let result = usecase.handle_message(&mut stream, WebSocketMessage::Binary(data.clone()));
        assert!(result.is_ok());

        let messages = stream.get_messages();
        assert!(!messages.is_empty());
        
        if let WebSocketMessage::Binary(received) = &messages[0] {
            assert_eq!(received, &data);
        } else {
            panic!("Expected binary message");
        }
    }

    #[test]
    fn test_usecase_ping_pong() {
        let port = MockWebSocketPort;
        let usecase = WsMessageUsecase::new(port);
        let mut stream = MockStream::new();

        let ping_data = vec![0x42];
        let result = usecase.handle_message(&mut stream, WebSocketMessage::Ping(ping_data));
        assert!(result.is_ok());
        // Pong should be sent automatically
    }
}
