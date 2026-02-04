# Active Context

## Şu Anki Durum
Detaylı "Savaş Planı" (Architecture Planning) tamamlandı. Proje, teknik risklerden arındırılmış (De-risked) bir şekilde kodlama aşamasına (Faz 1) hazır.

## Son Alınan Kararlar
1.  **iOS Survival:** SLC (Significant Location Change) ve "Heartbeat" mekanizması ile iOS'un 30sn kısıtlaması aşılacak.
2.  **DB Güvenliği:** WatermelonDB kullanılarak "Chunked Transaction" (Parçalı Yazma) yöntemiyle veri kaybı önlenecek.
3.  **Proxy:** Cloudflare Worker, sadece HTML temizleme ve IP maskeleme için "Acil Durum Kapısı" olarak kullanılacak.
4.  **Delta Scraping:** xxHash algoritması ile HTML değişim kontrolü yapılacak (Hız: <1ms).
5.  **Offline Sharing:** Mesh Network yerine QR-Sync (Gzip+Base45) kullanılacak.

## Aktif Görevler
-   [x] Mimari Planlama (docs/PLAN.md)
-   [ ] **(SIRADAKİ)** Faz 1: Prototip Kurulumu (Expo Init + WebView Orchestrator)
-   [ ] User-Agent Generator Modülü
-   [ ] MMKV & WatermelonDB Kurulumu

## Risk İzleme
-   **Risk:** BİDB, Cloudflare IP bloğunu komple banlayabilir.
    -   *Mitigation:* Residential Proxy Gateway (Smartproxy vb.) entegrasyonu hazır tutulacak.
-   **Risk:** iOS SLC tetiklemesi çok nadir olabilir.
    -   *Mitigation:* Kullanıcıya "Sınav Haftası Modu" (Live Activity) açılarak frekans artırılacak.
