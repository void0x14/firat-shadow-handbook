---
story_id: 2-1
title: CAS Authentication
epic: CAS Auth & Scraper
status: in-progress
created: 2026-02-24
---

# Story 2-1: CAS Authentication

## Goal
Fırat Üniversitesi CAS (Central Authentication Service) sistemi ile gerçek kimlik doğrulama entegrasyonu.

## Acceptance Criteria
- [x] CAS login flow (TGT/ST ticket) implement edildi (`rustls` + redirect/cookie handling)
- [x] Session cookie management (`ShadowSession`, `MoodleSession`, `CSRF-Token`)
- [x] MoodleSession gerçek CAS response cookie zincirinden çıkarılıyor
- [x] Login sonrası Debsis session doğrulaması uygulama session store üzerinden yapılıyor (`/api/validate-session`)

## Implementation Snapshot (2026-02-27)

### Tamamlananlar
1. Hexagonal auth katmanları oluşturuldu:
   - `src/domain/ports/auth_port.rs`
   - `src/infrastructure/cas_adapter.rs`
   - `src/application/login_usecase.rs`
2. Auth API endpointleri aktif:
   - `POST /api/login`
   - `POST /api/logout`
   - `GET /api/validate-session`
3. Güvenlik:
   - CSRF token doğrulama (`X-CSRF-Token` + `CSRF-Token` cookie match)
   - Session fixation mitigation (login'de `ShadowSession` rotation)
   - Prod ortamında `Secure` cookie flag (`APP_ENV=production`)
   - Input validation + deterministic error handling
4. Test durumu:
   - `cargo test` ile auth/security testleri geçiyor (17/17)

### Açık Kalanlar
1. Canlı CAS kullanıcı bilgileri ile E2E doğrulama (ortam bağımlı)
2. Opsiyonel: replay attack telemetry/audit logging genişletmesi

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
