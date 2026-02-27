# Project Context — Fırat Shadow Handbook

## Executive Summary

Fırat Shadow Handbook is a zero-dependency, pure-metal web application that serves as an autonomous companion to Fırat University's broken Debsis (Moodle) and Collab (BigBlueButton) systems. Built with Rust `std::net` backend and vanilla JavaScript frontend, it solves critical usability issues without requiring IT department involvement or external dependencies.

## Current State

**Phase**: Epic 2 (CAS Auth & Scraper) - In Progress / Review  
**Date**: 2026-02-27  
**Progress**: Epic 1 complete, Story 2-1 real CAS flow implemented

### Completed Work
- ✅ Rust HTTP server foundation using `std::net::TcpListener`
- ✅ Frontend bootstrap with vanilla JavaScript and JSDoc type safety
- ✅ Mock authentication placeholder system
- ✅ Internationalization system (Turkish/English)
- ✅ Reactive state management using Proxy + CustomEvent
- ✅ Sprint tracking and memory bank documentation
- ✅ Static asset routing recovery (`/css/*`, `/js/*`, `/i18n/*`, `/images/*`)
- ✅ Story 2-1 real flow (`rustls` HTTPS CAS + `/api/login`, `/api/logout`, `/api/validate-session`)
- ✅ CSRF + session fixation protections for auth lifecycle

### Next Immediate Task
**Story 2-1: Review and Live Verification**
- Validate real TGT/ST flow against live CAS credentials
- Close remaining review findings (if any)
- Mark Story 2-1 done and move to Story 2-2

## Technical Architecture

### Core Philosophy: Zero Dependency
- **Backend**: Pure Rust `std::net` - no frameworks, no crates except build tools
- **Frontend**: Vanilla JavaScript with JSDoc for type safety
- **Database**: SQLite/flat files for persistence
- **Styling**: Pure CSS with modern browser features
- **Deployment**: Single portable binary with embedded frontend assets

### Hexagonal Architecture Pattern
```
[ Frontend: Vanilla ESM ] 
        ↕ (HTTP/WS)
[ Backend: Rust (Axum-less) ]
    ├── [ Core / Domain ] : İş mantığı, kurallar
    └── [ Ports ]         : Interface tanımları
        ├── AuthPort
        ├── ScraperPort
        └── StoragePort
    └── [ Adapters ]      : Dış dünya implementasyonları
        ├── CASAdapter (CAS REST)
        ├── MoodleAdapter (Scraping/API)
        └── SQLiteAdapter
```

## Problem Domain

### Target Users
1. **Primary: Teachers** - Forced to use two computers, audio issues, delayed recordings
2. **Secondary: Students** - Early connection problems, missed notifications, recording access delays

### Critical Pain Points Solved
| Problem | Impact | Shadow Handbook Solution |
|---------|--------|--------------------------|
| Two-computer requirement for teachers | Physical burden, setup complexity | OBS WebSocket integration for single computer |
| Audio distortion during fullscreen sharing | Reduced lesson quality | Independent microphone channel routing |
| Recordings available hours later, 720p only | Students cannot review effectively | Immediate high-quality recording via OBS/MediaRecorder |
| Missing direct message notifications | Teachers miss student questions | Web Push + email notifications |
| Early join issues (no audio/video) | Students miss lesson start | Supabase Realtime + auto-reconnect |
| Complex Debsis UI causing delays | Late class attendance | One-click Collab access |
| Database crashes during exam weeks | Data loss | Independent Supabase + daily backups |

## Key Features

### Core Functionality
1. **Autonomous Authentication** - CAS SSO integration without IT involvement
2. **One-Click Class Access** - Direct Collab joining bypassing Debsis complexity
3. **Instant Recording Access** - High-quality recordings available immediately after class
4. **Real-time Notifications** - Web push for messages and class updates
5. **Cross-Platform Compatibility** - Works on any modern browser without plugins

