---
story_id: 2-3
title: Hexagonal Adapters
epic: CAS Auth & Scraper
status: done
created: 2026-02-27
completed: 2026-02-28
---

# Story 2-3: Hexagonal Adapters

## Story

As a backend maintainer,
I want external service integrations to be consumed strictly via domain ports,
so that adapters can be swapped/tested without touching domain and application business logic.

## Acceptance Criteria

1. Application katmanı yalnızca `domain::ports` contract'ları üzerinden çalışır; infrastructure concrete tiplerine doğrudan bağımlılık kalmaz.
2. Auth ve Scraper akışlarında adapter değişimi (real adapter vs test double) business logic kodunu değiştirmeden yapılabilir.
3. Story 2-1 ve 2-2 endpoint davranışları korunur (auth + collab scrape regresyonu yok).
4. Port seviyesinde test doubles/fake implementasyonlarla en az birer anlamlı unit test seti bulunur.
5. Dosya/modül organizasyonu hexagonal mimariyi açıkça yansıtır ve yeni bağımlılık eklenmez.

## Tasks / Subtasks

- [x] Composition root ve dependency wiring'i port-first hale getir (AC: 1,2)
  - [x] `main.rs` içinde use-case oluşturma noktalarını port contract odaklı sadeleştir
  - [x] Concrete adapter seçimini tek noktada tut (environment/config bazlı genişlemeye hazır)
- [x] Domain/Application katmanında adapter-bağımsızlık audit'i yap (AC: 1,5)
  - [x] Application dosyalarında infrastructure import sızıntısı olmadığını doğrula
  - [x] Port imzalarını minimal ve yeterli hale getir
- [x] Test double stratejisini güçlendir (AC: 2,4)
  - [x] `AuthPort` ve `ScraperPort` için fake/mock implementasyonlarla başarılı ve hata senaryolarını doğrulayan testler ekle
  - [x] Testlerde business logic'in concrete adapter davranışına bağımlı olmadığını göster
- [x] Regresyon güvence paketi çalıştır (AC: 3)
  - [x] `cargo test` ile mevcut auth/scrape path'lerinin kırılmadığını doğrula
  - [x] Story 2-2'de eklenen route-level hata testlerinin yeşil kaldığını doğrula

## Dev Notes

- Mimari zorunluluk: Domain/Application katmanı infrastructure concrete tiplerini bilmemeli.
- Story 2-1 ve 2-2'de güvenlik sertleştirmeleri yapıldı; bu hikayede fonksiyonel davranış değiştirme değil, bağımlılık sınırını netleştirme hedefleniyor.
- Yeni crate ekleme yok. Mevcut stack ile ilerle (`serde`, `thiserror`, `chrono`, `rustls`, `webpki-roots`).
- BMAD yaklaşımı: Önce test, sonra minimal implementasyon, sonra refactor.

### Technical Requirements

- `ScraperPort` ve `AuthPort` application use-case'lerde birincil dependency olmalı.
- Handler/route katmanında adapter instantiate edilirken business logic contract'tan yürümeli.
- Test doubles gerçek adapter davranışını taklit etmek yerine contract semantiğini doğrulamalı.

### Architecture Compliance

- Hexagonal sınır: `domain` -> bağımsız, `application` -> `domain::ports`, `infrastructure` -> port implementasyonu.
- Story 2-3'te amaç "yeni feature" değil "mimari izolasyon" ve "testlenebilirlik".

### Library / Framework Requirements

- Proje bağımlılıkları güncel durumda yeterli: `rustls 0.23.37`, `webpki-roots 0.26.11`, `thiserror 1.0.69` (cargo info çıktısı).
- Dev sürüm veya major upgrade zorunluluğu yok; bu story kapsamında dependency bump yapılmamalı.

### File Structure Requirements

