# Project Brief: Fırat Shadow Handbook (Operasyon Gölge Vekil)

## 🎯 Vision
Fırat Üniversitesi'nin (FÜ) hantal ve sorunlu eğitim altyapısını (Debsis/Collab) bypass eden, öğretmenlere 1080p yankısız ders anlatma, öğrencilere ise modern ve kesintisiz eğitim alma imkanı sunan **"Shadow Platform"** ekosistemidir.

## 🔑 Core Philosophy
1.  **Stealth (Görünmezlik):** Resmi sisteme (BİDB) zarar vermez, trafiği taklit ederek varlığını gizler.
2.  **Resilience (Direnç):** Offline-first. Sistemler çökse de dersler ve veriler "Gölge Arşiv"de güvende kalır.
3.  **Stateless Efficiency:** Sunucu tarafında minimum veri tutulur. Hesaplamalar ve kayıtlar tamamen istemci (Client) ve hocanın/okulun bulut alanında (R2/Drive) biter.
4.  **Anti-Spaghetti Architecture:** Kod kalitesinden ödün verilmez. Feature-Sliced Design (FSD) ve i18n ilk günden zorunludur.

## 🛠 Project Scope
-   **Shadow Web Studio (Teacher/Student Platform):** Next.js tabanlı ana portal. Hocalar ders anlatır (1080p Full Screen Capture), öğrenciler "Shadow Player" ile dersleri izler.
-   **Shadow Mobile App (Student):** React Native (Expo) ile ders programı, notlar ve duyurulara erişim.
-   **Storage Pipeline:** Cloudflare R2 (Anlık Akış) -> Google Drive (Kalıcı Arşiv).

## ⚠️ Constraints & Risks
-   **Bandwidth Control:** Öğrenci tarafında veri sömürmeyi önlemek için Cloudflare R2 (Egress Free) kullanımı kritiktir.
-   **Network Throttling:** Tarayıcının arka planda ses/görüntü kısıtlamasını aşmak için Audio Worklet ve aktif sekme yönetimi kullanılır.
