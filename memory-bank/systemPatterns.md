# System Patterns & Architecture

## Architecture: Shadow Multi-Cloud Pipeline
Sistem, hoca ve öğrenci arasındaki veriyi "Stateless" (Sunucusuz) bir şekilde taşır.

```mermaid
graph TD
    subgraph Client_Side [Teacher/Student Browser]
        SWS[Next.js App]
        Recorder[MediaRecorder API]
        Composer[Canvas Composer]
        Player[Shadow Player]
    end

    subgraph Edge_Infrastructure [Cloudflare]
        Middleware[Next.js Middleware (Auth/Bot Block)]
        Workers[CF Workers (Sync Logic)]
        R2[Cloudflare R2 (Primary Stream)]
    end

    subgraph Cold_Storage [Google Drive]
        Archive[Long-term Lesson Archive]
    end

    SWS --> Composer
    Composer --> Recorder
    Recorder -- "Stream (Egress Free)" --> R2
    R2 -- "Sync" --> Archive
    R2 -- "HLS/Direct" --> Player
```

## Critical Patterns
1.  **Feature-Sliced Design (FSD):** Proje `src/features` (işlev), `src/entities` (veri yapıları) ve `src/shared` (bileşenler) olarak bölünür. Spagetti kod engellenir.
2.  **i18n-Driven Development:** Tüm UI metinleri `next-intl` üzerinden geçer.
3.  **Audio Worklet Priority:** Ses yakalama işlemi (`Processor.js`) ana UI thread'inden ayrılarak işletim sistemi düzeyinde kesintisiz hale getirilir.
4.  **Managed Windows:** Debsis, `window.open` (controlled pop-up) üzerinden izole edilir, ana stüdyo ekranı (Admin) bağımsız kalır.

## Implementation Rules
-   **No Hardcoding:** API URL'leri, renkler ve stringler asla kodun içinde gömülü olmaz.
-   **Type-Driven:** Tüm veri alışverişi `zod` şemaları veya TypeScript Interface'leri ile tanımlanır.
-   **Atomic Changes:** Her commit/faz roadmap'teki tek bir maddeye karşılık gelir.
