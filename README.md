# Firat Shadow Handbook

**WARNING: THIS PROJECT IS ARCHIVED AND NO LONGER UNDER ACTIVE DEVELOPMENT.**
**Status: Final / Archived (March 18, 2026)**

## Purpose & Scope
Firat Shadow Handbook serves as a performant bridge between legacy Moodle/Collab educational systems and modern local shadow environments. It provides high-efficiency data extraction and secure session bridging for users in restricted network environments.

## Technical Invariants
The system is built upon a set of immutable technical principles (Invariants) that must always hold:
1. **Dependency Invariant**: Core logic (HTTP, WebSocket, Cryptography, JSON) must remain dependency-free, relying exclusively on the Rust Standard Library (`std`).
2. **Archtectural Invariant**: The project adheres strictly to **Hexagonal Architecture**. Domain logic is isolated from infrastructure; infrastructure depends on domain ports, never the reverse.
3. **Runtime Invariant**: No asynchronous runtimes (e.g., Tokio, async-std) are permitted. Execution is managed via a custom Synchronous Multi-Threaded Worker Pool.
4. **Security Invariant**: All sessions must be server-side HMAC-signed. No sensitive plain-text data is persisted without cryptographic verification.

### Core Implementation Details
- **Pure Metal HTTP Server**: Manual RFC 2616 implementation over `std::net::TcpListener`. Includes a custom thread-pool with `mpsc` channel-driven workload distribution.
- **Zero-Dependency Cryptography**: Hardware-unlocked implementations of SHA-1, SHA-256, and HMAC-SHA256 implemented in `src/crypto.rs`.
- **WebSocket Protocol (RFC 6455)**: Low-level frame processing including masking, fragmented-frame handling, and control frame sequencing (Ping/Pong/Close).
- **Session Layer**: Server-mode session bridging using `ShadowSession` state, persisted as authenticated JSON blobs in `data/runtime/`.
- **Moodle/Collab Scraper**: Advanced pattern-matching scraper optimized for the 2025/2026 Moodle AJAX/HTML structure.

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
