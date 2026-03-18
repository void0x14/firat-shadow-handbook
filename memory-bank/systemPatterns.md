# System Patterns — Fırat Shadow Handbook

## Mimari: Hexagonal (Ports & Adapters)

Uygulama, teknik borcu minimize etmek ve bileşen değiştirilebilirliğini artırmak için Hexagonal mimariyi kullanır.

### Görünüm
```
[ Frontend: Vanilla ESM ] 
        ↕ (HTTP/WS)
[ Backend: Rust (Axum) ]
    ├── [ Core / Domain ] : İş mantığı, kurallar
    └── [ Ports ]         : Interface tanımları
        ├── AuthPort
        ├── ScraperPort
        └── DBPort
    └── [ Adapters ]      : Dış dünya implementasyonları
        ├── CASAdapter (CAS REST)
        ├── MoodleAdapter (Scraping/API)
        └── SQLiteAdapter / SupabaseAdapter
```

## Kritik Akışlar

### 1. Modüler Kimlik Doğrulama
Domain katmanı sadece "authenticate(user, pass)" der. Hangi servisin (CAS, LDAP, Local) kullanılacağına Adapter karar verir.

### 2. Otonom Scraper
Moodle verileri çekilirken:
- Önce JSON API (varsa) denenir.
- Fallback olarak HTML Scraping (Rust TL kütüphanesi ile) devreye girer.
- Veri, Domain modellerine dönüştürülüp UI'a iletilir.

### 3. Reactive UI (Zero-Dependency)
Frontend, bir framework yerine `CustomEvents` ve `Native Proxy` objeleri kullanarak state yönetimi yapar. Fragment güncellemeleri `template` etiketleri üzerinden manuel ve optimize şekilde yönetilir.

## DB Şeması (Soyutlanmış)
*Veritabanı bağımsızdır, ancak aşağıdaki varlıklar Domain seviyesinde tanımlıdır:*
- `User`: Moodle kimliği ve rolü.
- `Session`: Canlı ders durumları.
- `Recording`: Metadata ve dosya linkleri.
- `Message`: Chat geçmişi.
