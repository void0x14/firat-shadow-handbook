# Tech Context

## Teknoloji Yığını (Stack)

### Frontend (Mobile)
-   **Framework:** React Native (Expo Managed Workflow).
-   **Language:** TypeScript.
-   **Engine:** `react-native-webview` (Hidden Mode).

### Veri & Depolama
-   **Local DB:** WatermelonDB (SQLite tabanlı, Reactive, Offline-first).
-   **Secure Storage:** `react-native-mmkv` (Encrypted). *Keychain/Keystore sadece encryption key saklar.*
-   **Delta Hashing:** `xxhashjs` (veya JSI binding varsa `react-native-xxhash`).

### Backend (Serverless & Proxy)
-   **Edge:** Cloudflare Workers (HTMLRewriter ile optimizasyon).
-   **Key-Value:** Cloudflare KV (Kill Switch yönetimi).
-   **Push:** Expo Notifications (APNs/FCM).

### Evasion & Security Tools
-   **UA Management:** `react-native-device-info` (Real OS detection).
-   **Jitter:** Sunucu tarafında rastgele gecikme algoritması.
-   **Proxy:** SOCKS5/HTTP Residential Proxy desteği (Worker üzerinden).

## Geliştirme Ortamı
-   **IDE:** Windsurf / VS Code.
-   **Linting:** ESLint + Prettier.
-   **Repo:** Git (Atomic Commits).

## Kritik Kütüphaneler
| Kütüphane | Amaç | Neden Seçildi? |
| :--- | :--- | :--- |
| `react-native-background-fetch` | Arka plan yönetimi | iOS `beginBackgroundTask` wrapper'ı en sağlam olan. |
| `expo-location` | Uyanış Tetikleyici | Geofence ve SLC desteği. |
| `cheerio` | HTML Parser | Hafif ve hızlı DOM manipülasyonu. |
| `pako` | Compression | QR-Sync için Gzip sıkıştırma. |
| `react-native-webrtc` | Canlı Yayın | Shadow Studio için streaming altyapısı. |
| `react-native-record-screen` | Otomatik Kayıt | Derslerin otomatik kaydedilmesi. |
| `react-native-floating-bubble` | Floating Chat | Öğretmenler için overlay chat penceresi. |
