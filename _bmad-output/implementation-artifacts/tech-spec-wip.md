---
title: 'Fırat Shadow Handbook - Core Skeleton Implementation'
slug: 'firat-shadow-core-skeleton'
created: '2026-02-24T20:50:00.000Z'
status: 'in-progress'
stepsCompleted: [1]
tech_stack: ['Rust (std::net)', 'Vanilla JS (ESM + JSDoc)', 'Native CSS', 'Filesystem Storage']
files_to_modify: ['src/main.rs', 'src/lib.rs', 'src/handlers/mod.rs', 'src/handlers/http.rs', 'src/auth/mod.rs', 'src/scraper/mod.rs', 'web/index.html', 'web/js/app.js', 'web/css/styles.css']
code_patterns: ['Hexagonal Architecture', 'Zero Dependency', 'Portable Binary']
test_patterns: ['Unit tests for HTTP handlers', 'Integration tests for auth flow', 'E2E tests for scraper']
---

# Tech-Spec: Fırat Shadow Handbook - Core Skeleton Implementation

**Created:** 2026-02-24T20:50:00.000Z

## Overview

### Problem Statement

Öğretmenler debsis.firat.edu.tr Collab platformunda yaşadığı ses/video kalitesi sorunlarından dolayı ders kayıtları alamıyor. Mevcut sistem 720p ve 320kbps gibi düşük kalite sunuyor, otomatik derse katılım özelliği yok, ve mobil uyumlu değil. Öğretmenlerin yüksek kalitede ders kayıtları alması, otomatik derse katılım yapması ve soru-cevap otomasyonu ihtiyacı var.

### Solution

Zero dependency, portable binary olarak çalışan Rust backend + Vanilla JS frontend ile otonom shadow companion. Sistemin amacı: Collab'in kısıtlarını bypass ederek en yüksek kalitede video kaydı, otomatik derse katılım, soru-cevap otomasyonu (sazan.avi modu), ve tüm cihazlarda responsive çalışma.

### Scope

**In Scope:**
- Zero dependency Rust HTTP sunucusu (std::net)
- Vanilla JS frontend with JSDoc typing
- CAS authentication integration
- Collab scraper for course data
- High-quality video recording (OBS WebSocket)
- Auto-join scheduler for classes
- Sazan.avi mod (question-answer automation)
- Responsive design for mobile/desktop/web
- Filesystem-based storage
- Portable binary deployment

**Out of Scope:**
- External database systems (PostgreSQL, MySQL, etc.)
- Cloud services integration
- Multi-user authentication (single user focus)
- Real-time collaboration features
- Advanced AI/ML features

## Context for Development

### Codebase Patterns

**Hexagonal Architecture Pattern:**
- Domain Layer: Core business logic (auth, scraping, recording)
- Application Layer: Use cases and workflows
- Infrastructure Layer: External adapters (CAS, Collab, OBS)
- Interface Layer: HTTP handlers and frontend

**Zero Dependency Principle:**
- Backend: Sadece std::net ve minimal async
- Frontend: Sadece vanilla JS, JSDoc ile typing
- No build tools, no frameworks, no transpilation

**Portable Binary Strategy:**
- Frontend assets embed edilecek binary içinde
- Single executable deployment
- Cross-platform compatibility

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `src/main.rs` | Application entry point and server initialization |
| `src/lib.rs` | Core library with domain logic |
| `src/handlers/http.rs` | HTTP request handlers and routing |
| `src/auth/cas.rs` | CAS authentication implementation |
| `src/scraper/collab.rs` | Collab platform scraper |
| `src/media/recorder.rs` | High-quality video recording |
| `src/scheduler/auto_join.rs` | Auto-join class scheduler |
| `web/index.html` | Main frontend application |
| `web/js/app.js` | Frontend application logic |
| `web/css/styles.css` | Responsive styling |

### Technical Decisions

**Backend Technology:**
- Rust with std::net for maximum control
- No external crates except absolutely necessary
- Custom HTTP/1.1 parser implementation
- Filesystem-based configuration storage

**Frontend Technology:**
- Vanilla JavaScript with ES modules
- JSDoc for type safety without transpilation
- Native CSS with CSS Grid and Variables
- Service Worker for offline capability

**Storage Strategy:**
- JSON files for user preferences and settings
- Filesystem for recording metadata
- SQLite option for complex queries (future)

**Authentication Flow:**
- Direct CAS integration with TGT/ST tickets
- Session management in-memory with persistence
- Automatic token refresh

## Implementation Plan

### Tasks

**Phase 0 - Core Skeleton (Current Sprint):**

