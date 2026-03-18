# Project Brief — Fırat Shadow Handbook (ARŞİV)

> [!CAUTION]
> **DURUM:** Bu proje 18 Mart 2026 itibariyle aktif geliştirmeye kapatılmıştır.
 (ARŞİV)


## İlerleme Özeti
- [x] **Phase 0: Architecture Definition (Pure Metal)** - Tamamlandı.
- [x] **Phase 1: Rust Core & TCP Bridge** - Tamamlandı.
  - [x] Story 1-1: Rust HTTP Server Foundation
  - [x] Story 1-2: Frontend Bootstrap
  - [x] Story 1-3: Mock Auth Placeholder
- [ ] **Phase 2: CAS Auth & Scraper** - Devam ediyor.
  - [x] Story 2-1: CAS Authentication (done)
  - [~] Story 2-1R: Auth Single Authority Session (review)
  - [x] Story 2-2: Collab Scraper Core (done)
  - [x] Story 2-3: Hexagonal Adapters (done)

## Kilometre Taşları
- **2026-02-19**: Mimari temizliği yapıldı. Bağımlılıklar sıfırlandı.
- **2026-02-19**: `std::net` ve JSDoc tabanlı yeni stack üzerinde uzlaşıldı.
- **2026-02-19**: Minimalist README ve Roadmap yayınlandı.
- **2026-02-24**: Epic 1 (Core Skeleton) tamamlandı.
- **2026-02-27**: Static route/not-found krizleri çözüldü, frontend tekrar stabil çalışır hale geldi.
- **2026-02-27**: Story 2-1 için login/logout/session endpoint baseline'ı implement edildi.
- **2026-02-27**: Story 2-1 gerçek CAS HTTPS akışı + CSRF/session-fixation korumaları implement edildi.
- **2026-02-28**: **Performance Optimization Sprint** - 9 optimizasyon uygulandı.
  - Quick Wins: Security header duplikasyonu, web_root cache, to_hex lookup table, CompositionRoot singleton, Rate limiter lazy cleanup
  - Derin: Thread pool (zero-dep), Router wildcard ayrıştırma, Request cookie cache, CAS write! macro
- **2026-03-06**: Story 2-1R tamamlandı ve review durumuna alındı.
  - `/api/cas/callback` fake session üretimi kaldırıldı (deprecated no-op redirect)
  - `validate_session_with_transport` redirect allowlist politikası sertleştirildi
  - Logout semantiği `local-only` olarak netleştirildi
  - Auth regression test kapsamı genişletildi, `cargo test` yeşil
- **2026-03-06**: 2-1R follow-up bugfix (gerçek credential 401) uygulandı.
  - CAS login form hidden-field parsing dinamikleştirildi (`lt` zorunlu değil)
  - CAS POST hedefi `login?service=...` korunacak şekilde düzeltildi
  - Login formu backend hata mesajını doğrudan kullanıcıya gösterir hale getirildi
  - `cargo test`: 45/45 (lib) + 57/57 (bin)
- **2026-03-07**: 2-1R persistence stabilization follow-up uygulandı.
  - Browser auth modeli imzalı `ShadowSession` (HttpOnly) cookie + server-side `MoodleSession` store olarak güncellendi
  - `/api/validate-session` local-first + remote TTL (5 dk) + grace period (10 dk) modeline çekildi
  - CAS validate debug logları (status/location/body snippet) eklendi ve redirect allowlist genişletildi
  - Frontend `restoreSession()` için transient fail retry eklendi
  - `cargo test`: 46/46 (lib) + 60/60 (bin)
- **2026-03-07**: 2-1R refresh stability hotfix uygulandı.
  - Remote validate fail durumunda hard logout kaldırıldı; local shadow session aktif kaldıkça auth korunuyor
  - Rate limit sadece `/api/*` endpointlerine uygulanıyor (static asset refresh trafiği 429 üretmiyor)
  - Frontend `restoreSession()` retry listesine 429 eklendi
  - `cargo test`: 46/46 (lib) + 60/60 (bin)
- **2026-03-07**: 2-1R restart persistence hotfix uygulandı.
  - Session state ve signing key disk persist edildi (`data/runtime/shadow_sessions.json`)
  - Shadow session zamanları restart-safe epoch alanlarına çevrildi
  - `cargo test`: 46/46 (lib) + 62/62 (bin)
- **2026-03-07**: Auth remediation için kök neden ve fazlı refactor planı dokümante edildi.
  - `docs/root-cause-remedatation-plan.md` auth authority, god-file parçalama ve string governance başlıklarını topluyor
- **2026-03-07**: Repo hygiene temizliği uygulandı.
  - Format-only Rust diff'leri ayrı chore commit'e ayrıldı
  - Track edilen audit log artefact'ı repo index'inden çıkarıldı; `/logs/` ignore kuralı korunuyor
- **2026-03-07**: 2-1R review follow-up bulguları kapatıldı.
  - Handler-level auth lifecycle testi eklendi (`login -> validate -> logout -> validate invalid`)
  - Remote `InvalidSession` local shadow session temizliği ile eşlendi
  - `validate_session_with_transport` 200 response’ları authenticated-page sinyali + Moodle AJAX user/role çözümlemesine çekildi
  - Auth response `role` alanı backend/frontend arasında gerçek değerle taşınır hale geldi
  - `ShadowSession` signing `ring` HMAC-SHA256 ile güçlendirildi
  - `cargo test`: 47/47 (lib) + 68/68 (bin)

## Sonraki Adım
- Geliştirme durduruldu. `main` branch ana arşivdir.