### Advanced Features (Future)
1. **Sazan.avi Mode** - AI-powered automated class participation
2. **Auto-Join Scheduler** - Automatic class connection based on timetable
3. **High-Quality Recording** - 1080p+ recording with independent audio channels
4. **Chat Analytics** - Message analysis and Q&A pattern recognition

## Implementation Strategy

### Development Phases
1. **Phase 0: Core Skeleton** ✅ - Basic HTTP server, frontend shell, mock auth
2. **Phase 1: CAS Auth & Scraper** 🔄 - Real authentication, data extraction
3. **Phase 2: Live Engine & Media** ⏳ - WebSocket, OBS integration, recording
4. **Phase 3: Automation & Deploy** ⏳ - Scheduler, Sazan mode, packaging

### Technical Constraints
- **Zero External Dependencies**: No npm packages, no Rust crates (except build tools)
- **No Framework Overhead**: Direct protocol implementation
- **Single Binary Deployment**: Frontend embedded in Rust binary
- **Cross-Platform**: Linux, Windows, macOS compatibility

### Reverse Engineering Requirements
- **CAS Protocol**: TGT/ST ticket flow implementation
- **OBS WebSocket**: RFC 6455 compliance for recording control
- **Collab API**: JWT token handling and video URL extraction
- **Moodle Services**: AJAX API integration for course data

## Success Metrics

### Technical Success
- [ ] Server starts with `cargo run` on port 8080
- [ ] CAS authentication works with real university credentials
- [ ] WebSocket connections established for real-time features
- [ ] OBS integration enables single-computer teaching
- [ ] Recordings available within 2 minutes of class ending

### User Experience Success
- [ ] Teacher can start class in 2 clicks
- [ ] Student can join class in 3 clicks
- [ ] Notifications work reliably across devices
- [ ] System functions when Debsis is down
- [ ] Interface is fully Turkish with English option

## Risk Assessment

### Technical Risks
- **CAS Protocol Changes**: University may update SSO implementation
- **OBS WebSocket Compatibility**: Version compatibility issues
- **Browser Restrictions**: MediaRecorder API limitations
- **Network Policies**: Firewall blocking WebSocket connections

### Mitigation Strategies
- **Modular Adapters**: Hexagonal architecture allows quick protocol swaps
- **Fallback Mechanisms**: Multiple authentication and recording methods
- **Extensive Testing**: Real-world testing with actual university systems
- **Documentation**: Detailed reverse engineering documentation

## Development Environment

### Current Setup
- **OS**: CachyOS Linux for performance-focused development
- **Editor**: VS Code with Windsurf extensions
- **Build System**: Cargo with custom build.rs for asset embedding
- **Testing**: Manual testing with automated integration tests planned

### Project Structure
```
firat-shadow-handbook/
├── src/                    # Rust backend source
├── web/                    # Frontend assets (embedded)
├── data/                   # Runtime configuration and i18n
├── memory-bank/            # Session persistence and context
├── docs/                   # Implementation plans and documentation
└── _bmad-output/          # BMAD workflow artifacts
```

## Next Steps

### Immediate Actions
1. **Implement Story 2-1**: CAS authentication with real university credentials
2. **Test TGT/ST Flow**: Validate ticket exchange with CAS server
3. **Cookie Management**: Implement MoodleSession persistence
4. **Error Handling**: Robust error recovery for network issues

### Short-term Goals (1-2 weeks)
- Complete Epic 2: Full authentication and data scraping
- Begin Epic 3: WebSocket implementation and OBS integration
- User testing with actual university accounts
- Performance optimization and bug fixes

### Long-term Vision (1-2 months)
- Complete all 4 epics for full MVP
- Deploy to production environment
- Gather user feedback and iterate
- Consider mobile app development

---

**Last Updated**: 2026-02-27  
**Status**: Epic 2 active (Story 2-1 in review)  
**Next Review**: After live credential verification
