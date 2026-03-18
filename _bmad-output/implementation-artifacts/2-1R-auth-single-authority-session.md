---
story_id: 2-1R
title: Auth Single Authority Session
epic: CAS Auth & Scraper
status: done
created: 2026-03-06
---

# Story 2.1R: Auth Single Authority Session

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a backend developer,
I want a single authoritative session issuance and validation flow for CAS authentication,
so that login/session state is deterministic, secure, and free from fake-session side effects.

## Acceptance Criteria

1. **Single Session Authority**: `MoodleSession` sadece gerçek CAS/Debsis akışından elde edilmeli; rastgele/uydurma session üretimi olmamalı.
2. **Callback Hardening**: `/api/cas/callback` endpointi ya kaldırılmalı ya da fake session set etmeden güvenli no-op/redirect davranışına çekilmeli.
3. **Validation Policy Tightening**: `validate_session_with_transport` 302/303 için yalnız bilinen güvenli Debsis/CAS hedeflerini kabul etmeli; belirsiz redirect "valid" sayılmamalı.
4. **Logout Semantics**: logout davranışı "local clear" ile "remote invalidate" stratejilerinden biri olarak net tanımlanmalı; kod ve dokümantasyon tutarlı olmalı.
5. **JSON Response Safety**: auth lifecycle endpointlerinde JSON response üretimi güvenli serialization ile kalmalı (`serde_json::json!` veya eşdeğer typed response).
6. **Regression Safety**: login/logout/validate/callback için testler güncellenmeli ve mevcut auth akışını kırmadan geçmeli.

## Tasks / Subtasks

- [x] Auth tek-otorite kararını kodda uygula (AC: 1, 2)
  - [x] `/api/cas/callback` akışını analiz et ve fake `MoodleSession` üretimini kaldır
  - [x] Gerekirse endpointi deprecated/no-op redirect modeline çek
- [x] Session validation redirect politikasını sertleştir (AC: 3)
  - [x] `validate_session_with_transport` içinde allowlist tabanlı redirect kontrolü ekle
  - [x] CAS login redirect, unauthorized ve unknown redirect ayrımlarını netleştir
- [x] Logout davranışını tek modele indir (AC: 4)
  - [x] Uygulanan stratejiyi (`local-only` veya `remote+verify`) kodda ve story notlarında tekilleştir
  - [x] `logout_with_transport` testlerini bu stratejiye göre güncelle
- [x] Auth API response güvenliğini doğrula (AC: 5)
  - [x] `handle_login`, `validate_session`, `handle_logout`, callback path response formatlarını kontrol et
  - [x] Typed JSON serialization standardını sabitle
- [x] Test ve regression güvenliği (AC: 6)
  - [x] Unit test: callback fake-session regression test
  - [x] Unit test: redirect allowlist/denylist
  - [x] Integration test: login -> validate -> logout -> validate invalid
  - [x] `cargo test` yeşil

### Review Follow-ups (AI)

- [x] [AI-Review][Critical] Story’de tamamlandı diye işaretlenen `login -> validate -> logout -> validate invalid` entegrasyon testini gerçek HTTP handler seviyesi için ekle; mevcut kanıt yalnız use-case/mock port testi. [src/application/login_usecase.rs:245]
- [x] [AI-Review][High] `validate_session()` içinde remote probe `AuthError::InvalidSession` döndürdüğünde local shadow session’ı geçerli bırakma; açık remote invalidation durumunda oturumu düşür. [src/main.rs:944]
- [x] [AI-Review][High] `validate_session_with_transport()` için 200 response doğrulamasını body heuristic yerine daha deterministik authenticated-page sinyaline çek. [src/infrastructure/cas_adapter.rs:341]
- [x] [AI-Review][High] Auth response modeline gerçek kullanıcı rolünü ekle ve frontend’de sabit `student` atamasını kaldır. [src/main.rs:743]
- [x] [AI-Review][High] `ShadowSession` imzasını `DefaultHasher` yerine kriptografik amaç için uygun bir MAC/HMAC tasarımına taşı. [src/main.rs:650]
- [x] [AI-Review][Medium] Deprecated callback için `info=cas_callback_deprecated` mesajını frontend’de görünür hale getir. [src/main.rs:987]

## Dev Notes

### Problem Context

