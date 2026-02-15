# Active Context

## Şu Anki Durum
**FAZ 1: ALPHA BUILD - AKTİF**

Expo dev server çalışıyor: `http://localhost:8081`

## Son Alınan Kararlar
1.  **iOS Survival:** SLC (Significant Location Change) ve "Heartbeat" mekanizması ile iOS'un 30sn kısıtlaması aşılacak.
2.  **DB Güvenliği:** WatermelonDB kullanılarak "Chunked Transaction" (Parçalı Yazma) yöntemiyle veri kaybı önlenecek.
3.  **Cookie Yönetimi:** `react-native-nitro-cookies` (JSI-based, 5x faster) kullanılıyor.

## Tamamlanan Görevler (Phase 1)
- [x] Expo TypeScript projesi oluşturuldu
- [x] `react-native-mmkv` (Encryption) entegre edildi
- [x] `react-native-device-info` ile Safe UA Generator yazıldı
- [x] `HiddenWebView` orkestrator komponenti oluşturuldu
- [x] `CookieManager` (nitro-cookies) servisi yazıldı
- [x] `LauncherScreen` ve `VisualMonitor` UI oluşturuldu
- [x] Expo dev server başlatıldı
- [x] Proje genelinde detaylı inceleme (Indexing) ve Mimari Analiz tamamlandı
- [x] Detaylı Stratejik Yol Haritası (ROADMAP.md) oluşturuldu

## Bekleyen Görevler
- [ ] iOS/Android simülatörde test
- [ ] OBS login flow testi
- [ ] Cookie extraction doğrulaması
- [ ] Development build (.apk / .app) oluşturma
- [ ] DEBSIS login flow ve Live Session Bridge mimarisi tasarımı
- [ ] Redundant Storage (Shadow Cache) ve Video Download mekanizması tasarımı

## Git Commit Özeti
| Hash | Mesaj |
|------|-------|
| `b5b10c1` | feat(core): initialize Expo project with SecureStorage and UAGenerator |
| `aadd9c4` | feat(webview): add HiddenWebView orchestrator and CookieManager |
| `1a7fc88` | feat(ui): add LauncherScreen and VisualMonitor components |
| `a485d34` | refactor(cookies): migrate to react-native-nitro-cookies |
| `64bf204` | chore(deps): fix nitro-modules version for nitro-cookies compatibility |
