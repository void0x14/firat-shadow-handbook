# Fırat Shadow Handbook — MVP Roadmap

Debsis/firat.edu.tr'nin üzerine oturan, öğrenci ve öğretmenin yaşadığı somut sorunları çözen, **sıfır bütçeyle, tarayıcı uzantısı olmadan, all-in-one** çalışan bir shadow companion web uygulamasının kapsamlı MVP planı.

---

## 0. Altyapı Keşfi — Gerçek Bulgular

| Sistem | Detay |
|--------|-------|
| **Debsis** | Open LMS (Moodle tabanlı) — `debsis.firat.edu.tr` |
| **Auth** | Apereo CAS — `jasig.firat.edu.tr/cas` |
| **CAS REST API** | `POST /cas/v1/tickets/` → TGT → ST → Moodle session |
| **Moodle REST API** | `webservice/rest/server.php` — ders listesi, BBB linkleri, kayıtlar |
| **BBB modülü** | `mod_bigbluebuttonbn` — `get_join_url`, `get_recordings`, `meeting_info` |
| **OBS (Öğrenci Bilgi Sistemi)** | `obs.firat.edu.tr` — aynı CAS credentials |

### Auth Flow (Tarayıcı Uzantısı Yok, All-in-One)

```
Kullanıcı → Bizim sitemizde OBS kullanıcı adı + şifre girer
  → Backend: POST jasig.firat.edu.tr/cas/v1/tickets/
      username=xxx&password=yyy
    ← TGT (Ticket Granting Ticket)
  → Backend: POST /cas/v1/tickets/{TGT}
      service=https://debsis.firat.edu.tr/login/index.php?authCAS=CAS
    ← ST (Service Ticket)
  → Backend: GET debsis.firat.edu.tr/login/index.php?authCAS=CAS&ticket={ST}
    ← Moodle session cookie (MoodleSession)
  → Backend: Moodle REST API çağrıları (session cookie ile)
    ← Ders listesi, Collab join URL'leri, kayıtlar
```

