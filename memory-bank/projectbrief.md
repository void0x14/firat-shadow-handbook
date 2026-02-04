# Project Brief: Operasyon Gölge Vekil (Fırat Shadow Handbook)

## Proje Tanımı
Fırat Üniversitesi öğrencilerinin OBS (Öğrenci Bilgi Sistemi) verilerine, üniversitenin sunduğu hantal ve mobil uyumsuz arayüz yerine; modern, hızlı, offline-first ve dağıtık bir yapı üzerinden erişmesini sağlayan "Gölge" bir mobil uygulamadır.

## Temel Felsefe
- **Gölge Gibi:** Üniversite sunucularında iz bırakmadan (Stealth Mode), sanki gerçek bir kullanıcıymış gibi davranır.
- **Yok Edilemez:** IP banlansa, sunucular çökse bile offline verilerle çalışmaya devam eder (Resilience).
- **Otonom:** Kullanıcı sormadan notları arka planda günceller (Background Scraping).

## Kritik Hedefler
1.  **Evasion (Gizlilik):** BİDB (Bilgi İşlem) radarından kaçınmak için "Smart UA Pinning", "Jitter" ve "Proxy Gateway" kullanır.
2.  **Survival (Hayatta Kalma):** iOS'un acımasız arka plan kısıtlamalarına (30sn Limit) karşı "Chunked Write" ve "SLC Wake-up" mekanizmalarıyla çalışır.
3.  **Hız (Velocity):** WatermelonDB ve xxHash Delta Check ile milisaniyeler içinde veri sunar.

## Temel Özellikler
- **Not & Duyuru Takibi:** Anlık bildirimler.
- **Yemekhane Menüsü:** Offline erişim ve QR-Sync ile internetsiz paylaşım.
- **Akademik Takvim & Ders Programı:** Kişiselleştirilmiş görünüm.
- **Gölge İletişim:** Öğrenciler arası anonim (veya yarı-anonim) haberleşme.

## Başarı Kriterleri
- **Fake Traffic:** Sunucuya giden isteklerin %100'ü gerçek browser davranışı (Client Hints uyumlu) sergilemeli.
- **Offline UX:** Kullanıcı internet yokken bile son verileri "Loading" görmeden görebilmeli.
- **Zero-Bug:** Arka plan işlemleri asla veritabanını bozmamalı (Atomic Transactions).