- Story 2-1 kapatıldı ancak kök neden analizinde auth tarafında "tek otorite session" ihtiyacı açık kaldı.
- Mevcut kodda callback akışı, geçmişte rastgele session üretimi yaptığı için güvenilirlik riski taşıyor.
- Bu story, yeni feature değil; auth güven modelini deterministik hale getiren remedial hardening story'sidir.

### Technical Requirements

- `MoodleSession` kaynağı tek olmalı: CAS -> service ticket -> Debsis cookie chain.
- Callback endpoint, session issuer olmamalı.
- Session validation’da redirect kabul kriterleri açık ve kodda merkezi olmalı.
- Logout semantiği ürün kararı ile sabitlenmeli:
  - Seçenek A: local cookie clear tek kaynak.
  - Seçenek B: remote CAS logout + follow-up probe.
- JSON output tüm auth endpointlerinde parse-safe kalmalı.

### Architecture Compliance

- Hexagonal sınırlar korunmalı:
  - Policy kararları adapter seviyesinde tutulmalı.
  - Route handler sadece orchestration yapmalı.
  - Auth port sözleşmesi ile runtime davranışı çelişmemeli.
- `main.rs` daha fazla şişirilmemeli; yeni logic helper/policy extraction ile eklenmeli.

### Library / Framework Requirements

- Yeni bağımlılık ekleme zorunlu değil.
- Mevcut stack kullanılmalı: `serde_json`, `thiserror`, `rustls`, `std::net`.
- Zero-dependency prensibi kapsamında gereksiz crate eklenmemeli.

### File Structure Requirements

Beklenen dokunuş alanları:

- `src/main.rs`
- `src/infrastructure/cas_adapter.rs`
- `src/application/login_usecase.rs` (gerekirse davranış netliği için)
- `src/domain/ports/auth_port.rs` (sözleşme netliği gerekiyorsa)
- `src/infrastructure/mod.rs` / ilgili test modülleri
- `_bmad-output/implementation-artifacts/2-1-cas-authentication.md` (completion/file list senkronu için opsiyonel güncelleme)

### Testing Requirements

- Redirect güvenlik testleri:
  - allowlisted redirect -> kontrollü kabul
  - unknown redirect -> invalid session
  - CAS login redirect -> invalid session
- Callback testleri:
  - callback fake session üretmemeli
  - callback sonrası auth state yalancı pozitif olmamalı
- Logout testleri:
  - seçilen stratejiye göre başarı/fail kriteri net olmalı
- End-to-end senaryo:
  - login success
  - validate success
  - logout
  - validate fail

### Previous Story Intelligence

- 2-1 review maddelerinden kalan en kritik risk: redirect ve logout semantics tutarsızlığı.
- Önceki düzeltmelerde JSON serialization güvenliği ve parser hardening iyi bir baseline oluşturdu.
- Bu story, 2-1’in "done" statüsünü bozmadan güvenilirlik boşluklarını kapatan refinement katmanıdır.

### Git Intelligence Summary

Son commit paterni auth düzeltmeleri + performans hardening üzerinde:

- `ba8d133 fix: CAS login session restore and logout button fixes`
- `58c9d6e Remove mock data and demo login, implement CAS redirect authentication`
- Sonraki commitlerde performans ve memory bank güncellemeleri var.

Bu, auth tarafında incremental refactor yaklaşımının repo ile uyumlu olduğunu gösteriyor.

### Project Structure Notes

- Proje Rust `std::net` + Vanilla JS; auth kritik mantığı backend’de kalmalı.
- Sprint akışında bu story bir remediation slice olarak ele alınmalı, Epic 2 kapsamını netleştirmeli.

### References

