# System Patterns — Fırat Shadow Handbook

## Mimari Genel Görünüm

```
Tarayıcı (Next.js 15 / Vercel)
  ↕ Server Actions / API Routes
Next.js Backend
  ├── CAS REST Client     → jasig.firat.edu.tr/cas
  ├── Moodle REST Client  → debsis.firat.edu.tr/webservice/rest/server.php
  ├── OBS WebSocket       → localhost:4455 (opsiyonel)
  └── MediaRecorder API   → tarayıcı (fallback)
  ↕
Harici Servisler
  ├── Supabase (DB + Realtime + Auth + Storage)
  ├── Cloudflare R2 (büyük video dosyaları)
  └── Resend (e-posta bildirimleri)
```

## Kritik Entegrasyon Noktaları

### 1. CAS Auth Flow (Server-Side)
```
POST jasig.firat.edu.tr/cas/v1/tickets/
  body: username=xxx&password=yyy
← 201 Created, Location: /cas/v1/tickets/{TGT}

POST /cas/v1/tickets/{TGT}
  body: service=https://debsis.firat.edu.tr/login/index.php?authCAS=CAS
← 200 OK, body: ST-xxx

GET debsis.firat.edu.tr/login/index.php?authCAS=CAS&ticket=ST-xxx
← MoodleSession cookie

→ Moodle REST API çağrıları (cookie ile)
→ Supabase'e kullanıcı profili + session token kaydet
→ Kullanıcıya JWT döndür
```

### 2. Moodle REST API Çağrıları
```
wsfunction=core_enrol_get_users_courses          → kayıtlı dersler
wsfunction=mod_bigbluebuttonbn_get_bigbluebuttonbns_by_courses → BBB aktiviteleri + join URL
wsfunction=mod_bigbluebuttonbn_get_recordings    → BBB kayıtları
```

### 3. Kayıt Stratejisi — Graceful Degradation
```
OBS kurulu mu?
├── EVET → OBS WebSocket (localhost:4455) → StartRecord/StopRecord
│           1080p H.264/NVENC, yüksek bitrate, anında hazır
└── HAYIR → browser MediaRecorder API
            getDisplayMedia() + getUserMedia() → VP9, ~5 Mbps
            native çözünürlük (1080p/1440p/4K)
            BBB'nin 720p/500kbps CRF-30'undan belirgin üstün
```

### 4. Realtime Session Bridge
```
Supabase Realtime → sessions tablosunu dinle
  status: scheduled → live   : toast + Collab otomatik açılır
  status: live → ended       : "Kayıt hazırlanıyor..." bildirimi

Auto-reconnect: 15 sn polling → ses/görüntü gelmezse soft-reload
Her zaman görünür: [↺ Yeniden Bağlan] butonu
```

### 5. Bildirim Mimarisi
```
Supabase DB (messages) → Supabase Realtime → açık sekme anlık güncellenir
                       → Supabase Edge Function
                           → Web Push (VAPID) → tarayıcı bildirimi
                           → Resend API → e-posta
```

## DB Şeması

```sql
users         (id, moodle_user_id, name, email, role: student|teacher)
courses       (id, moodle_course_id, name, teacher_id)
sessions      (id, course_id, status: scheduled|live|ended, collab_url, started_at)
recordings    (id, session_id, url, source: obs|browser|bbb, duration, size)
messages      (id, course_id, sender_id, content, created_at, read_at)
notifications (id, user_id, type, payload, read, created_at)
push_subs     (id, user_id, endpoint, keys)
```

## Tasarım Kararları

- **Tarayıcı uzantısı yok**: CAS REST API ile server-side auth, Moodle API ile veri çekme
- **IT bağımsız**: BBB sunucu konfigürasyonu, API anahtarı, webhook gerektirmez
- **OBS opsiyonel**: MediaRecorder fallback ile sıfır kurulum
- **Supabase tercih sebebi**: Realtime + Auth + Storage tek pakette, free tier yeterli
- **Vercel tercih sebebi**: Sıfır ops, otomatik HTTPS, Next.js native desteği
