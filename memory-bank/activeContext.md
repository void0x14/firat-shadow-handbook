# Active Context

## Current Status: Building the Foundation (Faz 0.1)
Şu anda projenin "Shadow Web Studio" (SWS) iskeletini kurma aşamasındayız. Mimari kararlar kesinleşti ve atomik adımlara bölündü.

## Recent Decisions & Changes
1.  **Next.js 15 Requirement:** API Routes ve Server-Side Render avantajları için (SPA yerine) Next.js framework'ü seçildi.
2.  **Cloudflare R2 Integration:** Bant genişliği (Bandwidth) maliyetini sıfırlamak için video akışı R2 üzerinden yapılacak.
3.  **i18n Mandatory:** Sert kodlanmış (hardcoded) stringlerin önüne geçmek için `next-intl` ilk günden kurulacak.
4.  **Full Screen Capture Focus:** Slayt yükleme yerine hocanın tüm ekranını (IDE, Browser vs.) yakalama senaryosu ana odak noktası oldu.
5.  **Clean Code:** Feature-Sliced Design (FSD) ile spagetti kodun önüne geçilecek.

## Active Task: 0.1.1 Project Init
Mevcut `ROADMAP.md` doğrultusunda Next.js kurulumu ve mimari iskeletin oluşturulması bekleniyor.

## Learnings & Insights
-   Hocaların tek laptop kullanımında yaşadığı ses sorunu, tarayıcının arka plan sekme kısıtlamasından (Throttling) kaynaklanıyor. Çözüm: High-Priority Audio Worklets.
-   Veri güvenliği ve BİDB radarı için **Edge Middleware** üzerinden User-Agent ve IP kontrolü yapılacak.
