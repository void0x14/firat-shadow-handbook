# Fırat Shadow Handbook - Epics

## Epic 1: Core Skeleton
**Goal:** Zero dependency Rust HTTP sunucusu + Vanilla JS frontend iskeleti

### Story 1.1: Rust HTTP Server Foundation
- std::net::TcpListener ile HTTP/1.1 sunucusu
- Request parsing ve response handling
- Multi-threaded request processing

### Story 1.2: Frontend Bootstrap
- Responsive layout (mobile-first)
- JSDoc typing throughout JavaScript
- Basic navigation ve state management

### Story 1.3: Mock Auth Placeholder
- Mock authentication for development
- Session management structure
- Login/logout UI components

---

## Epic 2: CAS Auth & Scraper
**Goal:** Gerçek CAS authentication ve Collab scraper

### Story 2.1: CAS Authentication
- TGT/ST ticket management
- Secure cookie handling
- Login flow integration

### Story 2.2: Collab Scraper Core
- HTML parsing ve data extraction
- Course schedule retrieval
- Video URL discovery

### Story 2.3: Hexagonal Adapters
- External service abstraction
- Domain layer isolation
- Test doubles for development

---

## Epic 3: Live Engine & Media
**Goal:** Yüksek kaliteli video kayıt ve streaming

### Story 3.1: Native WebSocket
- RFC 6455 compliant implementation
- Real-time communication
- Connection management

### Story 3.2: High-Quality Recording
- OBS WebSocket client integration
- Multiple quality settings
- Recording metadata management

### Story 3.3: Live Streaming Engine
- Real-time video processing
- Adaptive bitrate handling
- Stream recording

---

## Epic 4: Automation & Deployment
**Goal:** Otomatik derse katılım ve portable binary

### Story 4.1: Auto-Join Scheduler
- Course schedule parsing
- Automatic class joining
- Notification system

### Story 4.2: Sazan.avi Mode
- Question detection ve answering
- AI integration for responses
- Configurable automation levels

### Story 4.3: Portable Binary Packaging
- Frontend asset embedding
- Cross-platform compilation
- Single executable distribution