**Neden bu çalışır:** Apereo CAS REST protokolü (v3.x'ten beri) programatik TGT/ST almayı destekler. Bizim backend'imiz kullanıcı adına CAS'a authenticate olur, Moodle session açar, API çağrıları yapar. Kullanıcı şifresini bize bir kez verir, biz saklamayız (session token saklarız).

### Kritik Kısıtlar

| Kısıt | Detay |
|-------|-------|
| **Bütçe** | Sıfır — tamamı ücretsiz tier |
| **IT erişimi** | Yok — BBB sunucu konfigürasyonu mümkün değil |
| **OBS** | Opsiyonel — öğretmen kurmak istemeyebilir |
| **Platform** | Web öncelikli; mobil responsive; native app sonraki aşama |
| **Ölçeklenebilirlik** | 1-2 yıllık proje; mimari buna göre kurulmalı |

---

## 1. Sorun & Kök Neden Analizi

| # | Sorun | Kök Neden | Çözüm |
|---|-------|-----------|-------|
| 1 | Kayıt 720p, geç yayınlanıyor | BBB server-side ffmpeg CRF 30-32; işlem kuyruğu; IT erişimi yok | OBS (varsa) veya browser `MediaRecorder` API — her ikisi de BBB'den anlık ve daha iyi |
| 2 | Öğretmen iki bilgisayar kullanıyor | BBB'de ekran paylaşımı + mikrofon aynı anda stabil çalışmıyor | OBS WebSocket (opsiyonel) + tek bilgisayar kurulum rehberi |
| 3 | Tam ekranda ses bozuluyor | WebRTC TWCC: ekran paylaşımı başlayınca audio bitrate düşüyor | OBS varsa bağımsız mikrofon; yoksa BBB "Sadece ses" modu rehberi |
| 4 | Erken giren öğrenci ses/görüntü almıyor | BBB bilinen bug: ICE renegotiation başarısız; webhook yok | Supabase Realtime `session.status` → otomatik soft-reload + her zaman görünür [Yeniden Bağlan] |
| 5 | DM bildirimi gelmiyor, UI berbat | Debsis DM push notification göndermez | Supabase Realtime chat + Web Push (VAPID) + Resend e-posta |
| 6 | Sınav haftası DB çöktü, veri silindi | Debsis'in backup stratejisi yok | Bağımsız Supabase + günlük yedek + Cloudflare R2 |

### MediaRecorder vs BBB Kayıt — Kalite Karşılaştırması

| Kriter | BBB Kaydı | MediaRecorder (browser) | OBS |
|--------|-----------|------------------------|-----|
| Çözünürlük | 720p (sabit) | Ekranın native çözünürlüğü (1080p, 1440p, 4K) | 1080p+ (ayarlanabilir) |
| Bitrate | ~500 kbps (CRF 30-32) | ~2-8 Mbps (VP9, ayarlanabilir) | 8-50 Mbps (NVENC) |
| Gecikme | Saatler (işlem kuyruğu) | Anında (ders biter bitmez) | Anında |
| Kurulum | Sıfır | Sıfır | 15 dk (bir kez) |
| IT bağımlılığı | Evet | Hayır | Hayır |

**Kanıt:** BBB, ffmpeg ile `CRF=30` kullanır (GitHub issue #15504). MediaRecorder VP9 codec'i ile `videoBitsPerSecond: 5_000_000` ayarlandığında ekranın native çözünürlüğünde kayıt alır — BBB'nin 720p/500kbps'inden belirgin şekilde üstün.

---

## 2. Tech Stack (Tamamı Ücretsiz Tier)

```
Frontend   : Next.js 15 (App Router) + TypeScript
Styling    : TailwindCSS + shadcn/ui
Database   : Supabase Free (PostgreSQL 500 MB, Realtime, Auth)
Auth       : Supabase Auth — ama kullanıcı kimliği CAS'tan gelir (hibrit)
Storage    : Supabase Storage (1 GB) + Cloudflare R2 (10 GB/ay) büyük kayıtlar için
Push       : Web Push API + VAPID (ücretsiz, FCM gerekmez)
E-posta    : Resend (3.000 e-posta/ay ücretsiz)
Recording  : OBS WebSocket v5 (opsiyonel) VEYA browser MediaRecorder API (fallback)
Deploy     : Vercel (frontend, ücretsiz) + Supabase Cloud (ücretsiz)
Debsis köprüsü: CAS REST API → Moodle REST API (server-side, uzantı yok)
```

**Neden PostgreSQL / Supabase?**
- 500 MB: Bizim DB'miz sadece sessions, messages, notifications, recordings metadata tutar — öğrenci kişisel verisi Debsis'te kalır. 500 MB fazlasıyla yeter.
- Realtime: WebSocket üzerinden anlık bildirim, chat, session status — başka bir servise gerek yok.
- Alternatif (PlanetScale/Neon/Turso) ölçekte düşünülebilir ama Supabase Realtime + Auth kombinasyonu şimdilik en verimli.

---

## 3. Mimari Genel Görünüm

```
┌─────────────────────────────────────────────────────────────┐
│                    Kullanıcı Tarayıcısı                     │
│  Next.js 15 (Vercel)                                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ Dashboard│  │ Canlı    │  │ Kayıtlar │  │ Mesajlar  │  │
│  │ (Dersler)│  │ Ders     │  │ Arşivi   │  │ & Bildirim│  │
│  └──────────┘  └──────────┘  └──────────┘  └───────────┘  │
└──────────────────────────┬──────────────────────────────────┘
                           │ Server Actions / API Routes
┌──────────────────────────▼──────────────────────────────────┐
│                    Next.js Backend                          │
│  ┌─────────────────┐  ┌──────────────────────────────────┐  │
│  │ CAS REST Client │  │ Moodle REST Client               │  │
│  │ TGT → ST →      │  │ core_course_get_enrolled_courses  │  │
│  │ Moodle session  │  │ mod_bigbluebuttonbn_get_join_url  │  │
│  └─────────────────┘  │ mod_bigbluebuttonbn_get_recordings│  │
│                        └──────────────────────────────────┘  │
│  ┌─────────────────┐  ┌──────────────────────────────────┐  │
│  │ OBS WebSocket   │  │ MediaRecorder Orchestrator       │  │
│  │ (opsiyonel)     │  │ (fallback kayıt)                 │  │
│  └─────────────────┘  └──────────────────────────────────┘  │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                    Harici Servisler                         │
│  Supabase (DB + Realtime + Storage)                         │
│  Cloudflare R2 (büyük video dosyaları)                      │
│  Resend (e-posta bildirimleri)                              │
│  jasig.firat.edu.tr/cas (auth)                              │
│  debsis.firat.edu.tr (Moodle REST API)                      │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. MVP Fazları

### Faz 0 — Bu Akşam Demo (Bugün, ~2-3 saat)
> Hocaya gösterilecek temel akış. Mock data, gerçek UI.

**Yapılacaklar:**
- [ ] Next.js 15 projesi kur, Vercel'e deploy et
- [ ] Ana sayfa: "Bugünkü Dersler" listesi (mock data)
- [ ] Ders kartı: ders adı, saat, öğretmen adı, [Katıl] butonu
- [ ] [Katıl] → Collab URL'yi yeni sekmede aç + ders sayfasına geç
- [ ] Ders sayfası: [Collab'ı Aç] + [↺ Yeniden Bağlan] + mock chat
- [ ] Öğretmen paneli mock: [Dersi Başlat] / [Dersi Bitir] + kayıt durumu
- [ ] Fırat renkleri: lacivert `#1a3a6b`, kırmızı `#c0392b`

**Demo mesajı:** *"Debsis'e girmeden derslerinizi görüyorsunuz, tek tıkla Collab açılıyor, ses/görüntü gelmezse bu buton var, ders biter bitmez kayıt hazır."*

---

### Faz 1 — CAS Auth & Moodle Entegrasyonu (Hafta 1-2)
> Gerçek kullanıcılar, gerçek ders verileri.

**Auth akışı (server-side, uzantı yok):**
```
1. Kullanıcı → OBS kullanıcı adı + şifre girer (bizim login sayfamız)
2. Backend → POST jasig.firat.edu.tr/cas/v1/tickets/
             body: username=xxx&password=yyy
           ← 201 Created, Location: /cas/v1/tickets/{TGT}
3. Backend → POST /cas/v1/tickets/{TGT}
             body: service=https://debsis.firat.edu.tr/login/index.php?authCAS=CAS
           ← 200 OK, body: ST-xxx
4. Backend → GET debsis.firat.edu.tr/login/index.php?authCAS=CAS&ticket=ST-xxx
           ← MoodleSession cookie
5. Backend → Moodle REST API (cookie ile)
           ← Ders listesi, join URL'leri, kayıtlar
6. Backend → Supabase'e kullanıcı profili + session token kaydet
7. Kullanıcı → Bizim JWT'mizi alır, sonraki isteklerde kullanır
```

**Moodle API çağrıları:**
```
POST /webservice/rest/server.php
  wsfunction=core_enrol_get_users_courses
  userid={userId}
  → Kayıtlı dersler

POST /webservice/rest/server.php
  wsfunction=mod_bigbluebuttonbn_get_bigbluebuttonbns_by_courses
  courseids[0]={courseId}
  → BBB aktivite listesi (join URL dahil)

POST /webservice/rest/server.php
  wsfunction=mod_bigbluebuttonbn_get_recordings
  bigbluebuttonbnid={bbnId}
  → Ders kayıtları (BBB tarafındaki)
```

**DB şeması:**
```sql
users         (id, moodle_user_id, name, email, role: student|teacher)
courses       (id, moodle_course_id, name, teacher_id)
sessions      (id, course_id, status: scheduled|live|ended, collab_url, started_at)
recordings    (id, session_id, url, source: obs|browser|bbb, duration, size)
messages      (id, course_id, sender_id, content, created_at, read_at)
notifications (id, user_id, type, payload, read, created_at)
push_subs     (id, user_id, endpoint, keys)
```

---

### Faz 2 — Live Session Bridge (Hafta 3-5)
> Collab'a girmek için tek, temiz giriş noktası + auto-reconnect.

**Öğrenci akışı:**
1. Dashboard → "Bugünkü Dersler" → [Katıl]
2. Supabase Realtime `sessions` tablosunu dinler
3. Öğretmen [Dersi Başlat] → `status="live"` → Realtime push → otomatik toast + Collab açılır
4. Ses/görüntü gelmezse → 15 sn polling → otomatik soft-reload
5. Her zaman görünür [↺ Yeniden Bağlan] butonu
6. Ders biter → `status="ended"` → "Kayıt hazırlanıyor..." bildirimi

**Öğretmen akışı:**
1. [Dersi Başlat] → `status="live"` yaz → Collab açılır
2. OBS bağlıysa → WebSocket `StartRecord` (1080p, H.264)
3. OBS yoksa → browser MediaRecorder başlar (VP9, ~5 Mbps, native çözünürlük)
4. Kayıt göstergesi: süre, kaynak (OBS/Browser), tahmini boyut
5. [Dersi Bitir] → kayıt durdur → Cloudflare R2'ye yükle → bildirim gönder

**UI — Öğrenci Ders Sayfası:**
```
┌─────────────────────────────────────────────────────┐
│  Veri Yapıları  ● Canlı  •  14:30'dan beri  [90 dk]│
│  ─────────────────────────────────────────────────  │
│  [Collab'ı Aç →]              [↺ Yeniden Bağlan]   │
│  Bağlantı sorunu? Otomatik kontrol ediliyor...      │
│  ─────────────────────────────────────────────────  │
│  Ders Soruları                                      │
│                                                     │
│  ┌──────────────────────────────────────────────┐  │
│  │ Ali K.                              14:42    │  │
│  │ Hoca, 3. slayttaki örneği anlatır mısınız?  │  │
│  └──────────────────────────────────────────────┘  │
│           ┌──────────────────────────────────────┐  │
│           │ Merve H. (Öğretmen)        14:43    │  │
│           │ Tabii, şimdi dönüyorum.             │  │
│           └──────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────┐  │
│  │ Soru veya yorum yaz...                  [→] │  │
│  └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

**UI — Öğretmen Kontrol Paneli:**
```
┌─────────────────────────────────────────────────────┐
│  Veri Yapıları  •  Ders Kontrolü                    │
│  ─────────────────────────────────────────────────  │
│  [▶ Dersi Başlat]                                   │
│                                                     │
│  Kayıt: ● OBS (1080p)  •  00:42:17                 │
│  OBS bağlı değil? [Browser kaydına geç]            │
│                                                     │
│  Yeni sorular: 3  [Görüntüle]                      │
└─────────────────────────────────────────────────────┘
```

---

### Faz 3 — Kayıt Arşivi (Hafta 6-7)
> BBB'nin geç ve düşük kaliteli kayıtlarına alternatif.

- Ders biter bitmez kayıt Cloudflare R2'ye yüklenir (anında erişilebilir)
- Video player: MP4/WebM, ileri/geri sarma, hız kontrolü (0.5x–2x)
- BBB kaydı da gösterilir (hazır olunca — saatler sonra)
- Moodle API'den gelen BBB kayıtları da listelenir (`get_recordings`)
- Anonim erişim: öğretmen izin verdiyse kayıtlar giriş yapmadan izlenebilir

**UI — Kayıt Sayfası:**
```
┌─────────────────────────────────────────────────────┐
│  Veri Yapıları  •  12 Kasım 2024  •  90 dk          │
│                                                     │
│  ┌─────────────────────────────────────────────┐   │
│  │  ▶  [══════════════●══════════] 42:17/90:00 │   │
│  │  0.5x  0.75x  [1x]  1.25x  1.5x  2x        │   │
│  │  Kaynak: Browser (1080p native)  ↓ İndir    │   │
│  └─────────────────────────────────────────────┘   │
│  BBB kaydı: İşleniyor... (tahminen 2 saat)         │
└─────────────────────────────────────────────────────┘
```

---

### Faz 4 — Mesajlaşma & Bildirim (Hafta 8-9)
> Debsis DM'inin yetersizliğini çözer. Modern, gerçek zamanlı.

**Mimari:**
```
Öğrenci mesaj gönderir
  → Supabase DB (messages tablosu)
    → Supabase Realtime → öğretmenin açık sekmesi anında güncellenir
    → Supabase Edge Function:
        → Web Push (VAPID) → tarayıcı bildirimi (telefon/masaüstü)
        → Resend API → e-posta (öğretmen@firat.edu.tr)
```

**Özellikler:**
- Kurs bazlı soru kanalı — Slack `#channel` mantığı (herkes görür, öğretmen yanıtlar)
- Özel mesaj (öğrenci ↔ öğretmen, gizli)
- Okundu/okunmadı göstergesi + "3 yanıtsız soru" özeti
- Bildirim tercihleri: push + e-posta / sadece biri / hiçbiri
- WhatsApp/Slack benzeri baloncuk UI — Debsis DM'inden tamamen farklı

---

### Faz 5 — Veri Güvenliği & Yedekleme (Hafta 10)
> "Sınav haftası DB çöktü" senaryosunu önler.

- Supabase built-in günlük yedekleme (free tier: 7 gün PITR)
- Haftalık `pg_dump` → Cloudflare R2'de şifreli `.sql.gz` (30 gün)
- Cloudflare R2 kayıtları: versiyonlama aktif
- Durum sayfası: sistem sağlığı görünür (`/status`)
- Veri ihracı: öğrenci kendi verilerini ZIP olarak indirebilir

---

## 5. Öğretmen Merve — Özet Çözüm Tablosu

| Sorun | Çözüm | Faz |
|-------|-------|-----|
| İki bilgisayar, ses sorunları | OBS WebSocket (opsiyonel) + tek bilgisayar rehberi | Faz 2 |
| Tam ekranda ses bozuluyor | OBS varsa bağımsız mikrofon; yoksa BBB "Sadece ses" modu rehberi | Faz 2 |
| Erken giren öğrenci ses/görüntü almıyor | Supabase Realtime + 15 sn polling → auto soft-reload | Faz 2 |
| DM bildirimi gelmiyor | Web Push (VAPID) + Resend e-posta | Faz 4 |
| Debsis çöküyor, veri siliniyor | Bağımsız Supabase + günlük yedek + Cloudflare R2 | Faz 5 |

---

## 6. UI/UX Prensipleri

- **All-in-one** — tek web sitesi, uzantı yok, ek uygulama yok
- **Sıfır öğrenme eğrisi** — öğrenci: 3 tıkla derse gir. Öğretmen: 2 tıkla dersi başlat
- **Otonom/otomatik** — kullanıcıdan mümkün olan en az şey istenir
- **Web öncelikli, mobil responsive** — büyük dokunma hedefleri, tek sütun mobilde
- **Türkçe öncelikli** — tüm UI Türkçe
- **Durum görünürlüğü** — "Ders canlı 🔴", "Kayıt hazır ✓", "Bağlantı kesildi ⚠️" her zaman net
- **Renk paleti** — Fırat lacivert `#1a3a6b` + kırmızı `#c0392b` + shadcn/ui nötr tonlar
- **Dark mode** — sistem tercihine göre otomatik
- **Chat UI** — WhatsApp/Slack benzeri baloncuklar, zaman damgası, okundu işareti
- **Ölçeklenebilir mimari** — 1-2 yıllık proje; modüler yapı; her faz bağımsız deploy edilebilir

---

## 7. Zaman Çizelgesi

```
Bugün       : Faz 0 — Demo (mock data, temel UI, Vercel deploy)
Hafta 1-2   : Faz 1 — CAS auth, Moodle API entegrasyonu, DB şeması
Hafta 3-5   : Faz 2 — Live session bridge, OBS/MediaRecorder, auto-reconnect
Hafta 6-7   : Faz 3 — Kayıt arşivi, video player
Hafta 8-9   : Faz 4 — Mesajlaşma & bildirim (Web Push + Resend)
Hafta 10    : Faz 5 — Yedekleme & güvenlik
Hafta 11    : Beta test (gerçek derslerle, gerçek öğrencilerle)
Hafta 12    : MVP yayın
```

---

## 8. Kalan Açık Sorular

1. **CAS REST aktif mi?** `jasig.firat.edu.tr/cas/v1/tickets/` endpoint'i aktif mi? (Test edilmesi gerekiyor — credentials olmadan test edilemez)
2. **Moodle web servisleri aktif mi?** `debsis.firat.edu.tr/webservice/rest/server.php` erişilebilir mi? (IT aktif etmemişse Moodle API çalışmaz — fallback: Moodle sayfalarını server-side parse et)
3. **Cloudflare R2 hesabı:** Kredi kartı doğrulaması gerekebilir (ücret yok, sadece doğrulama).
4. **Anonim erişim kapsamı:** Giriş yapmayan kullanıcı sadece kayıtları mı izler, yoksa chat'e de katılabilir mi?