- Beklenen dokunuş alanları (gerektiğinde):
  - `src/application/login_usecase.rs`
  - `src/application/collab_scraper_usecase.rs`
  - `src/main.rs`
  - `src/domain/ports/auth_port.rs`
  - `src/domain/ports/scraper_port.rs`
  - ilgili test blokları
- `domain` katmanına infrastructure referansı eklenmemeli.

### Testing Requirements

- En az bir success + bir failure senaryosu her port için test double ile doğrulanmalı.
- Route-level ve use-case testleri birlikte yeşil olmalı.
- Final doğrulama: `cd src && cargo test`.

### Previous Story Intelligence (2-2)

- Collab scraper için parser edge-case'leri (case-insensitive anchor, quote-aware tag parse, fallback scope) yakın zamanda düzeltildi; bu davranışlar korunmalı.
- Story 2-2 review bulguları kapandı ancak review section içeriği tarihsel olarak eski kalmış olabilir; 2-3 implementasyonunda kod gerçekliğini test ile kanıtlamak kritik.

### Git Intelligence Summary

- Son commitler auth sertleştirme odaklı (`fix(auth): ...`).
- Kod tabanında güvenlik odaklı yaklaşım sürüyor; 2-3'te bu yaklaşımı mimari izolasyona taşı.

### Latest Tech Information

- `cargo info` çıktısına göre kullanılan sürümler stable çizgide; `rustls` için latest dev sürümü var ancak stable 0.23.x hattında kalmak bu story için daha güvenli.
- Bu story'nin amacı mimari netlik olduğundan version migration kapsam dışı.

### Project Structure Notes

- `src/` altında modüler ayrım zaten mevcut (`domain`, `application`, `infrastructure`).
- Bu story, mevcut ayrımı enforce edip adapter seçim ve test-double stratejisini kalıcı hale getirmeli.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-2-CAS-Auth--Scraper]
- [Source: _bmad-output/planning-artifacts/project-context.md]
- [Source: _bmad-output/implementation-artifacts/2-2-collab-scraper-core.md]
- [Source: src/domain/ports/auth_port.rs]
- [Source: src/domain/ports/scraper_port.rs]
- [Source: src/application/login_usecase.rs]
- [Source: src/application/collab_scraper_usecase.rs]
- [Source: src/main.rs]

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- create-story workflow: story auto-discovery from sprint-status (`2-3-hexagonal-adapters`)
- artifact analysis: epics + previous story + current source tree
- dependency check: `cargo info rustls/webpki-roots/thiserror`

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story intentionally focused on architecture isolation + test doubles, not new feature scope.
- **2026-02-27: Implementation complete**
  - Composition root created: `src/application/composition.rs`
  - Test doubles added: `FakeAuthPort`, `FakeScraperPort`
  - Hexagonal architecture enforced: adapter selection centralized
  - All 44 tests passing (no regressions)

### File List

- `_bmad-output/implementation-artifacts/2-3-hexagonal-adapters.md` (new)
- `src/application/composition.rs` (new) - Composition root ve test doubles
- `src/application/mod.rs` (modified) - composition modülü eklendi
- `src/application/login_usecase.rs` (modified) - Box<dyn AuthPort> desteği eklendi
- `src/application/collab_scraper_usecase.rs` (modified) - Box<dyn ScraperPort> desteği eklendi
- `src/domain/ports/auth_port.rs` (modified) - Clone trait eklendi, expires_at kaldırıldı
- `src/domain/ports/scraper_port.rs` (modified) - Unknown error varyantı kaldırıldı
- `src/infrastructure/collab_scraper_adapter.rs` (modified) - new() metodu eklendi
- `src/infrastructure/cas_adapter.rs` (modified) - expires_at kaldırıldı, Unknown error düzeltildi
- `src/main.rs` (modified) - port-first yaklaşımı uygulandı

## Change Log

- 2026-02-27: Story created with implementation-ready context, guardrails, and test strategy.
- 2026-02-27: Hexagonal architecture implemented - composition root created, test doubles added, adapter independence verified.
- 2026-02-28: Code Review issues fixed - dead code removed, operator precedence fixed, File List updated

