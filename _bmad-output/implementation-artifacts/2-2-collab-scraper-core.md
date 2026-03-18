---
story_id: 2-2
title: Collab Scraper Core
epic: CAS Auth & Scraper
status: done
created: 2026-02-27
---

# Story 2-2: Collab Scraper Core

## Goal
Collab ders sayfasından temel ders/schedule/playback verilerini güvenli şekilde çekebilen scraper çekirdeğini kurmak.

## Acceptance Criteria
- [x] Collab kaynak HTML/endpoint içeriğinden ders listesi parse ediliyor.
- [x] Course schedule alanları normalize edilip domain modeline dönüştürülüyor.
- [x] Video/playback URL keşfi için temel extraction akışı çalışıyor.
- [x] Parse hatalarında deterministic error dönülüyor (panic yok).
- [x] En az birim testleri ile parser ve error-path'ler doğrulanıyor.

## Tasks / Subtasks
- [x] Scraper port/domain modeli iskeletini tamamla (AC: 1,2)
  - [x] `ScraperPort` contract'ını netleştir
  - [x] Course/schedule veri modeli mapping kur
- [x] Infrastructure scraper çekirdeğini uygula (AC: 1,3,4)
  - [x] HTML/response parser fonksiyonlarını yaz
  - [x] Playback URL extraction logic ekle
  - [x] Hata durumlarını `Result` tabanlı standartlaştır
- [x] Uçtan uca kullanım akışını bağla (AC: 2,3,4)
  - [x] Uygulama katmanına use-case seviyesi çağrı ekle
  - [x] Session/cookie gereksinimini mevcut auth akışıyla hizala
- [x] Test kapsamı ekle (AC: 5)
  - [x] Parser success/failure testleri
  - [x] Bozuk/eksik alan senaryoları

### Review Follow-ups (AI)
- [x] [AI-Review][High] `parse_playback_entries` içinde `<a ...>` araması case-sensitive (`"<a"`) çalışıyor; uppercase/mixed-case anchor etiketlerinde playback linkleri kaçırılıyor. Case-insensitive tag scan veya HTML parser stratejisi eklenmeli (`src/infrastructure/collab_scraper_adapter.rs:91`).
- [x] [AI-Review][Medium] `extract_tag_text` ilk `>` karakterinde tag'i kapatıyor; quoted attribute içinde `>` geçtiğinde tag kırılıyor ve yanlış parse/false error üretebiliyor. Quote-aware tag boundary parse eklenmeli (`src/infrastructure/collab_scraper_adapter.rs:215`).
- [x] [AI-Review][Medium] Playback label fallback'i `extract_text_between(&html[tag_end..], ">", "</a>")` ile yanlış başlangıç alıyor; `data-label` olmayan anchor'larda label extraction güvenilir değil. Anchor content extraction düzeltmesi gerekli (`src/infrastructure/collab_scraper_adapter.rs:117`).
- [x] [AI-Review][Medium] Route-level error path testleri eksik: `/api/collab/scrape` için invalid JSON, missing `html`, expired session ve non-allowlisted playback URL → `422` senaryoları doğrudan handler düzeyinde doğrulanmıyor (`src/main.rs:529`, `src/main.rs:700`).

## Dev Notes
- Hexagonal yapıyı koru: Domain/Ports ↔ Infrastructure ayrımı bozulmayacak.
- Security-first: parse edilen dış veri güvenilmez kabul edilmeli.
- Story 2-1’den gelen session doğrulama davranışıyla uyumlu ilerle.

