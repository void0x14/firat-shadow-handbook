# Active Context — Fırat Shadow Handbook

## Güncel Durum
**Epic 1 (Core Skeleton) TAMAMLANDI.** Epic 2 (CAS Auth & Scraper) aktif geliştirme aşamasında.
**Security Hardening Phase 1 COMPLETED** - Tüm kritik güvenlik açıkları düzeltildi.
**Performance Optimization COMPLETED** - 9 optimizasyon uygulandı, testler geçiyor (49/49).
**Story 2-1R (Auth Single Authority Session) COMPLETED → review** - fake session üretimi kaldırıldı, auth session authority tekilleştirildi.
**2-1R Follow-up Fix APPLIED (2026-03-06)** - gerçek credential 401 raporuna karşı CAS form-post uyumluluğu güçlendirildi.
**2-1R Persistence Stabilization APPLIED (2026-03-07)** - imzalı `ShadowSession` + server-side `MoodleSession` modeli, validate TTL/grace ve frontend retry aktif.
**2-1R Stability Hotfix APPLIED (2026-03-07)** - remote validate fail hard-logout kaldırıldı, rate limit API-only hale getirildi (refresh 429/siyah ekran engeli).
**2-1R Restart Persistence Hotfix APPLIED (2026-03-07)** - session store ve signing key disk persist edildi; restart sonrası auth restore destekleniyor.
**2-1R Review Follow-ups APPLIED (2026-03-07)** - handler-level lifecycle testi, explicit remote invalidation clear, deterministic authenticated-page validation, real role propagation ve HMAC signing tamamlandı.