## Code Review Report

**Review Date:** 2026-02-28  
**Reviewer:** BMAD Code Review Agent  
**Fix Date:** 2026-02-28  
**Test Results:** ✅ 50/50 tests passing  
**Git Status:** All issues fixed

---

### 🔴 HIGH SEVERITY ISSUES - FIXED ✅

**Issue #1: Dead Code - Unused Session Field** ✅ FIXED
- **Location:** [`src/domain/ports/auth_port.rs`](src/domain/ports/auth_port.rs)
- **Problem:** `Session.expires_at` alanı tanımlı ancak hiçbir yerde kullanılmıyor
- **Fix:** `expires_at` alanı `Session` struct'ından kaldırıldı
- **Files Modified:** `auth_port.rs`, `cas_adapter.rs`, `composition.rs`, `login_usecase.rs`

**Issue #2: File List Incomplete** ✅ FIXED
- **Location:** Story Dev Agent Record → File List
- **Problem:** Git'te modified olan dosyalar File List'te belirtilmemiş
- **Fix:** File List güncellendi - `login_usecase.rs`, `collab_scraper_usecase.rs`, `scraper_port.rs`, `cas_adapter.rs` eklendi

---

### 🟡 MEDIUM SEVERITY ISSUES - FIXED ✅

**Issue #3: Operator Precedence Risk** ✅ FIXED
- **Location:** [`src/application/composition.rs`](src/application/composition.rs)
- **Problem:** `if !cookie.is_empty() && cookie.starts_with("session_") || cookie == "test_session"`
- **Fix:** Parantez eklendi: `if (!cookie.is_empty() && cookie.starts_with("session_")) || cookie == "test_session"`

**Issue #4: Unused Test Double Methods** ✅ FIXED
- **Location:** [`src/application/composition.rs`](src/application/composition.rs)
- **Problem:** `FakeAuthPort::with_delay`, `FakeScraperPort::with_result`, `with_parse_error`, `with_delay` tanımlı ancak kullanılmıyor
- **Fix:** Kullanılmayan metodlar ve `delay_ms` alanları kaldırıldı

**Issue #5: Unused Error Variant** ✅ FIXED
- **Location:** [`src/domain/ports/scraper_port.rs`](src/domain/ports/scraper_port.rs) ve [`src/domain/ports/auth_port.rs`](src/domain/ports/auth_port.rs)
- **Problem:** `ScraperError::Unknown(String)` ve `AuthError::Unknown(String)` varyantları tanımlı ancak kullanılmıyor
- **Fix:** Her iki `Unknown` varyantı da kaldırıldı, kullanım yerleri düzeltildi

---

### ✅ AC VALIDATION RESULTS

| AC # | Description | Status | Evidence |
|------|-------------|--------|----------|
| 1 | Application katmanı port-first çalışır | ✅ PASS | `login_usecase.rs`, `collab_scraper_usecase.rs` sadece port trait'lerini import ediyor |
| 2 | Adapter değişimi business logic'i değiştirmeden yapılabilir | ✅ PASS | `CompositionRoot` ile `AdapterConfig::Production/Test` geçişi |
| 3 | Story 2-1 ve 2-2 davranışları korunur | ✅ PASS | 50/50 test passing, regression yok |
| 4 | Port seviyesinde test doubles var | ✅ PASS | `FakeAuthPort`, `FakeScraperPort` implementasyonları mevcut ve test edilmiş |
| 5 | Hexagonal organizasyon yansıtılır | ✅ PASS | `domain/ports`, `application`, `infrastructure` ayrımı net |

---

### 📊 SUMMARY

- **Git vs Story Discrepancies:** ✅ All resolved
- **Issues Fixed:** 2 High, 3 Medium
- **All Acceptance Criteria:** Met ✅
- **Tests:** 50/50 passing ✅
- **Status:** All issues resolved, code review complete ✅

---

## Status

done
