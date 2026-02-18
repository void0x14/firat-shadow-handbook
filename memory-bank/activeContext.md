# Active Context — Fırat Shadow Handbook

## Şu Anki Odak

**Faz 0 — Demo hazırlığı:** memory-bank kurulumu tamamlanıyor, ardından Next.js 15 projesi kurulacak.

## Son Yapılanlar

1. Debsis altyapısı araştırıldı:
   - Debsis = Open LMS (Moodle tabanlı)
   - Auth = Apereo CAS (`jasig.firat.edu.tr/cas`)
   - CAS REST API ile server-side auth mümkün (tarayıcı uzantısı gereksiz)
   - Moodle REST API ile ders listesi + Collab join URL'leri çekilebilir

2. MVP Roadmap yazıldı → `docs/mvp-roadmap.md`

3. memory-bank klasörü inşa ediliyor (şu an bu adım)

## Sonraki Adımlar

1. **memory-bank tamamla** — `activeContext.md`, `progress.md` yaz (şu an)
2. **Faz 0 — Next.js 15 projesi kur** worktree'de
   - `npx create-next-app@latest` ile scaffold
   - shadcn/ui, TailwindCSS kur
   - Mock data ile ders listesi sayfası
   - Ders detay sayfası (Collab aç + Yeniden Bağlan)
   - Öğretmen kontrol paneli mock
   - Fırat renkleri + dark mode
3. **Vercel'e deploy et** — demo linki oluştur

## Aktif Kararlar

- **Tarayıcı uzantısı yok**: CAS REST → Moodle REST, tamamen server-side
- **OBS opsiyonel**: MediaRecorder fallback, graceful degradation
- **Supabase**: Realtime + Auth + Storage tek pakette, free tier yeterli
- **Faz 0 mock data**: Gerçek API entegrasyonu Faz 1'de; demo için hardcoded veri yeterli

## Açık Sorular (Yanıt Bekliyor)

1. CAS REST endpoint'i (`/cas/v1/tickets/`) aktif mi? → Credentials ile test edilmeli
2. Moodle web servisleri aktif mi? → IT aktif etmemişse fallback gerekecek
3. Anonim erişim kapsamı: sadece kayıt izleme mi, yoksa chat de dahil mi?

## Önemli Notlar

- Kullanıcı OBS şifresini bize verir; biz saklamayız, sadece CAS token exchange için kullanırız
- BBB kayıt kalitesi: CRF-30, ~500kbps, 720p — MediaRecorder VP9 5Mbps native çözünürlükten belirgin kötü
- Debsis login sayfasında iki yol var: CAS butonu (öğrenci/öğretmen) + username/password formu (admin)
