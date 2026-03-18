# Project Brief — Fırat Shadow Handbook
> [!CAUTION]
> **DURUM:** Bu proje 18 Mart 2026 itibariyle aktif geliştirmeye kapatılmıştır.
 (ARŞİV)

## Güncel Durum
**PROJE DURDURULDU.** Geliştirme süreci 18 Mart 2026 tarihinde kullanıcı isteğiyle sonlandırılmış ve tüm çalışmalar `main` branch'ine merge edilerek arşivlenmiştir.

## Sprint Status
```yaml
epic-1: done
epic-2: partial-done (archived)
epic-3: cancelled
status: inactive
```

## Kararlar
1. **Archive**: Proje uzun süreliğine donduruldu.
2. **Main Branch Consolidation**: Tüm commit geçmişinin görünür olması için `workspace` üzerindeki çalışmalar `main` branch'ine taşındı.

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
