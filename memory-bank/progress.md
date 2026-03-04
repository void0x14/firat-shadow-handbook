# Progress — Fırat Shadow Handbook

## İlerleme Özeti
- [x] **Phase 0: Architecture Definition (Pure Metal)** - Tamamlandı.
- [x] **Phase 1: Rust Core & TCP Bridge** - Tamamlandı.
  - [x] Story 1-1: Rust HTTP Server Foundation
  - [x] Story 1-2: Frontend Bootstrap
  - [x] Story 1-3: Mock Auth Placeholder
- [ ] **Phase 2: CAS Auth & Scraper** - Devam ediyor.
  - [~] Story 2-1: CAS Authentication (review, canlı CAS doğrulaması bekliyor)
  - [ ] Story 2-2: Collab Scraper Core
  - [ ] Story 2-3: Hexagonal Adapters

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

## Sonraki Adım
- Story 2-1 için canlı credential ile E2E doğrulama ve CR sonrası kapanış.
