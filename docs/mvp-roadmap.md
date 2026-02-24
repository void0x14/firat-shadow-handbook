# Fırat Shadow Handbook — MVP Roadmap (Pure Metal Edition)

Debsis/Collab ekosistemi üzerinde, **sıfır bağımlılık**, **maksimum kontrol** ve **teknik borçsuz** bir mimari ile ayağa kaldırılacak otonom shadow companion planı.

---

## 0. Mimari Vizyon: "Safe & Pure Control"

| Prensip | Detay |
|---------|-------|
| **Pure Rust** | Backend'de `std::net` ve minimal crate politikası. Her byte kontrol altında. |
| **Pure Vanilla** | Frontend'de framework yok, build tool yok. Sadece tarayıcının native gücü. |
| **Zero Transpile** | TypeScript yerine JSDoc ile tipleme. Derleme süreci (frontend) sıfır. |
| **Ken Thompson Way** | "Varlığından şüphe ettiğin koda güvenme." |

---

## 1. Saf Teknoloji Stack (Bağımlılıksız)

| Katman | Teknoloji | Neden |
|--------|-----------|-------|
| **Backend** | Rust (std::net + minimal async) | Öğrenme odaklı, en düşük seviye kontrol. |
| **Frontend** | Vanilla JS (ESM) + JSDoc | Build-time bağımlılıklardan arındırılmış tip güvenliği. |
| **Styling** | Native CSS (Variables & Grid) | Hantal utility kütüphanelerine gerek yok. |
| **Database** | Plain SQL / Filesystem | Karmaşık ORM'ler yerine doğrudan veri yönetimi. |

---

## 2. Operasyonel Fazlar

### Faz 0 — Core Skeleton (Şu An)
- [x] **Pivot**: Tüm framework ve build tool'lar temizlendi.
- [ ] **Rust TCP Listener**: `std::net::TcpListener` ile ham HTTP/1.1 sunucusu iskeleti.
- [ ] **Frontend Bootstrap**: Saf JS ve JSDoc tipleme yapısının kurulması.

### Faz 1 — CAS Auth & Scraper Logic
- **CAS Auth Port**: Rust ile ham POST/GET üzerinden bilet (TGT/ST) yönetimi.
- **Moodle Scraper Port**: HTML verisini stream olarak işleme.
- **Hexagonal Adapters**: Dış servisleri Domain katmanından soyutlama.

### Faz 2 — Live Engine & Media
- **Native WebSocket Implementation**: RFC 6455 standartlarına sadık iletişim.
- **Client-Side MediaRecorder**: Tarayıcı native API'ı ile yüksek kaliteli kayıt.
- **OBS WebSocket Client**: Yerel ağ kontrollü kayıt yönetimi.

### Faz 3 — Storage & Deployment
- **Portable Binary**: Her şeyin (frontend dahil) tek bir derlenmiş dosyada toplandığı yapı.
- **Zero-Ops Deploy**: CachyOS veya herhangi bir Linux sunucuda tek komutla çalışma.

---

## 4. 🎯 IMPLEMENTASYON PLANI (Teknik Spesifikasyon)

### Phase 0 - Core Skeleton (Mevcut Sprint)
**Dosyalar:**
- `src/main.rs` - Rust HTTP sunucusu (std::net)
- `web/index.html` - Responsive frontend
- `web/js/app.js` - Vanilla JS + JSDoc
- `web/css/styles.css` - Mobile-first CSS

**Görevler:**
1. Rust TCP Listener → HTTP/1.1 sunucusu
2. Frontend bootstrap (saf JS, JSDoc tipleme)
3. Mock auth placeholder

### Phase 1 - CAS Auth & Scraper Logic
**Dosyalar:**
- `src/auth/cas.rs` - CAS auth (TGT/ST tickets)
- `src/scraper/collab.rs` - Collab scraper
- `src/adapters/mod.rs` - Hexagonal adapters

**Görevler:**
4. Gerçek CAS authentication
5. Ders programı çekme
6. Video URL discovery

### Phase 2 - Live Engine & Media
**Dosyalar:**
- `src/websocket/mod.rs` - Native WebSocket
- `src/media/recorder.rs` - OBS integration
- `src/media/streaming.rs` - High-quality recording

**Görevler:**
7. RFC 6455 WebSocket implementation
8. OBS WebSocket client
9. Client-side MediaRecorder API

### Phase 3 - Storage & Deployment
**Dosyalar:**
- `src/scheduler/auto_join.rs` - Auto-join system
- `src/automation/sazan.rs` - Sazan.avi mod
- `build.rs` - Portable binary packaging

**Görevler:**
10. Otomatik derse katılım
11. Soru-cevap otomasyonu
12. Tek binary deployment

---

## 5. 📋 TECHNICAL SPECS

### Backend (Rust - Zero Dependency):
```rust
// std::net ile HTTP/1.1 sunucusu
// CAS auth client
// Collab scraper
// OBS WebSocket client
// Auto-join scheduler
```

### Frontend (Vanilla JS + JSDoc):
```javascript
// Responsive dashboard
// Live video preview
// Auto-join controls
// Sazan.avi mod panel
// Backup/restore interface
```

### Veri Depolama:
- Filesystem-based storage
- JSON configuration files
- Recording metadata
- User preferences

---

## 6. 🚀 SAZAN.AVI MOD ÖZELLİKLERİ

### Otomasyon Seviyeleri:
- **Level 1**: Sadece otomatik derse katılım
- **Level 2**: Soru detection + hazır cevaplar
- **Level 3**: AI-powered dynamic responses
- **Level 4**: Full autonomous participation

### Question-Answer Engine:
- Collab chat monitoring
- Keyword-based question detection
- Template response system
- Configurable automation rules

---

## 3. Portability

- **Unified Core**: Rust core mantığı değişmeden Tauri (Desktop) veya mobile wrapper'lara taşınabilir.
- **Minimalist UI**: Her ekrana uyum sağlayan, ağır asset içermeyen tasarım.

debsis.firat.edu.tr ana sitemiz
Giriş yapılacak debsis login sayfası: https://jasig.firat.edu.tr/cas/login
