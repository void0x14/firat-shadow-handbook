# Product Context

## The Problem
Fırat Üniversitesi'nin dijital altyapısı (Debsis/Collab) modern eğitim standartlarının çok gerisindedir:
1.  **Görüntü/Ses Kalitesi:** Collab 720p/480p'ye düşürerek kod yazımını ve sunumları okunulmaz kılıyor.
2.  **Performans Kaybı:** Hoca tek laptopta ekran paylaşımı (Full Screen) yaptığında sistem boğuluyor, ses bozuluyor.
3.  **Veri Kırılganlığı:** Bir sistem arızasında (DB hatası vb.) tüm ders materyalleri kalıcı olarak silinebiliyor.

## The Solution: Shadow Platform

### 1. Shadow Web Studio (The "Master" Platform)
Next.js tabanlı, hem hoca hem öğrenci için tek bir "Gölge Merkez".
-   **Hoca (Studio):** Tüm ekranı 1080p yakalar. Sesi tarayıcı kısıtlamalarına takılmadan (Audio Worklet) işler. Kaydı anlık Cloudflare R2'ye yükler.
-   **Öğrenci (Portal):** Netflix kalitesinde bir arayüzle derslere katılır veya geçmiş kayıtları (Shadow Player) izler.

### 2. Shadow Mobile App
Hareket halindeyken veriye erişim ve anlık bildirimler.
-   **Dashboard:** Notlar, duyurular ve yaklaşan dersler.
-   **Offline Sync:** Veriler bir kere çekilir, sistem çöksede telefonda kalır.

## User Experience Goals
-   **Öğretmen:** "Sanki yerel bir uygulamada ders anlatıyor gibiyim, Collab sekmelerim arka planda beni yavaşlatmıyor."
-   **Öğrenci:** "Hocanın paylaştığı terminal ekranındaki her satırı net görebiliyorum, dersi 2x hızla kesintisiz izliyorum."
-   **Hizmet:** Cloudflare R2 sayesinde bant genişliği maliyeti sıfır, hız maksimum.
