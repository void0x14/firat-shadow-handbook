# Tech Context — Fırat Shadow Handbook

## Tech Stack (Tamamı Ücretsiz Tier)

| Katman | Teknoloji | Neden |
|--------|-----------|-------|
| Frontend | Next.js 15 (App Router) + TypeScript | SSR + Server Actions, Vercel native |
| Styling | TailwindCSS + shadcn/ui | Hızlı geliştirme, erişilebilir bileşenler |
| Database | Supabase Free (PostgreSQL 500 MB) | Realtime + Auth + Storage tek pakette |
| Realtime | Supabase Realtime (WebSocket) | Postgres changes → anlık push |
| Auth | Supabase Auth (hibrit) | Kullanıcı kimliği CAS'tan gelir, JWT Supabase'de tutulur |
| Storage | Supabase Storage (1 GB) + Cloudflare R2 (10 GB/ay) | Küçük dosyalar Supabase, büyük videolar R2 |
| Push | Web Push API + VAPID | FCM gerekmez, tamamen ücretsiz |
| E-posta | Resend (3.000/ay ücretsiz) | Basit API, güvenilir deliverability |
| Kayıt | OBS WebSocket v5 (opsiyonel) + MediaRecorder API | Graceful degradation |
| Deploy | Vercel (frontend) + Supabase Cloud | Sıfır ops |
| Debsis köprüsü | CAS REST API → Moodle REST API | Server-side, uzantı yok |

## Harici Sistem Bilgileri

### Debsis (Moodle)
- **URL:** `https://debsis.firat.edu.tr`
- **Platform:** Open LMS (Moodle tabanlı)
- **Auth:** Apereo CAS — `https://jasig.firat.edu.tr/cas`
- **Login butonu:** `?authCAS=CAS` parametresiyle CAS'a yönlendirir
- **Moodle REST endpoint:** `https://debsis.firat.edu.tr/webservice/rest/server.php`
- **Web servisleri aktif mi?** Bilinmiyor — test edilmeli (credentials gerekir)

### CAS Sunucusu
- **URL:** `https://jasig.firat.edu.tr/cas`
- **Versiyon:** Apereo CAS (Jasig)
- **REST API:** `POST /cas/v1/tickets/` → TGT; `POST /cas/v1/tickets/{TGT}` → ST
- **REST aktif mi?** Bilinmiyor — test edilmeli

### OBS WebSocket
- **Port:** 4455 (default)
- **Versiyon:** obs-websocket v5
- **Kullanım:** `StartRecord`, `StopRecord`, `GetRecordStatus`
- **Bağlantı:** `ws://localhost:4455` (öğretmenin bilgisayarında)

## Geliştirme Ortamı Kurulumu

```bash
# Proje kur
npx create-next-app@latest firat-shadow-handbook \
  --typescript --tailwind --app --src-dir --import-alias "@/*"

# shadcn/ui ekle
npx shadcn@latest init

# Supabase client
npm install @supabase/supabase-js @supabase/ssr

# OBS WebSocket client
npm install obs-websocket-js

# Web Push
npm install web-push

# E-posta
npm install resend
```

## Ortam Değişkenleri (.env.local)

```env
NEXT_PUBLIC_SUPABASE_URL=
NEXT_PUBLIC_SUPABASE_ANON_KEY=
SUPABASE_SERVICE_ROLE_KEY=

CAS_BASE_URL=https://jasig.firat.edu.tr/cas
MOODLE_BASE_URL=https://debsis.firat.edu.tr

CLOUDFLARE_R2_ACCOUNT_ID=
CLOUDFLARE_R2_ACCESS_KEY_ID=
CLOUDFLARE_R2_SECRET_ACCESS_KEY=
CLOUDFLARE_R2_BUCKET_NAME=

RESEND_API_KEY=

VAPID_PUBLIC_KEY=
VAPID_PRIVATE_KEY=
VAPID_SUBJECT=mailto:admin@example.com
```

## Teknik Kısıtlar

- **CORS:** CAS ve Moodle API çağrıları server-side yapılmalı (Next.js API routes / Server Actions)
- **Şifre güvenliği:** Kullanıcı şifresi hiçbir zaman DB'ye yazılmaz; sadece CAS token exchange için kullanılır
- **Moodle web servisleri:** IT aktif etmemişse çalışmaz — fallback: Moodle sayfalarını server-side HTML parse et
- **OBS:** Sadece öğretmenin yerel ağında çalışır (`localhost:4455`), production'a expose edilmez
- **MediaRecorder:** `getDisplayMedia()` kullanıcı onayı gerektirir — UI'da açıkça belirtilmeli
