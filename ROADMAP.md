# Fırat Shadow Handbook - Stratejik Yol Haritası (Roadmap)

Bu belge, **Operasyon Gölge Vekil** kod adlı projenin teknik ve özellik bazlı gelişim sürecini detaylandırır. Her adım, projenin "Stealth", "Resilience" ve "Stateless" felsefesine uygun olarak tasarlanmıştır.

> **CRITICAL ARCHITECTURAL DECISIONS:**
> - **Framework:** Next.js (App Router, API Routes, Middleware)
> - **Backend:** Smart Edge (Cloudflare Workers + Next.js API)
> - **Storage:** Cloudflare R2 (Primary Stream), Google Drive (Long-term Archive)
> - **i18n:** `next-intl` (Day 1 Integration via Middleware)
> - **Security:** Edge-based Blocking (GeoIP, Bot detection)

---

## 🚀 FAZ 0: THE SHADOW STUDIO (Web Platform - MVP)
*Hedef: Hoca merkezli, 1080p kayıt yapabilen, yankısız ve sunucusuz eğitim platformunu kurmak.*

### 0.1 Foundation & Architecture (Granular Hygiene)
- [ ] **0.1.1 Project Init:** `create-next-app` ile TypeScript + TailwindCSS v4 kurulumu.
- [ ] **0.1.2 Code Quality:** ESLint, Prettier ve Husky hook'larının aktif edilmesi.
- [ ] **0.1.3 Directory Structure:** Feature-Sliced Design (FSD) yapısının `src/features`, `src/shared`, `src/app` olarak kurulması.
- [ ] **0.1.4 i18n Core Setup:**
    - `next-intl` konfigürasyonu.
    - `messages/tr.json` ve `messages/en.json` oluşturulması.
    - Middleware üzerinden dil algılama ve yönlendirme.
- [ ] **0.1.5 Security Middleware:**
    - `middleware.ts` içinde basit User-Agent filtreleme (BİDB botlarına karşı).
    - Rate Limiting (Basit sayaç) entegrasyonu.

### 0.2 The Studio Engine (Teacher Console)
- [ ] **0.2.1 Media Permissions:** `navigator.mediaDevices.getUserMedia` ile kamera/mikrofon izni alma mantığı (hook).
- [ ] **0.2.2 Full Screen Capture:** `getDisplayMedia` ile tüm ekranı yakalayan servis.
- [ ] **0.2.3 Client-Side Composer:**
    - `<canvas>` üzerinde kamera görüntüsünü ekran paylaşımının üzerine (Picture-in-Picture) bindiren render motoru.
    - Resize ve Drag-Drop desteği (Hoca kamerasını istediği yere koysun).
- [ ] **0.2.4 Audio Worklet:** Ses işlemenin (Noise Gate, Gain) ana thread'den `public/workers/audio-processor.js` dosyasına taşınması.

### 0.3 The Shadow Storage (R2 + Drive)
- [ ] **0.3.1 R2 Client Setup:** `aws-sdk` (S3 uyumlu) ile Cloudflare R2 bağlantısının kurulması.
- [ ] **0.3.2 Resumable Uploader:** Videoyu tarayıcıda 5-10MB'lık parçalara bölen `Blob` yönetimi.
- [ ] **0.3.3 Stream to R2:** Parçaların anlık olarak R2 bucket'ına yüklenmesi (Egress Free).
- [ ] **0.3.4 Async Backup:** R2'ye yüklenen dosyanın gece (veya ders bitimi) Google Drive'a kopyalanması (Cloudflare Worker ile).

### 0.4 The Studio Inteface (Teacher UX)
- [ ] **0.4.1 Studio Layout:** Dashboard ve "Canlı Yayın" ekran tasarımı (Glassmorphism).
- [ ] **0.4.2 Debsis Launcher:** `window.open` ile Debsis'i izole pencerede açan güvenli fonksiyon.
- [ ] **0.4.3 Audio Monitor:** Hocanın kendi ses seviyesini görebileceği görsel barlar.

---

## 🟢 Faz 1: The Student Experience (Web & Mobile)
*Hedef: Shadow Player ile dersleri Netflix kalitesinde izletmek.*

### 1.1 Web Player (Next.js)
- [ ] **1.1.1 R2 Indexer:** R2 bucket'ındaki videoları JSON olarak listeleyen API endpoint.
- [ ] **1.1.2 Shadow Player UI:** Custom video kontrolleri (Hız, İleri/Geri sarma).
- [ ] **1.1.3 Adaptive Streaming:** İnternet hızına göre kalite seçimi (Opsiyonel: HLS Transcoding).

### 1.2 Mobile Bridge (React Native)
- [ ] **1.2.1 WebView Integration:** Web Studio'nun mobil uygulama içinde (gerekirse) gösterilmesi.
- [ ] **1.2.2 Native Player:** Videoları mobilde `expo-av` veya `react-native-video` ile oynatma.
- [ ] **1.2.3 Download Manager:** R2 üzerinden videoyu telefona indirip offline izleme özelliği.

---

## 🟡 Faz 2: Mobile Resilience & Data
*Hedef: Debsis'ten bağımsız veri sürekliliği.*

- [ ] **2.1.1 WatermelonDB Schema:** Ders, Not, Duyuru tablolarının oluşturulması.
- [ ] **2.1.2 Sync Engine:** Web API'den gelen verileri yerel DB'ye yazan servis.
- [ ] **2.1.3 Background Fetch:** iOS/Android arka plan görevleri.

---

## 🔴 Faz 3: Dağıtım & Koruma (Hardening)
*Hedef: Prodüksiyon.*

- [ ] **3.1 Deployment:** Projenin Cloudflare Pages veya Vercel üzerine deploy edilmesi.
- [ ] **3.2 Domain & SSL:** `shadow.firat.edu.tr` (Şaka) -> `fushadow.com` gibi domain bağlanması.
- [ ] **3.3 Security Audit:** Penetrasyon testi.

---

**Son Güncelleme:** 2026-02-18
**Durum:** Faz 0.1 (Architecture & Init)
