# Progress — Fırat Shadow Handbook

## Şu Anki Durum

**Faz:** Pre-Faz 0 — Planlama & Dokümantasyon tamamlandı, kodlama başlamadı.

## Tamamlananlar

- [x] Debsis altyapısı araştırıldı (Moodle + CAS + BBB)
- [x] Sorun analizi yapıldı (6 kritik sorun, kök nedenler, çözümler)
- [x] Tech stack belirlendi (Next.js 15 + Supabase + Vercel + Cloudflare R2)
- [x] Auth flow tasarlandı (CAS REST → Moodle session, tarayıcı uzantısı yok)
- [x] MVP roadmap yazıldı → `docs/mvp-roadmap.md`
- [x] memory-bank klasörü inşa edildi (6 core dosya)

## Yapılacaklar

### Faz 0 — Demo (Bugün)
- [ ] Next.js 15 projesi scaffold (`create-next-app`)
- [ ] shadcn/ui + TailwindCSS kurulumu
- [ ] Mock data: ders listesi (3-4 ders, farklı durumlar)
- [ ] Ana sayfa: "Bugünkü Dersler" grid/list
- [ ] Ders kartı bileşeni: ad, saat, öğretmen, durum badge, [Katıl] butonu
- [ ] Ders detay sayfası: [Collab'ı Aç] + [↺ Yeniden Bağlan] + mock chat
- [ ] Öğretmen paneli: [▶ Dersi Başlat] / [■ Dersi Bitir] + kayıt durumu göstergesi
- [ ] Fırat renkleri + dark mode
- [ ] Vercel deploy

### Faz 1 — CAS Auth & Moodle Entegrasyonu (Hafta 1-2)
- [ ] Supabase projesi kur
- [ ] CAS REST client (server-side)
- [ ] Moodle REST client
- [ ] Login sayfası (OBS kullanıcı adı + şifre)
- [ ] DB şeması oluştur (Supabase migrations)
- [ ] JWT session yönetimi

### Faz 2 — Live Session Bridge (Hafta 3-5)
- [ ] Supabase Realtime entegrasyonu
- [ ] OBS WebSocket client
- [ ] MediaRecorder API entegrasyonu
- [ ] Auto-reconnect logic (15 sn polling)
- [ ] Cloudflare R2 upload

### Faz 3 — Kayıt Arşivi (Hafta 6-7)
- [ ] Video player bileşeni (hız kontrolü dahil)
- [ ] BBB kayıt listesi (Moodle API'den)
- [ ] Anonim erişim kontrolü

### Faz 4 — Mesajlaşma & Bildirim (Hafta 8-9)
- [ ] Realtime chat (Supabase)
- [ ] Web Push (VAPID) kurulumu
- [ ] Resend e-posta entegrasyonu
- [ ] Supabase Edge Function (bildirim tetikleyici)

### Faz 5 — Veri Güvenliği (Hafta 10)
- [ ] pg_dump → Cloudflare R2 otomatik yedek
- [ ] Durum sayfası (`/status`)
- [ ] Veri ihracı (ZIP)

## Bilinen Sorunlar / Riskler

| Risk | Olasılık | Çözüm |
|------|----------|-------|
| CAS REST endpoint aktif değil | Orta | Fallback: Moodle HTML scraping |
| Moodle web servisleri kapalı | Yüksek | Fallback: Moodle HTML scraping |
| Cloudflare R2 kredi kartı gerektirir | Düşük | Alternatif: Supabase Storage (1 GB) |

## Proje Kararlarının Evrimi

- **v1 (ilk taslak):** Tarayıcı uzantısı ile Collab linki çekme → **iptal edildi** (kullanıcı deneyimi kötü)
- **v2 (final):** CAS REST API → Moodle REST API, tamamen server-side, uzantı yok
- **Kayıt:** BBB kaydı birincil → **değiştirildi** → OBS/MediaRecorder birincil, BBB yedek
- **Auth:** Supabase Auth önce düşünüldü → **hibrit** yapıya geçildi (CAS kimlik, Supabase JWT)