### References
- [Source: _bmad-output/planning-artifacts/epics.md#Epic-2-CAS-Auth--Scraper]
- [Source: memory-bank/systemPatterns.md]
- [Source: memory-bank/activeContext.md]

## Dev Agent Record

### Debug Log References
- `cargo test` (src/) → 30 passed, 0 failed
- `cargo test` (src/) → 37 passed, 0 failed
- `cargo test` (src/) → 50 passed, 0 failed (after code review fixes)

### Completion Notes
- Hexagonal mimariye uygun şekilde `ScraperPort`, collab domain modelleri ve `CollabScraperUseCase` eklendi.
- `CollabScraperAdapter` ile course/schedule/playback parsing akışı yazıldı; allowlist dışı playback URL'leri `UnsupportedFormat` ile reject edildi.
- `/api/collab/scrape` endpoint'i eklendi; `ShadowSession` + app session doğrulaması yaparak scraper use-case entegrasyonu tamamlandı.
- Bozuk input ve parse hataları için deterministic HTTP hata kodları uygulandı (`400/401/422`).
- Parser/use-case/route seviyelerinde test kapsamı genişletildi.
- ✅ Resolved review finding [High]: Playback extraction artık uppercase/mixed-case `<A>` etiketlerini de yakalıyor.
- ✅ Resolved review finding [Medium]: `extract_tag_text` quote-aware boundary parse ile `>` karakterini attribute içinde güvenli ele alıyor.
- ✅ Resolved review finding [Medium]: `data-label` yokken anchor inner-text fallback ile label extraction düzeltildi.
- ✅ Resolved review finding [Medium]: `/api/collab/scrape` için invalid JSON, missing `html`, expired session ve allowlist dışı playback URL senaryolarını kapsayan route-level testler eklendi.

### File List
- `src/domain/collab.rs` (new)
- `src/domain/ports/scraper_port.rs` (new)
- `src/application/collab_scraper_usecase.rs` (new)
- `src/infrastructure/collab_scraper_adapter.rs` (new)
- `src/domain/mod.rs`
- `src/domain/ports/mod.rs`
- `src/application/mod.rs`
- `src/infrastructure/mod.rs`
- `src/main.rs`

## Change Log
- 2026-02-27: Story 2-2 implemented end-to-end (domain + adapter + use-case + route + tests), status set to `review`.
- 2026-02-27: Senior code review çalıştırıldı; 1 High + 3 Medium bulgu için Review Follow-ups (AI) maddeleri eklendi, story status `in-progress` olarak güncellendi.
- 2026-02-27: Addressed code review findings - 4 items resolved (1 High, 3 Medium); parser robustness ve route error-path test coverage artırıldı, story status `review` olarak güncellendi.
- 2026-02-28: Second code review completed - 3 Medium + 3 Low issues fixed:
  - AC checkboxes updated to completed status
  - `html_unescape` improved to handle hex (`&#x27;`) and decimal (`&#123;`) entities
  - `parse_attr` now rejects empty attribute values
  - `parse_course_entries` optimized with byte-index search (O(n²) → O(n))
  - `/api/collab/scrape` endpoint now validates `Content-Type: application/json` (returns 415 if missing/invalid)
  - 6 new tests added (50 total tests passing)
  - Story status updated to `done`

## Status
done

## Senior Developer Review (AI)

### Reviewer
- Void0x14

### Date
- 2026-02-27

### Outcome
- Changes Requested

### Git vs Story Discrepancies
- Story `File List` içindeki dosyalar mevcut değişiklik setiyle genel olarak uyumlu.
- Uygulama kaynaklarında story dışında da değişiklik/untracked dosya bulunuyor (`src/domain/user.rs`, `src/domain/ports/auth_port.rs`); bunlar Story 2-2 kapsamı dışında görünüyor ve review'da kapsam dışı bırakıldı.

### AC Coverage Sonucu
- AC1 (Ders listesi parse): **IMPLEMENTED** (`src/infrastructure/collab_scraper_adapter.rs:21`).
- AC2 (Schedule normalize + domain mapping): **IMPLEMENTED** (`src/infrastructure/collab_scraper_adapter.rs:138`, `src/domain/collab.rs:17`).
- AC3 (Playback URL extraction): **PARTIAL** (temel akış var, ancak anchor case-sensitivity nedeniyle kaçırma riski var) (`src/infrastructure/collab_scraper_adapter.rs:87`, `src/infrastructure/collab_scraper_adapter.rs:91`).
- AC4 (Deterministic error, panic yok): **IMPLEMENTED/PARTIAL** (`Result` tabanlı akış var; parse kırılganlığı false parse-error riski taşıyor) (`src/infrastructure/collab_scraper_adapter.rs:21`, `src/infrastructure/collab_scraper_adapter.rs:215`).
- AC5 (Test coverage): **PARTIAL** (parser testleri var, route error-path senaryoları eksik) (`src/infrastructure/collab_scraper_adapter.rs:264`, `src/main.rs:700`).

### Findings
- [High] Playback extraction `<a` aramasında case-sensitive davranış nedeniyle gerçek HTML varyantlarında link kaybı olabilir (`src/infrastructure/collab_scraper_adapter.rs:91`).
- [Medium] Tag extraction quote-aware değil; attribute value içinde `>` bulunan inputlarda parser hatalı boundary alıyor (`src/infrastructure/collab_scraper_adapter.rs:215`).
- [Medium] `data-label` yokken label fallback mantığı anchor içeriğini doğru yakalamıyor (`src/infrastructure/collab_scraper_adapter.rs:117`).
- [Medium] Handler testleri sadece 401(no cookie) ve 200(success) kapsıyor; invalid JSON/missing html/expired session/422 mapping regresyonları yakalanmıyor (`src/main.rs:700`).