- [Source: _bmad-output/implementation-artifacts/2-1-cas-authentication.md]
- [Source: docs/root-cause-remedatation-plan.md]
- [Source: src/main.rs]
- [Source: src/infrastructure/cas_adapter.rs]
- [Source: _bmad-output/planning-artifacts/epics.md]
- [Source: _bmad-output/planning-artifacts/project-context.md]

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- create-story workflow (manual execution with user-provided story key)
- artifact analysis: 2-1 story, sprint-status, root-cause remediation plan, auth source files
- red phase: callback fake-session ve redirect allowlist testleri fail edilerek doğrulandı
- validation: `cargo test` (43 lib + 55 bin test) tamamen yeşil
- production bug follow-up: gerçek credential ile 401 raporu sonrası CAS form-post uyumluluk düzeltmesi
- validation: `cargo test` (45 lib + 57 bin test) tamamen yeşil
- production stability follow-up: `ShadowSession` + server-side `MoodleSession` modeline geçiş, validate TTL/grace uygulanması, frontend restore retry iyileştirmesi
- validation: `cargo test` (46 lib + 60 bin test) tamamen yeşil
- persistence follow-up: shadow session state + signing key disk persist katmanı eklendi, restart sonrası auth restore destekleniyor
- validation: `cargo test` (46 lib + 62 bin test) tamamen yeşil
- review follow-up: auth state persist katmanı atomik dosya yazımı + private file permission (`0600`) ile sertleştirildi
- validation: `cargo test` (64 bin test) tamamen yeşil
- review follow-up: test runtime auth state yolu geçici dizine alındı; `cargo test` worktree içinde `src/data/` artefact bırakmıyor
- review follow-up: handler-level auth lifecycle testi, explicit remote invalidation clear, deterministic authenticated-page doğrulaması, gerçek role propagation ve HMAC session signing uygulandı
- validation: `cargo test` (47 lib + 68 bin test) tamamen yeşil

### Completion Notes List

- `/api/cas/callback` session issuer olmaktan çıkarıldı; endpoint artık sadece güvenli deprecated redirect döndürüyor, `MoodleSession`/`ShadowUser` set etmiyor.
- Auth single-authority modeli netleştirildi: `MoodleSession` sadece gerçek CAS->Debsis akışındaki `/api/login` üzerinden üretiliyor.
- `validate_session_with_transport` 302/303 için strict allowlist politikasına çekildi; unknown host/path veya login redirect `InvalidSession` oluyor.
- Logout semantiği `local-only` olarak tekilleştirildi; `handle_logout` local cookie clear + parse-safe JSON response (`serde_json::json!`) veriyor.
- Regression kapsamı genişletildi: callback regression, redirect allowlist/denylist, auth lifecycle (login->validate->logout->validate invalid) testleri eklendi.
- Sprint status `in-progress` ve tamamlanınca `review` olarak güncellendi.
- CAS login form parsing dinamik hale getirildi: hidden field’lar (`lt` zorunlu olmadan) otomatik toplanıyor, POST hedefi `login?service=...` ile korunuyor.
- Frontend login ekranı artık backend’in gerçek hata mesajını gösteriyor; yanlış “hatalı şifre” genellemesi kaldırıldı.
- Kalıcı session modeli uygulandı: browser sadece imzalı `ShadowSession` (HttpOnly) taşıyor, gerçek `MoodleSession` server-side session store’da saklanıyor.
- `/api/validate-session` önce local `ShadowSession` doğruluyor; Debsis probe yalnız TTL periyodunda yapılıyor, remote fail durumunda grace period ile session hemen düşürülmüyor.
- `validate_session_with_transport` için status/location/body-snippet debug logları eklendi ve Debsis geçiş redirect path allowlist’i genişletildi.
- Frontend `restoreSession()` akışına tek seferlik retry eklendi; transient validate hatasında kullanıcı anında logout edilmiyor.
- Remote validate fail davranışı daha da yumuşatıldı: probe başarısız olsa bile local `ShadowSession` aktif kaldığı sürece oturum düşürülmüyor (yalnız `degraded` işaretleniyor).
- Rate limiting refresh kaynaklı siyah ekranı engellemek için API-only hale getirildi; static asset istekleri artık 429 ile kesilmiyor.
- `ShadowSession` state artık `data/runtime/shadow_sessions.json` altında persist ediliyor; server restart sonrası signing key ve aktif session kayıtları geri yükleniyor.
- Auth state persist yazımı artık atomik temp-file + rename modeliyle yapılıyor; crash sırasında yarım JSON bırakma riski azaltıldı.
- Persist edilen auth state dosyası Unix ortamında `0600` izinleriyle yazılıyor; gerçek `MoodleSession` ve signing key yalnız süreç sahibi tarafından okunabiliyor.
- Test çalışma zamanı auth state dosyası `std::env::temp_dir()` altına taşındı; repo içinde yanlışlıkla track edilebilecek runtime JSON artefact üretimi engellendi.
- Review follow-up: gerçek HTTP handler seviyesinde `login -> validate -> logout -> validate invalid` lifecycle testi eklendi; story AC6 kanıtı route katmanında tamamlandı.
- Review follow-up: remote probe açıkça `InvalidSession` döndürdüğünde local `ShadowSession` artık siliniyor; yalnız transient/network hataları `degraded` olarak kalıyor.
- Review follow-up: Debsis authenticated-page doğrulaması `sesskey`/logout marker sinyallerine çekildi ve Moodle AJAX üzerinden gerçek kullanıcı bilgisi + rol çözümlemesi eklendi.
- Review follow-up: auth JSON response’larına `role` alanı eklendi; frontend login/restore akışlarındaki sabit `student` ataması kaldırıldı, `admin` öğretmen-benzeri UI olarak ele alındı.
- Review follow-up: `ShadowSession` imzası `ring` HMAC-SHA256 ile kriptografik MAC’e taşındı.
- Review follow-up: deprecated callback `info=cas_callback_deprecated` mesajı frontend toast olarak görünür hale getirildi.

