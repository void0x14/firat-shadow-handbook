# Project Brief: Operasyon Gölge Vekil (Fırat Shadow Handbook)

## Proje Tanımı
Fırat Üniversitesi öğrencilerinin OBS (Öğrenci Bilgi Sistemi) ve **DEBSIS (Uzaktan Eğitim Sistemi)** verilerine, üniversitenin sunduğu hantal ve mobil uyumsuz arayüzler yerine; modern, hızlı, offline-first ve dağıtık bir yapı üzerinden erişmesini sağlayan "Gölge" bir mobil uygulamadır.

## Temel Felsefe
- **Gölge Gibi:** Üniversite sunucularında (OBS & DEBSIS) iz bırakmadan (Stealth Mode), sanki gerçek bir kullanıcıymış gibi davranır.
- **Yok Edilemez:** IP banlansa, ana sistemler (DEBSIS vb.) çökse bile yedeklenmiş verilerle 7/24 çalışmaya devam eder (Extreme Resilience).
- **Otonom:** Kullanıcı sormadan notları, ödevleri ve ders kayıtlarını arka planda günceller/yedekler (Background Archiving).

## Kritik Hedefler
1.  **Evasion (Gizlilik):** BİDB (Bilgi İşlem) radarından kaçınmak için "Smart UA Pinning", "Jitter" ve "Proxy Gateway" kullanır.
2.  **Survival (Hayatta Kalma):** iOS'un acımasız arka plan kısıtlamalarına karşı "Chunked Write" ve "SLC Wake-up" mekanizmalarıyla çalışır.
3.  **Redundancy (Yedeklilik):** DEBSIS çöktüğünde bile ders materyallerine ve kayıtlarına erişim sağlayan "Shadow Cache" mekanizması.
4.  **Integration (Tam Entegrasyon):** Canlı derslere (Blackboard Collaborate) uygulama içinden katılım ve kayıtların doğrudan indirilmesi.

## Temel Özellikler
- **Not & Duyuru Takibi:** Anlık bildirimler.
- **DEBSIS Yönetimi:** Ödev takibi/gönderimi ve ders materyallerine offline erişim.
- **Canlı Ders Entegrasyonu:** Uygulama içerisinden kesintisiz canlı derse katılım ve etkileşim.
- **Yemekhane Menüsü:** Offline erişim ve QR-Sync ile internetsiz paylaşım.
- **Akademik Takvim & Ders Programı:** Kişiselleştirilmiş görünüm.
- **Gölge İletişim:** Öğrenciler arası anonim (veya yarı-anonim) haberleşme.

## Başarı Kriterleri
- **Fake Traffic:** Sunucuya giden isteklerin %100'ü gerçek browser davranışı (Client Hints uyumlu) sergilemeli.
- **Offline UX:** Kullanıcı internet yokken bile son verileri "Loading" görmeden görebilmeli.
- **Zero-Bug:** Arka plan işlemleri asla veritabanını bozmamalı (Atomic Transactions).
