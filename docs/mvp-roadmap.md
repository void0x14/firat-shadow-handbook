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

## 3. Portability

- **Unified Core**: Rust core mantığı değişmeden Tauri (Desktop) veya mobile wrapper'lara taşınabilir.
- **Minimalist UI**: Her ekrana uyum sağlayan, ağır asset içermeyen tasarım.
