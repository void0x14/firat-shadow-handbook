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
  2-1-cas-authentication: in-progress  # mock flow implemented, real CAS pending
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
- [x] **Story 2-1 Baseline Implementation** (2026-02-27)
  - [x] Domain port + use case + CAS adapter katmanları aktif
  - [x] `/api/login`, `/api/logout`, `/api/validate-session` endpointleri çalışıyor
  - [x] Cookie güvenlik attribute'ları: `HttpOnly`, `SameSite=Strict`
  - [x] `cargo test`: 7/7 passing

## Odak Noktası
**Story 2-1: CAS Authentication** (Epic 2)
- Gerçek TGT/ST ticket akışına geçiş (mock -> real CAS)
- HTTPS üzerinden CAS login request/response handling
- **Security:** `Secure` cookie flag (production HTTPS)
- **Security:** CSRF token implementation
- **Security:** Session fixation protection

## Sonraki Workflow
1. `/bmad-bmm-dev-story` ile Story 2-1 real CAS entegrasyonu
2. `/bmad-bmm-code-review` ile Story 2-1 güvenlik/code review
3. Epic 2 tamamlandığında Security Hardening Phase 2 (CSRF, audit logging)

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
