# Tech Context

## Technology Stack

### Web Platform (Shadow Studio)
-   **Framework:** Next.js 15 (App Router)
-   **UI:** TailwindCSS v4 (Modern CSS-First)
-   **Auth:** Stateless JWT / Shadow ID
-   **State Management:** Zustand (Client), React Query (Server)
-   **Localization:** `next-intl`
-   **Recording Engine:** Native `MediaRecorder API` + `getDisplayMedia`
-   **Media Processing:** `AudioContext`, `AudioWorklet`, `Canvas API` (Overlay)

### Infrastructure & Storage
-   **hosting:** Vercel or Cloudflare Pages
-   **Video Streaming:** Cloudflare R2 (S3 Compatible, Zero Egress)
-   **Automation:** Cloudflare Workers (Sync SWS to Drive)
-   **Archive:** Google Drive API

### Mobile (Shadow App)
-   **Framework:** Expo SDK 52 (React Native)
-   **DB:** WatermelonDB (Offline Sync)
-   **Storage:** MMKV
-   **Networking:** JSI-based Nitro Cookies

## Development Environment
-   **OS:** Linux (CachyOS)
-   **Language:** Strict TypeScript
-   **Code Reviews:** Anti-spaghetti audits mandatory.
-   **Standards:** i18n support from Day 1.

## Constraints
-   **Chrome Background Throttling:** Aktif olmayan sekmelerin CPU kısıtlaması.
-   **Eduroam/BİDB:** IP ve User-Agent takibi. Edge-level spoofing/proxy gerekli.
-   **Google Drive Quota:** Egress limitleri (Cloudflare R2 bu yüzden zorunlu).
