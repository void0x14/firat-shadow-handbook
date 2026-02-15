# 🚨 MVP AKSIYON PLANI - 16 Şubat 2026, Saat 19:00 Sunumu

**Hazırlanma Tarihi:** 2026-02-16 02:03
**Deadline:** 2026-02-16 19:00 (~17 saat)
**Hedef:** Hocalara (Merve Hoca, Harun Hoca) çalışan bir demo göstermek.

---

## 📊 MEVCUT DURUM ANALİZİ (Kod Taraması Sonuçları)

### Dosya Yapısı (6 kaynak dosya)
| Dosya | Satır | Durum | Sorun |
|-------|-------|-------|-------|
| `App.tsx` | 27 | ✅ Temiz | Kullanılmayan `styles` objesi var |
| `LauncherScreen.tsx` | 309 | ✅ Temiz | Sadece login butonu var, gerçek feature yok |
| `HiddenWebView.tsx` | 515 | ✅ Temiz | OBS login'e yönelik, DEBSIS desteği yok |
| `VisualMonitor.tsx` | 668 | ✅ Temiz | Sadece dev araç, kullanıcıya değer katmıyor |
| `CookieManager.ts` | 797 | ⚠️ Riskli | `react-native-nitro-cookies` JSI modülü, Expo Go'da ÇALIŞMAZ |
| `SecureStorage.ts` | 890 | ⚠️ Riskli | `crypto.getRandomValues` RN'de mevcut olmayabilir |
| `UAGenerator.ts` | 1022 | ✅ Temiz | Fonksiyonel ama şu an gereksiz karmaşıklık |

### TypeScript Derleme: ✅ HATA YOK (`npx tsc --noEmit` başarılı)

### Kritik Sorunlar
1. **Native Module Bağımlılığı:** `react-native-nitro-cookies`, `react-native-mmkv`, `react-native-device-info` hepsi JSI/Native modül. Expo Go'da çalışmaz, `expo run:android` ile dev build gerekir.
2. **Sıfır İşlevsellik:** Uygulama şu an SADECE OBS'ye login butonu gösteriyor. Giriş yapınca ne oluyor? → HİÇBİR ŞEY. Not görüntüleme, ders programı, hiçbir şey yok.
3. **Belge Tutarsızlığı:** `systemPatterns.md` Streamlit/Python mimarisinden bahsediyor ama proje React Native. Mimari belgeleri spagetti.
4. **DEBSIS Desteği Yok:** Projenin ana hedeflerinden biri DEBSIS entegrasyonu ama tek satır DEBSIS kodu yok.
5. **crypto.getRandomValues:** `SecureStorage.ts` içinde React Native'de bulunmayan Web API kullanılıyor.

---

## 🎯 MVP TANIMI: "Bu Akşam Ne Göstereceğiz?"

Hocalar teknik detaylarla ilgilenmez. Şunu görmek isterler:
> "Telefonumdan/bilgisayarımdan girip, öğrenci notlarımı, ders programımı, ödevlerimi görebiliyor muyum?"

### ✅ MVP Kapsamı (YAPILACAKLAR)
1. **OBS Login** → WebView ile giriş (mevcut, çalışır hale getirilecek)
2. **Not Görüntüleme** → Login sonrası notları parse edip güzel bir UI'da gösterme
3. **DEBSIS Login** → DEBSIS'e otomatik giriş
4. **Ders Materyalleri Listesi** → DEBSIS'ten ders listesini çekme
5. **Güzel UI** → Hocayı etkileyecek modern, karanlık tema arayüz

### ❌ MVP Dışı (ŞİMDİLİK YAPILMAYACAKLAR)
- Canlı ders streaming
- Otomatik kayıt
- Floating chat overlay
- Offline-first
- Background sync
- QR-Sync
- Shadow Cache

---

## 🔧 TEKNİK STRATEJİ DEĞİŞİKLİĞİ

### Sorun: Native Module = Build Süreci
Mevcut yapıda `react-native-nitro-cookies`, `react-native-mmkv` gibi native modüller var. Bunlar:
- Expo Go'da çalışmaz
- `expo run:android` ile dev build gerektirir
- Build süreci uzun ve hata riski yüksek

