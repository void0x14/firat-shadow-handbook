---
story_id: 1-1
title: Rust HTTP Server Foundation
epic: Core Skeleton
status: ready-for-dev
created: 2026-02-24
---

# Story 1.1: Rust HTTP Server Foundation

## Goal
std::net::TcpListener ile zero-dependency HTTP/1.1 sunucusu kurmak.

## Acceptance Criteria
- [ ] Rust HTTP server port 8080'de çalışıyor
- [ ] GET requests respond ediyor
- [ ] Static files (HTML, CSS, JS) serve ediliyor
- [ ] Multi-threaded request processing

## Technical Specs

### File Structure
```
src/
├── main.rs          # Entry point, server init
├── lib.rs            # Core library
├── handlers/
│   ├── mod.rs
│   └── http.rs       # HTTP request handling
└── static.rs         # Static file serving
```

### Implementation Notes
1. TcpListener::bind("127.0.0.1:8080")
2. HTTP/1.1 request parsing (method, path, headers)
3. Response builder (status, headers, body)
4. File system routing for /web/* assets

## Dependencies
- None (std::net only)

## Estimated Effort
Medium (4-6 hours)
