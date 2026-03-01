# Active Context — Fırat Shadow Handbook

## Güncel Durum
**Epic 1 (Core Skeleton) TAMAMLANDI.** Epic 2 (CAS Auth & Scraper) aktif geliştirme aşamasında.
**Security Hardening Phase 1 COMPLETED** - Tüm kritik güvenlik açıkları düzeltildi.

## Sprint Status
```yaml
epic-1: done
  1-1-rust-http-server: done
  1-2-frontend-bootstrap: done
  1-3-mock-auth-placeholder: done
  1-4-security-hardening-phase-1: done  # ← YENI

epic-2: in-progress
  2-1-cas-authentication: review  # real flow implemented, live CAS doğrulaması bekliyor
```

## Yapılanlar
- [x] Rust HTTP Server (std::net::TcpListener)
- [x] Frontend Bootstrap (Vanilla JS + JSDoc)
- [x] Mock Auth Placeholder
- [x] i18n System (tr.json, en.json)
- [x] Sprint Status & Memory Bank güncellendi
- [x] **Security Hardening Phase 1** (2026-02-25)
  - [x] Path traversal prevention (serve_static_file)
  - [x] Input validation framework (validate_path, validate_header_key, body size limits)
  - [x] Rate limiting middleware (100 req/min per IP)
  - [x] Secure headers (CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy)
  - [x] CORS restriction (same-origin default, no wildcard)
  - [x] XSS prevention frontend (escapeHtml utility, parameter escaping)
  - [x] Information leakage fix (removed User-Agent from logs)
  - [x] Comprehensive security audit report
- [x] **Static Routing & Frontend Recovery** (2026-02-27)
  - [x] `/`, `/css/*`, `/js/*`, `/i18n/*`, `/images/*` static serving düzeltildi
  - [x] Nested asset path desteği eklendi (wildcard route)
  - [x] Sidebar active-state bug fix (`data-nav`)
  - [x] Avatar asset eksikliği giderildi (`avatar-placeholder.svg`)
- [x] **Story 2-1 Real CAS Implementation** (2026-02-27)
  - [x] `rustls` ile gerçek HTTPS CAS istemcisi
  - [x] TGT/ST redirect + cookie chain akışı implementasyonu
  - [x] `/api/login`, `/api/logout`, `/api/validate-session` endpointleri session store ile güncellendi
  - [x] CSRF doğrulama ve session fixation koruması eklendi
  - [x] `cargo test`: 17/17 passing

## Odak Noktası
**Story 2-1: CAS Authentication — FIXED** (Epic 2)
- Login formu orijinal tasarıma döndürüldü (username/password → backend headless CAS → gerçek MoodleSession)
- TLS os error 11 (EAGAIN) fix: socket timeout kaldırıldı, blocking mode
- TLS close_notify: Zero Trust validation ile truncation koruması
- Logout düzeltildi: CAS'a sahte token gönderme yerine sadece cookie temizleme
- ShadowUser cookie eklendi (frontend kullanıcı adını okuyabilsin)
- SameSite=Lax (cross-site redirect uyumluluğu)
- 88/88 test geçiyor

## Sonraki Workflow
1. E2E test: gerçek CAS kimlik bilgileriyle canlı login doğrulama
2. Epic 2 tamamlandığında Security Hardening Phase 2 (CSRF, audit logging)
3. Story 2-2: Collab Scraper entegrasyonu

## Kritik Dosyalar
| Dosya | Açıklama |
|-------|----------|
| `docs/SECURITY_AUDIT_REPORT.md` | Kapsamlı güvenlik audit raporu (16 zafiyet) |
| `docs/plan.md` | Reverse engineering + implementation plan |
| `memory-bank/` | Sessionlar arası context |
| `_bmad-output/sprint-status.yaml` | Sprint tracking |
| `_bmad-output/implementation-artifacts/` | Story dosyaları |

## Kararlar
1. **No Framework**: Hem backend hem frontend'de framework yok.
2. **JSDoc Safety**: TypeScript'in bağımlılıklarını kurmadan tip güvenliği.
3. **Modular Portability**: Modüler yapı sayesinde her an native katmanlara geçiş imkanı.
4. **Security-First Development**: Her epic'de security task'ı mandatory. "No feature without security review" prensibi.
5. **Zero-Dependency Security**: Harici crate kullanmadan custom security implementations (rate limiter, validation, etc.).
6. **Shift-Left Security**: Geliştirme başlarken security'yi düşün, sonra değil.

## Güvenlik Standartları (Security Baseline)
Tüm future code şu kurallara uymalı:

### Backend (Rust)
- [ ] Tüm user input'u validate et (path, headers, body)
- [ ] Path traversal'ı engelle (sanitize_filename)
- [ ] Rate limiting uygula (per-IP)
- [ ] Security headers ekle (CSP, X-Frame-Options, etc.)
- [ ] CORS'u restrict et (same-origin veya whitelist)
- [ ] Log'larda sensitive data olmasın
- [ ] Cookie'ler secure attributes ile

### Frontend (Vanilla JS)
- [ ] `innerHTML` kullanma, `textContent` veya `escapeHtml()` kullan
- [ ] Dynamic parametreleri escape et
- [ ] Template literal'lerde user data'ı sanitize et
- [ ] XSS prevention pattern'lerini uygula

### Deployment
- [ ] HTTPS mandatory (production)
- [ ] Reverse proxy (nginx) security headers
- [ ] Regular security audits (cargo audit)
- [ ] Penetration test before launch

## Bilinen Riskler
1. **Epic 2'de CAS auth implementasyonu** - Cookie security ve CSRF protection eklenmeli
2. **Scraper (Epic 3)** - SSRF ve HTML injection riskleri var, mitigation planlı
3. **Production deployment** - HTTPS, audit logging, penetration test yapılmamış

## Referanslar
- OWASP Top 10
- Rust Security Guidelines
- MDN Web Security
- `docs/SECURITY_AUDIT_REPORT.md` (detaylı rapor)