## Sprint Status
```yaml
epic-1: done
  1-1-rust-http-server: done
  1-2-frontend-bootstrap: done
  1-3-mock-auth-placeholder: done
  1-4-security-hardening-phase-1: done
  1-5-performance-optimization: done  # ← YENI

epic-2: in-progress
  2-1-cas-authentication: done
  2-1R-auth-single-authority-session: review
  2-2-collab-scraper-core: done
  2-3-hexagonal-adapters: done
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
- [x] **Performance Optimization Sprint** (2026-02-28)
  - [x] **Quick Wins** (5 optimizasyon)
    - [x] Security header duplikasyonu kaldırıldı (http.rs)
    - [x] web_root() OnceLock ile cache'lendi (I/O azaltma)
    - [x] to_hex() lookup table ile optimize edildi (CPU -50%)
    - [x] CompositionRoot singleton yapıldı (Memory -30%, CPU -15%)
    - [x] Rate limiter lazy cleanup (CPU -40%)
  - [x] **Derin Optimizasyonlar** (4 optimizasyon)
    - [x] Thread pool (zero-dependency, std::sync::mpsc)
    - [x] Router wildcard ayrıştırma (O(n) → O(1) exact match)
    - [x] Request cookie cache (OnceLock lazy parsing)
    - [x] CAS form body write! macro (allocation azaltma)
  - [x] `cargo test`: 49/49 passing
- [x] **Story 2-1R: Auth Single Authority Session** (2026-03-06)
  - [x] `/api/cas/callback` deprecated no-op redirect moduna alındı (session cookie issuance kaldırıldı)
  - [x] `MoodleSession` otoritesi `/api/login` CAS->Debsis cookie chain akışına sabitlendi
  - [x] `validate_session_with_transport` 302/303 redirect policy strict allowlist modeline çekildi
  - [x] Logout semantiği `local-only` olarak netleştirildi (`handle_logout` + `logout_with_transport`)
  - [x] Auth regression testleri eklendi; `cargo test` tamamen yeşil
- [x] **2-1R Follow-up: Real Credential 401 Fix** (2026-03-06)
  - [x] CAS login POST hedefi `login?service=...` olarak düzeltildi (service query korunuyor)
  - [x] Hidden input parsing dinamik hale getirildi (`lt` zorunlu değil, whitespace tolerant parser)
  - [x] Frontend login ekranında backend kaynaklı gerçek hata mesajı gösteriliyor
  - [x] Testler genişletildi; `cargo test` 45/45 (lib) + 57/57 (bin)
- [x] **2-1R Follow-up: Refresh Persistence Stabilization** (2026-03-07)
  - [x] Browser auth cookie modeli `ShadowSession` (HttpOnly) olarak tekilleştirildi; `MoodleSession` browser’dan kaldırılıp server-side store’a taşındı
  - [x] `/api/validate-session` local doğrulama + remote TTL (5 dk) + grace period (10 dk) ile agresif logout davranışı yumuşatıldı
  - [x] CAS validate probe debug logları (status, location, body snippet) eklendi; allowlist geçiş path’leri genişletildi
  - [x] Frontend `restoreSession()` akışında transient fail için tek retry eklendi
  - [x] Testler güncellendi; `cargo test` 46/46 (lib) + 60/60 (bin)
- [x] **2-1R Hotfix: Refresh 429 / Hard Logout** (2026-03-07)
  - [x] Remote validate fail artık local session’ı düşürmüyor; session yalnız local shadow expiry ile kapanıyor
  - [x] Rate limiter `/api/*` yollarına sınırlandı; static asset request’leri rate-limit dışına alındı
  - [x] Frontend validate retry 429 durumunu da kapsayacak şekilde genişletildi
- [x] **2-1R Hotfix: Restart Persistence** (2026-03-07)
  - [x] `ShadowSession` state + signing key `data/runtime/shadow_sessions.json` dosyasına persist edildi
  - [x] Session süreleri epoch tabanlı hale getirildi; restart sonrası geçerli session kayıtları filtrelenip geri yükleniyor
  - [x] Testler genişletildi; `cargo test` 46/46 (lib) + 62/62 (bin)
- [x] **Architecture Follow-up Documentation** (2026-03-07)
  - [x] Auth/session kök nedenleri ve modülerleşme öncelikleri `docs/root-cause-remedatation-plan.md` altında dokümante edildi
- [x] **Repository Hygiene Cleanup** (2026-03-07)
  - [x] Format-only Rust diffs ayrı chore commit seti olarak ayrıştırıldı
  - [x] Track edilen audit log artefact'ı repodan çıkarılacak şekilde temizlendi
- [x] **2-1R Review Findings Closure** (2026-03-07)
  - [x] Handler-level `login -> validate -> logout -> validate invalid` testi eklendi
  - [x] Remote `InvalidSession` sinyali local shadow session temizliği ile eşlendi
  - [x] `validate_session_with_transport` 200 doğrulaması authenticated-page sinyallerine çekildi
  - [x] Moodle AJAX üzerinden gerçek kullanıcı bilgisi ve rol çözümlemesi eklendi
  - [x] Frontend sabit `student` ataması kaldırıldı; callback info mesajı görünür hale geldi
  - [x] `ShadowSession` imzası HMAC-SHA256 MAC modeline taşındı
  - [x] `cargo test`: 47/47 (lib) + 68/68 (bin)

## Odak Noktası
**Story 2-1R: Auth Single Authority Session — REVIEW**
- Callback path artık session üretmiyor; fake auth state kaynağı kapatıldı.
- Session persistence modeli artık local `ShadowSession` otoritesine dayanıyor; refresh doğrulaması Debsis probe’a her seferinde bağlı değil.
- Auth lifecycle testleri login→validate→logout→validate invalid senaryosunu kapsıyor.
- CAS login form değişikliklerine dayanıklı parsing ve service-preserving POST akışı aktif.
- Review bulguları kapatıldı: handler-level lifecycle kanıtı, explicit remote invalidation clear, gerçek role propagation, authenticated-page determinism ve HMAC signing aktif.

## Sonraki Workflow
1. Story 2-1R için yeni code review turu çalıştır (`bmad-bmm-code-review`)
2. E2E test: gerçek CAS kimlik bilgileriyle canlı login + role resolution doğrulama
3. Story 3-1 Native WebSocket implementasyonuna geçiş

## Kritik Dosyalar
| Dosya | Açıklama |
|-------|----------|
| `docs/SECURITY_AUDIT_REPORT.md` | Kapsamlı güvenlik audit raporu (16 zafiyet) |
| `docs/plan.md` | Reverse engineering + implementation plan |
| `docs/root-cause-remedatation-plan.md` | Auth/session ve modülerlik için kök neden + fazlı remediasyon planı |
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
1. **Canlı CAS E2E doğrulaması** - geliştirme ortamında network/credential bağımlılığı nedeniyle henüz koşulmadı; yeni role resolution akışı da canlı doğrulama bekliyor
2. **Scraper (Epic 3)** - SSRF ve HTML injection riskleri var, mitigation planlı
3. **Production deployment** - HTTPS, audit logging, penetration test yapılmamış

## Referanslar
- OWASP Top 10
- Rust Security Guidelines
- MDN Web Security
- `docs/SECURITY_AUDIT_REPORT.md` (detaylı rapor)
