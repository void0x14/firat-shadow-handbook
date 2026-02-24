---
story_id: 2-1
title: CAS Authentication
epic: CAS Auth & Scraper
status: ready-for-dev
created: 2026-02-24
---

# Story 2-1: CAS Authentication

## Goal
Fırat Üniversitesi CAS (Central Authentication Service) sistemi ile gerçek kimlik doğrulama entegrasyonu.

## Acceptance Criteria
- [ ] CAS login flow (TGT/ST ticket) çalışıyor
- [ ] Session cookie management
- [ ] MoodleSession elde ediliyor
- [ ] Login sonrası Debsis sayfalarına erişim

## Technical Specs

### CAS Login Flow
```
1. GET https://jasig.firat.edu.tr/cas/login?service={callback}
   → HTML form + JSESSIONID cookie
   → Hidden fields: lt, execution, _eventId

2. POST https://jasig.firat.edu.tr/cas/login
   → Body: username={user}&password={pass}&lt={ticket}&execution=e1s1&_eventId=submit
   → Response: 302 Redirect to service URL with ticket param

3. GET {service_url}?ticket={ST}
   → Response: MoodleSession cookie
```

### File Structure
```
src/
├── domain/
│   ├── mod.rs
│   ├── user.rs
│   └── ports/
│       ├── mod.rs
│       └── auth_port.rs
├── infrastructure/
│   ├── mod.rs
│   └── cas_adapter.rs
└── application/
    ├── mod.rs
    └── login_usecase.rs
```

### Implementation Notes
1. HTTP client: std::net::TcpStream ile ham HTTP
2. Cookie parsing: regex yok, manuel string parsing
3. HTML parsing: hidden field extraction
4. Session storage: memory veya file-based

## Dependencies
- None (std::net only)

## Estimated Effort
High (6-8 hours)

## Reverse Engineering Reference
- `docs/plan.md` Section 8.1: CAS Login Flow (Detaylı)
