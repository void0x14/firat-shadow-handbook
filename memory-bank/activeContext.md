# Active Context — Fırat Shadow Handbook

## Güncel Durum
**Epic 1 (Core Skeleton) TAMAMLANDI.** Epic 2 (CAS Auth & Scraper) başlıyor.

## Sprint Status
```yaml
epic-1: done
  1-1-rust-http-server: done
  1-2-frontend-bootstrap: done
  1-3-mock-auth-placeholder: done

epic-2: in-progress
  2-1-cas-authentication: ready-for-dev  # ← SONRAKI
```

## Yapılanlar
- [x] Rust HTTP Server (std::net::TcpListener)
- [x] Frontend Bootstrap (Vanilla JS + JSDoc)
- [x] Mock Auth Placeholder
- [x] i18n System (tr.json, en.json)
- [x] Sprint Status & Memory Bank güncellendi

## Odak Noktası
**Story 2-1: CAS Authentication**
- TGT/ST ticket flow implementasyonu
- Ham HTTP ile CAS login
- MoodleSession cookie management

## Sonraki Workflow
`/bmad-bmm-dev-story` ile Story 2-1 implementasyonu

## Kritik Dosyalar
| Dosya | Açıklama |
|-------|----------|
| `docs/plan.md` | Reverse engineering + implementation plan |
| `memory-bank/` | Sessionlar arası context |
| `_bmad-output/sprint-status.yaml` | Sprint tracking |
| `_bmad-output/implementation-artifacts/` | Story dosyaları |

## Kararlar
1. **No Framework**: Hem backend hem frontend'de framework yok.
2. **JSDoc Safety**: TypeScript'in bağımlılıklarını kurmadan tip güvenliği.
3. **Modular Portability**: Modüler yapı sayesinde her an native katmanlara geçiş imkanı.