1. **Rust HTTP Server Foundation**
   - File: `src/main.rs`
   - Implement std::net::TcpListener based HTTP/1.1 server
   - Basic request parsing and response handling
   - Multi-threaded request processing

2. **Frontend Bootstrap**
   - File: `web/index.html`, `web/js/app.js`, `web/css/styles.css`
   - Responsive layout with mobile-first design
   - JSDoc typing throughout JavaScript code
   - Basic navigation and state management

3. **Basic Auth Placeholder**
   - File: `src/auth/mod.rs`
   - Mock authentication for development
   - Session management structure
   - Login/logout UI components

**Phase 1 - CAS Auth & Scraper Logic:**

4. **CAS Authentication Implementation**
   - File: `src/auth/cas.rs`
   - TGT/ST ticket management
   - Secure cookie handling
   - Login flow integration

5. **Collab Scraper Core**
   - File: `src/scraper/collab.rs`
   - HTML parsing and data extraction
   - Course schedule retrieval
   - Video URL discovery

6. **Hexagonal Adapters**
   - File: `src/adapters/mod.rs`
   - External service abstraction
   - Domain layer isolation
   - Test doubles for development

**Phase 2 - Live Engine & Media:**

7. **Native WebSocket Implementation**
   - File: `src/websocket/mod.rs`
   - RFC 6455 compliant implementation
   - Real-time communication
   - Connection management

8. **High-Quality Video Recording**
   - File: `src/media/recorder.rs`
   - OBS WebSocket client integration
   - Multiple quality settings
   - Recording metadata management

9. **Live Streaming Engine**
   - File: `src/media/streaming.rs`
   - Real-time video processing
   - Adaptive bitrate handling
   - Stream recording

**Phase 3 - Automation & Deployment:**

10. **Auto-Join Scheduler**
    - File: `src/scheduler/auto_join.rs`
    - Course schedule parsing
    - Automatic class joining
    - Notification system

11. **Sazan.avi Mode**
    - File: `src/automation/sazan.rs`
    - Question detection and answering
    - AI integration for responses
    - Configurable automation levels

12. **Portable Binary Packaging**
    - File: `build.rs`
    - Frontend asset embedding
    - Cross-platform compilation
    - Single executable distribution

### Acceptance Criteria

**Phase 0 Acceptance:**
- [ ] Rust HTTP server responds to GET requests on port 8080
- [ ] Frontend loads and displays responsive layout
- [ ] Mock authentication flow completes successfully
- [ ] Basic navigation between pages works
- [ ] Mobile responsive design verified

**Phase 1 Acceptance:**
- [ ] Real CAS authentication works with debsis.firat.edu.tr
- [ ] Course schedule successfully scraped and displayed
- [ ] Video URLs extracted and playable
- [ ] Session persistence works across restarts

**Phase 2 Acceptance:**
- [ ] High-quality video recording functions
- [ ] OBS integration controls recording
- [ ] Real-time streaming works without lag
- [ ] Multiple quality options available

**Phase 3 Acceptance:**
- [ ] Auto-join successfully enters classes on schedule
- [ ] Sazan.avi mode answers questions automatically
- [ ] Portable binary runs without installation
- [ ] All features work on mobile, desktop, and web

## Additional Context

### Dependencies

**System Dependencies:**
- Rust toolchain (stable)
- OBS Studio (for recording features)
- Modern web browser with ES module support

**External Services:**
- debsis.firat.edu.tr CAS server
- Collab platform (scraping target)
- OBS WebSocket (local connection)

**Zero External Libraries Policy:**
- No npm packages for frontend
- No Rust crates except std library
- No build tools or transpilers

### Testing Strategy

**Unit Testing:**
- HTTP handler testing with mock requests
- Authentication flow testing
- Scraper functionality testing

**Integration Testing:**
- End-to-end auth flow
- Complete scraping workflow
- Recording pipeline testing

**E2E Testing:**
- Full user journey testing
- Cross-browser compatibility
- Mobile device testing

**Performance Testing:**
- Concurrent user handling
- Memory usage monitoring
- Recording quality benchmarks

### Notes

**Development Guidelines:**
- Always prioritize zero dependency principle
- Mobile-first responsive design
- Security-first authentication handling
- Performance optimization for recording

**Known Challenges:**
- CAS authentication complexity
- Cross-origin recording limitations
- Mobile browser recording restrictions
- OBS WebSocket integration complexity

**Future Considerations:**
- Multi-user support scaling
- Cloud storage integration
- Advanced AI features
- Real-time collaboration
