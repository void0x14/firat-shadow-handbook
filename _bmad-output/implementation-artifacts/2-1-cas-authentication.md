---
story_id: 2-1
title: CAS Authentication
epic: CAS Auth & Scraper
status: done
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
   - `cargo test` ile auth/security testleri geçiyor (23/23)

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
- `rustls` (TLS istemcisi)
- `webpki-roots` (trusted root CA seti)
- `std::net` (ham TCP/HTTP transport)

## Estimated Effort
High (6-8 hours)

## Reverse Engineering Reference
- `docs/plan.md` Section 8.1: CAS Login Flow (Detaylı)

## Tasks / Subtasks

### Review Follow-ups (AI)
- [x] [AI-Review][High] CASAdapter `validate_session` gerçek Debsis/CAS doğrulaması yapacak şekilde implement edildi (`src/infrastructure/cas_adapter.rs`).
- [x] [AI-Review][High] CASAdapter `logout` uzak CAS logout çağrısı ile gerçek invalidation denemesi yapacak şekilde implement edildi (`src/infrastructure/cas_adapter.rs`).
- [x] [AI-Review][High] Hidden input parser attribute sırası ve tek tırnak varyantlarını destekleyecek şekilde sertleştirildi (`src/infrastructure/cas_adapter.rs`).
- [x] [AI-Review][Medium] Geçersiz HTTP method fallback davranışı kaldırıldı; invalid method parse aşamasında reject ediliyor (`src/main.rs`).
- [x] [AI-Review][Medium] `src/Cargo.toml` bağımlılıkları kök manifest ile hizalandı; `src/` içinde `cargo test` tekrar çalışır hale getirildi.
- [ ] [AI-Review][High] `handle_login` ve `validate_session` JSON response body üretimi `format!` yerine `serde_json::json!` ile escape-safe hale getirilmeli (`src/main.rs:391`, `src/main.rs:517`).
- [ ] [AI-Review][High] `validate_session_with_transport` 302/303 redirect durumlarında yalnızca güvenli/known Debsis hedefleri kabul edilmeli; belirsiz redirect "valid session" sayılmamalı (`src/infrastructure/cas_adapter.rs:256`).
- [ ] [AI-Review][Medium] `logout_with_transport` 302/303 durumlarında `Location` doğrulaması ve/veya follow-up session probe eklenmeli (`src/infrastructure/cas_adapter.rs:292`).
- [ ] [AI-Review][Medium] Story `File List` gerçek implementasyon kapsamını yansıtacak şekilde `src/application/login_usecase.rs` ve `src/domain/ports/auth_port.rs` ile güncellenmeli.

## Dev Agent Record

### Debug Log
- 2026-02-27: Review bulgularına göre auth adapter ve request parser güncellendi.
- 2026-02-27: Root ve `src/` çalışma dizinlerinde testler çalıştırıldı.

### Completion Notes
- `validate_session` artık CAS login redirect/401/403 durumlarını `InvalidSession` olarak ele alıyor; sadece geçerli içerik/redirect senaryolarında başarılı dönüyor.
- `logout` artık CAS logout endpointine gerçek istek atıyor ve 2xx/3xx dışındaki cevaplarda hata veriyor.
- Hidden field extraction parserı daha toleranslı hale getirildi (attribute order + quote varyasyonları).
- Invalid HTTP method fallback (`GET`) kaldırıldı; parser artık bilinmeyen methodu reject ediyor.
- Test kapsamı genişletildi ve tüm testler geçti (`23/23`).

## File List
- `src/infrastructure/cas_adapter.rs` (modified)
- `src/main.rs` (modified)
- `src/Cargo.toml` (modified)

## Change Log
- 2026-02-27: Code review bulgularına yönelik High/Medium düzeltmeleri uygulandı; auth doğrulama/logout davranışları gerçek akışa çekildi, parser sertleştirildi, method fallback kaldırıldı ve testler genişletildi.
- 2026-02-27: Senior code review çalıştırıldı; 2 High + 2 Medium bulgu için yeni Review Follow-up maddeleri eklendi, story status `in-progress` olarak güncellendi.

## Senior Developer Review (AI)

### Reviewer
- Void0x14

### Date
- 2026-02-27

### Outcome
- Changes Requested

### Summary
- AC'ler genel olarak implement edilmiş, testler yeşil (23/23).
- Ancak auth/session güvenilirliğini etkileyen 2 yüksek ve 2 orta seviye açık nokta bulundu.

### Findings
- [High] JSON response body'leri `format!` ile string interpolasyon yapıyor; kullanıcı/CAS kaynaklı karakterler JSON yapısını bozabilir, istemci tarafında parse kırılmasına yol açar (`src/main.rs:391`, `src/main.rs:517`).
- [High] `validate_session_with_transport`, CAS login redirect'i dışındaki 302/303 yanıtlarını doğrudan valid kabul ediyor; hatalı yönlendirmelerde false-positive session üretme riski var (`src/infrastructure/cas_adapter.rs:256`).
- [Medium] `logout_with_transport`, 302/303'ü koşulsuz başarı sayıyor; logout doğrulaması için redirect hedefi veya ek probe kontrolü yok (`src/infrastructure/cas_adapter.rs:292`).
- [Medium] Story dokümantasyonunda `File List` implementasyon kapsamıyla tam senkron değil; hexagonal auth akışında kullanılan dosyalar eksik listelenmiş.

### Action Items
- [ ] `handle_login` ve `validate_session` için JSON üretimini `serde_json::json!` tabanına taşı.
- [ ] `validate_session_with_transport` redirect kabul kriterini whitelist tabanlı hale getir.
- [ ] `logout_with_transport` sonrası redirect/sonuç doğrulaması ekle.
- [ ] Story `File List` alanını gerçek kapsamla güncelle.
