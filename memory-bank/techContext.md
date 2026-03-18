# Tech Context — Fırat Shadow Handbook

## Tech Stack (Pure Metal / Zero-Dependency)

| Katman | Teknoloji | Neden |
|--------|-----------|-------|
| **Backend** | Rust (`std::net`) | En düşük seviye kontrol, maksimum öğrenme. |
| **Frontend** | Vanilla JS (JSDoc ile) | Bağımlılıksız tip güvenliği, sıfır build step. |
| **Styling** | Saf CSS | Modern tarayıcı özellikleri (Vars, Flex, Grid). |
| **Iletişim** | Ham TCP/HTTP Socket | Framework yükü olmadan protokol seviyesinde işlem. |
| **Database** | SQLite / Flat Files | Bağımlılıkları minimize edilmiş veri saklama. |

## Mimari Prensipler: Hexagonal (Modular)
- **Domain**: İş mantığı (Dersler, kayıtlar).
- **Ports**: Interface tanımları.
- **Adapters**: Dış servis (Moodle, CAS) ve UI implementasyonları.

## Geliştirme Ortamı
- **CachyOS**: Performans odaklı çalışma ortamı.
- **No NPM**: Projede `node_modules` bulunmaz.
- **Single Binary**: Backend derlendiğinde tüm frontend'i de içeren tek bir dosya çıkar.