### File List

- `Cargo.toml` (modified)
- `src/main.rs` (modified)
- `src/infrastructure/cas_adapter.rs` (modified)
- `src/domain/user.rs` (modified)
- `web/js/app.js` (modified)
- `web/js/components.js` (modified)
- `web/i18n/tr.json` (modified)
- `web/i18n/en.json` (modified)
- `_bmad-output/implementation-artifacts/2-1R-auth-single-authority-session.md` (modified)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (updated)

## Senior Developer Review (AI)

### Outcome

Approved

### Summary

- İlk review turunda açılan 6 takip maddesi, son fix commit’i `20c1624 fix(2-1R): resolve auth review follow-ups` ve mevcut `HEAD` kaynak kodu üzerinden yeniden doğrulandı.
- Story claim’leri, uygulama kaynak dosyaları ve test kanıtı birbiriyle tutarlı; yeni source-level bulgu kalmadı.
- `cargo test` çalıştırıldı; `47/47` lib ve `68/68` bin test geçti.
- Context7 üzerinden `ring` 0.17.14 HMAC dokümantasyonu kontrol edildi; implementasyon artık `HMAC_SHA256` tabanlı imza üretiyor ve önceki `DefaultHasher` zafiyeti kapanmış durumda.

### Findings

- Bu turda yeni bulgu yok. Önceki `Review Follow-ups (AI)` maddeleri kod ve test seviyesinde kapatılmış durumda.

### Change Log

- 2026-03-06: Story 2-1R tamamlandı; callback fake-session üretimi kaldırıldı, validate redirect allowlist sıkılaştırıldı, logout `local-only` + typed JSON standardı sabitlendi, auth regression testleri genişletildi.
- 2026-03-06: Follow-up fix: gerçek credential 401 problemi için CAS hidden-field parsing + service-preserving POST akışı güçlendirildi, login hata mesajı iyileştirildi.
- 2026-03-07: Session persistence hotfix: imzalı `ShadowSession` + server-side `MoodleSession` store, validate TTL/grace model, CAS validate debug logları, frontend validate retry.
- 2026-03-07: Stabilite hotfix: remote validate fail durumunda hard logout kaldırıldı; rate limiter API-only uygulanarak refresh sonrası 429 siyah ekran sorunu giderildi.
- 2026-03-07: Restart persistence hotfix: `ShadowSession` state ve signing key disk persist edildi; server restart sonrası geçerli session cookie’leri restore edilebilir hale getirildi.
- 2026-03-07: Review follow-up fix: auth state persist katmanı atomik yazım + private file permission (`0600`) ile sertleştirildi; buna yönelik regression testleri eklendi.
- 2026-03-07: Review follow-up fix: test auth state dosyası geçici dizine taşındı; `cargo test` sonrası `src/data/` runtime artefact bırakma yan etkisi kaldırıldı.
- 2026-03-07: Senior developer code review tamamlandı; 1 kritik, 4 yüksek, 1 orta seviye takip maddesi `Review Follow-ups (AI)` altında açıldı ve story statüsü yeniden `in-progress` olarak senkronlandı.
- 2026-03-07: Review follow-up fix: handler-level auth lifecycle testi, explicit remote invalidation clear, deterministic authenticated-page doğrulaması, gerçek role propagation, HMAC session signing ve callback info UX düzeltmeleri tamamlandı; story yeniden `review` durumuna çekildi.
- 2026-03-07: Follow-up code review tekrarlandı; `20c1624` fix seti ve mevcut `HEAD` için yeni source-level bulgu bulunmadı, story `done` statüsüne alındı ve sprint tracking senkronlanacak.