### Çözüm: Hybrid Yaklaşım
1. **Phase A (Hızlı Demo):** Native modülleri geçici olarak devre dışı bırak. `expo-secure-store` (zaten yüklü) ve standart `CookieManager` kullan. Bu sayede Expo Go'da test edilebilir.
2. **Phase B (Gerçek Build):** Demo sonrası native modülleri geri al ve `expo run:android` ile gerçek build yap.

---

## 📋 ADIM ADIM UYGULAMA PLANI

### Adım 1: Bağımlılık Temizliği (30 dk)
- [ ] `SecureStorage.ts` → `crypto.getRandomValues` yerine `expo-crypto` veya basit random kullan
- [ ] `CookieManager.ts` → `react-native-nitro-cookies` yerine `react-native-webview`'in kendi cookie yönetimini kullan (WebView `sharedCookiesEnabled` zaten true)
- [ ] Gereksiz native bağımlılıkları kaldır veya opsiyonel yap

### Adım 2: OBS Login Flow Düzeltme (1 saat)
- [ ] HiddenWebView'ı GÖRÜNÜR hale getir (kullanıcı kendi giriş yapacak)
- [ ] Login başarılı olunca cookie'leri `expo-secure-store` ile sakla
- [ ] Session kontrolü ekle

### Adım 3: OBS Veri Çekme (2 saat)
- [ ] Login sonrası WebView'da notları scrape et
- [ ] `injectedJavaScript` ile DOM'dan veri çek
- [ ] Not listesini React Native'e `postMessage` ile gönder
- [ ] Güzel bir "Notlarım" ekranı oluştur

### Adım 4: DEBSIS Entegrasyonu (2 saat)
- [ ] DEBSIS login URL'ini ekle
- [ ] CAS/JASIG login akışını yönet
- [ ] Ders listesini parse et
- [ ] Materyallere erişim linklerini göster

### Adım 5: Dashboard & UI Polish (2 saat)
- [ ] Ana Dashboard ekranı (karanlık tema, modern)
- [ ] Tab navigation (Notlar, DEBSIS, Profil)
- [ ] Animasyonlar ve geçiş efektleri
- [ ] Hoca'yı etkileyecek "WOW" efekti

### Adım 6: Test & Build (1 saat)
- [ ] Android cihazda test (Expo Go veya dev build)
- [ ] Tüm akışı uçtan uca test et
- [ ] Hataları düzelt

### Adım 7: Sunum Hazırlığı (30 dk)
- [ ] Demo senaryosu hazırla
- [ ] Yedek plan (offline screenshots)
- [ ] Hocaya sorulacak soruları hazırla

---

## ⏰ ZAMAN ÇİZELGESİ

| Saat | Görev | Süre |
|------|-------|------|
| 02:00 - 02:30 | Bağımlılık temizliği & refactor | 30dk |
| 02:30 - 03:30 | OBS Login flow düzeltme | 1s |
| 03:30 - 05:30 | OBS veri çekme + UI | 2s |
| 05:30 - 07:30 | DEBSIS entegrasyonu | 2s |
| 07:30 - 09:30 | Dashboard & UI polish | 2s |
| 09:30 - 10:30 | UYKU / Mola | 1s |
| 10:30 - 12:00 | Test & hata düzeltme | 1.5s |
| 12:00 - 13:00 | Build & cihaz testi | 1s |
| 13:00 - 14:00 | ÖĞLE YEMEĞI | 1s |
| 14:00 - 16:00 | Son düzeltmeler & polish | 2s |
| 16:00 - 17:00 | Sunum hazırlığı | 1s |
| 17:00 - 19:00 | Yedek süre & son kontroller | 2s |

---

## 🎯 BAŞARI KRİTERLERİ

Akşam 19:00'da hocaya şunları gösterebilmeliyiz:
1. ✅ Uygulamayı açınca modern, profesyonel görünen bir ekran
2. ✅ OBS'ye giriş yapabilme
3. ✅ Notları görebilme
4. ✅ DEBSIS'e giriş yapabilme (en azından proof-of-concept)
5. ✅ "Bunu kullanır mısınız?" sorusuna pozitif cevap alabilecek UX

---

**DURUM: BAŞLANACAK**
**ÖNCELİK: KRİTİK**
