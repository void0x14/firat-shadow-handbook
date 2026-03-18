# Firat Shadow Handbook

**WARNING: THIS PROJECT IS ARCHIVED AND NO LONGER UNDER ACTIVE DEVELOPMENT.**
**Status: Final / Archived (March 18, 2026)**

Firat Shadow Handbook is a high-performance, mid-level systems project designed as a bridge between Moodle/Collab systems and a local shadow environment. The project adheres to a "Pure Metal" philosophy, minimizing external dependencies and implementing core protocols from scratch.

## Technical Architecture

The project is built using a "Zero-Dependency" / "Pure Metal" approach in Rust. It implements critical infrastructure components manually to ensure maximum control and zero bloat.

### Core Features (Implemented)

- [x] **Pure Metal HTTP Server**: Built on `std::net::TcpListener` with a custom-built Thread Pool and manual HTTP/1.1 request/response parsing.
- [x] **Zero-Dependency Cryptography**: Manual implementations of SHA-1, SHA-256, and HMAC-SHA256 (see `src/crypto.rs`). No external crypto crates used for core logic.
- [x] **WebSocket Protocol (RFC 6455)**: A from-scratch implementation of the WebSocket protocol, including handshake, frame encoding/decoding, and control frames (Ping/Pong/Close).
- [x] **Hexagonal Architecture**: Strict separation of concerns using Domain, Application, and Infrastructure layers. Ports and Adapters pattern is enforced.
- [x] **Server-Side Session Management**: custom `ShadowSession` implementation with HMAC signatures for integrity. Sessions are maintained server-side and persisted to JSON state.
- [x] **Security Engine**:
    - [x] IP-based Rate Limiting.
    - [x] Strict Security Headers (CSP, XSS Protection, HSTS, No-Sniff).
    - [x] Input Sanitization and validation.
- [x] **Moodle/Collab Integration**:
    - [x] CAS (Central Authentication Service) Adapter for login flow.
    - [x] Custom HTML Scraper for course and playback metadata (zero external HTML parser).
- [x] **Vanilla Frontend**: A high-performance, buildless frontend using Vanilla JS with JSDoc for type safety and CSS Variables for theme management.

### Roadmap (Planned / Cancelled)

- [ ] **Automated Recording Downloader**: Integration with ffmpeg for direct playback capture.
- [ ] **Real-time Synchronization**: Pushing debsis update notifications via WebSockets.
- [ ] **Advanced Persistence**: Transition from JSON-file state to a proper SQL-based storage (PostgreSQL/SQLite).
- [ ] **Edge Deployment**: Optimization for lightweight ARM-based home servers.

## Development Status

This repository is maintained in its current state as a technical demonstration of low-level systems programming in Rust. No further features will be added.

### Environment Setup

- **Language**: Rust 1.75+
- **Configuration**: Managed via `Config` struct (see `src/application/composition.rs`).
- **Data Directory**: `data/runtime/` (Requires write permissions for session persistence).

## Licensing

This project is licensed under the MIT License. See the `LICENSE` file for details.

## Installation & Execution

### Prerequisites
- **Rust Toolchain**: 1.75 or newer.

### Running the Server
```bash
# From the project root
cargo run
```

### Running Tests
```bash
cargo test
```

---

*Note: This project is for educational and technical review purposes only. It is not officially affiliated with any institution.*
