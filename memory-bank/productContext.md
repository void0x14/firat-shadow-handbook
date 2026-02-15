# Product Context

## Çözdüğümüz Sorunlar
1.  **OBS Hantallığı:** Orijinal sistemin yavaşlığı, sürekli login istemesi ve mobil uyumsuzluğu.
2.  **DEBSIS Karmaşası:** Uzaktan eğitim derslerine girerken yaşanan yönlendirme (collab) ve mobil arayüz sorunları.
3.  **Sistem Kesintileri (Downtime):** DEBSIS'in sıkça çökmesi nedeniyle ders kayıtlarına erişememe mağduriyeti.
4.  **Ödev Takibi:** Unutulan ödevler ve mobil cihazdan ödev yükleme zorluğu.
5.  **Kaçırılan Notlar:** Öğrencilerin sürekli F5 yapmaktan bıkması ve not açıklandığını geç öğrenmesi.
6.  **İnternet Bağımlılığı:** Kampüste internet/çekim sorunu olduğunda yemek listesine veya programa ulaşılamaması.

## Çözüm Mimarisi: "The Autonomous Shadow"

### 1. Kimlik ve Erişim (Sütun A)
BİDB (Bilgi İşlem) tarafından "Bot" olarak algılanmamak için kurulan savunma hattı.
-   **Smart UA Pinning:** Cihazın gerçek işletim sistemiyle uyumlu, sabitlenmiş User-Agent.
-   **Cloudflare Gateway:** Olası IP engellemelerine karşı Residential Proxy rotası.
-   **Kill Switch:** Acil durumda 10ms içinde tüm filoyu durdurma yeteneği.

### 2. Veri Motoru (Sütun B)
iOS'un kısıtlı arka plan süresinde (30sn) maksimum işi yapan motor.
-   **Gizli WebView:** ASP.NET ViewState'i yöneten, kullanıcıdan gizli tarayıcı.
-   **Heartbeat Telemetry:** iOS'un öldürme süresini analiz eden, privacy-safe takip sistemi.
-   **29-Second Rush:** İşlemi 29. saniyede güvenle sonlandıran "Chunked Scraping" stratejisi.

### 3. Kullanıcı Deneyimi (Sütun C)
-   **Offline-First:** WatermelonDB ile "Sıfır Bekleme" (Zero-Latency).
-   **QR-Sync:** İnternetsiz ortamda Gzip+Base45 QR kodlarıyla menü paylaşımı.
-   **Silent Updates:** Kullanıcı fark etmeden verilerin güncellenmesi.

## Kullanıcı Hikayeleri (Örnek)
-   **Ali (iOS Kullanıcısı):** Uygulamayı kapatıp (Kill) cebine koyuyor. Kampüse girdiğinde (Baz istasyonu değişimi), uygulama sessizce uyanıyor, notları kontrol ediyor. Ali telefonu eline aldığında "Lineer Cebir Notu: AA" bildirimini görüyor.
-   **Ayşe (İnterneti Yok):** Yemekhanede interneti yok. Arkadaşı Fatma'nın telefonundaki QR kodu taratıp bu haftaki menüyü kendi telefonuna indiriyor.
