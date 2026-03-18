# Firat Shadow Handbook

DURUM: Proje aktif gelistirmeye kapatilmistir (18 Mart 2026).

## Genel Bakis
Firat Universitesi Debsis (Moodle) ve Collab (BigBlueButton) platformlarindaki kronik sorunlari gidermek amaciyla tasarlanmis, harici kutuphane bagimliligi minimuma indirilmis bir yardimci web uygulamasi. Bu proje, modern web ekosistemindeki bagimlilik yiginlari yerine, saf Rust ve Vanilla JS ile "low-level" cozumler uretmeyi hedefleyen bir muhendislik calismasidir.

## Teknik Mevcut Durum (Tamamlananlar)

### Backend (Rust / Pure Metal)
- [x] std::net tabanli ozel HTTP sunucusu ve router yapisi.
- [x] Harici bir runtime (Tokio) yerine std::thread ve mpsc kanal yonetimi.
- [x] src/crypto.rs: SHA-1 ve HMAC-SHA256 algoritmalarinin sifirdan implementasyonu.
- [x] CAS Authentication: TGT/ST redirect zinciri ve cookie yonetimi.
- [x] ShadowSession: HMAC imzali, server-side dogrulamali guvenli oturum mekanizmasi.
- [x] IP bazli rate limiting ve guvenlik basliklari (CSP, XSS korumasi).
- [x] Serde bagimliligi olmadan manuel JSON parsing ve query string handling.
- [x] Hexagonal Mimari: Domain, Application ve Infrastructure katmanlarinin ayristirilmasi.

### Frontend
- [x] Vanilla JS ve ESM (EcmaScript Modules) kullanimi.
- [x] Build/Transpile adimi gerektirmeyen dogrudan tarayici calistirma modeli.
- [x] JSDoc ile statik tip kontrolu ve dokümantasyon.
- [x] CSS Grid ve Flexbox tabanli modern responsive arayuz.
- [x] i18n altyapisi (Turkish/English).

## Roadmap (Yarim Kalanlar / Iptal Edilenler)

### Planlamada Olanlar
- [ ] Epic 3: Collab Scraper ve veri madenciligi servisleri.
- [ ] WebSocket: Native protokol uzerinden anlik bildirim sistemi.
- [ ] Veritabani: SQLite veya Flat-file persistency katmaninin tamamlanmasi.
- [ ] E2E Testleri: Playwright veya Cypress entegrasyonu.
- [ ] Production-ready HTTPS/TLS konfigürasyonu.

### Uzun Vadeli Hedefler
- [ ] Native mobil uygulama entegrasyonu.
- [ ] Otomatik ders kayit indirme ve transcoding asistani.

## Kurulum ve Calistirma

### Gereksinimler
- Rust Toolchain (1.75+)

### Calistirma
```bash
# Proje dizininde
cargo run
```

### Testler
```bash
cargo test
```

## Lisans
MIT License. Bu proje egitim ve teknik inceleme amaclidir. Herhangi bir universite ile resmi baglantisi yoktur.
