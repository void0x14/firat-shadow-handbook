# Fırat Shadow Handbook - Stratejik Yol Haritası (Roadmap)

Bu belge, **Operasyon Gölge Vekil** kod adlı projenin teknik ve özellik bazlı gelişim sürecini detaylandırır. Her adım, projenin "Stealth" ve "Resilience" felsefesine uygun olarak tasarlanmıştır.

---

## 🟢 Faz 1: Temel Kimlik ve Prototip (Mevcut Durum)
*Hedef: OBS verilerine erişim sağlamak ve kimlik bilgilerini güvenli bir şekilde saklamak.*

- [x] **Proje Altyapısı (Init):** Expo Managed Workflow ile TypeScript tabanlı modern bir React Native yapısı.
- [x] **Güvenli Depolama (The Vault):** `react-native-mmkv` ile JSI tabanlı, şifreli (AES-256) veri saklama.
- [x] **Görünmezlik (UA Generator):** Gerçek cihaz verilerini kullanarak BİDB radarından kaçan `Safe User-Agent` üretimi.
- [x] **Gölge Orkestratör (Hidden WebView):** Kullanıcının görmediği, arka planda login işlemlerini yürüten WebView bileşeni.
- [x] **Nitro Cookie Entegrasyonu:** `react-native-nitro-cookies` ile 5 kat daha hızlı, JSI tabanlı cookie yönetimi ve WebKit senkronizasyonu.
- [ ] **Simülatör/Cihaz Doğrulaması:** Yazılan servislerin gerçek bir cihazda (veya simülatörde) `ASP.NET_SessionId` ve `.ASPXAUTH` değerlerini başarıyla çektiğinin kanıtlanması.
- [ ] **OBS Login Akışı:** WebView üzerinden kullanıcı girişinin ardından otomatik cookie yakalama ve `SecureStorage`'a aktarım.

---

## 🟡 Faz 2: Hayatta Kalma ve Ağ Stratejisi (Planlanan)
*Hedef: iOS/Android kısıtlamalarına rağmen arka planda veri çekmeye devam etmek.*

- [ ] **SLC (Significant Location Change) Wake-up:** iOS'un 30 saniyelik arka plan limitini aşmak için lokasyon bazlı uyanma tetikleyicisi.
- [ ] **Cloudflare Worker Edge Proxy:** 
    - HTML verilerini istemciye göndermeden önce sunucu tarafında temizleme.
    - IP ban riskine karşı konut tipi (Residential) proxy desteği.
- [ ] **Kill-Switch (KV):** Uygulamanın tehlike anında (tespit edilme vb.) tüm ağ trafiğini durdurmasını sağlayan uzaktan komuta mekanizması.
- [ ] **Telemetry (Heartbeat):** Uygulamanın arka planda ne sıklıkla uyandığını ve başarı oranını takip eden anonim log sistemi.

---

## 🔵 Faz 3: Veri Motoru ve Senkronizasyon (Planlanan)
*Hedef: Verileri "milisaniyeler" içinde sunmak ve offline-first deneyimi mükemmelleştirmek.*

- [ ] **WatermelonDB Database Setup:** SQLite tabanlı, reaktif ve devasa veri setlerinde bile takılmayan veritabanı kurulumu.
- [ ] **xxHash Delta Check:** Sadece değişen verileri çekmek ve batarya tüketimini azaltmak için kullanılan hızlı hash kontrolü.
- [ ] **Chunked Write Logic:** Veritabanına yazma işlemleri sırasında uygulama kapanırsa veri bozulmasını önleyen atomik işlem (Transaction) yapısı.
- [ ] **Janitor Service:** Eski duyuru ve önbelleğe alınmış verilerin otomatik temizlenmesi.

---

## 🟠 Faz 4: Özellikler ve Kullanıcı Deneyimi (Planlanan)
*Hedef: Öğrencinin asıl kullanacağı araçları şık bir arayüzle sunmak.*

- [ ] **Dashboard:** Not ortalaması, bekleyen ödevler ve son duyuruların "Rich Design" ile gösterilmesi.
- [ ] **Akademik Takvim & Ders Programı:** Offline erişilebilen ve cihaz takvimiyle senkronize olabilen görünümler.
- [ ] **QR-Sync:** İnterneti olmayan bir arkadaşına ders programını veya notlarını tek bir QR kod ile (offline p2p) aktarma.
- [ ] **Live Activity (Grade Watch):** Yeni bir not girildiğinde kilit ekranında anlık (Dynamic Island uyumlu) bildirim.

---

## 🟣 Faz 6: The DEBSIS Shadow & Live Class (Yeni)
*Hedef: Uzaktan eğitim sistemini tamamen uygulama içine gömerek kullanıcıyı tarayıcıdan kurtarmak.*

- [ ] **DEBSIS Scraper & Orchestrator:**
    - `debsis.firat.edu.tr` için otomatik login ve cookie extraction.
    - Ders videoları ve dokümanları için offline indirme motoru.
- [ ] **Persistent DEBSIS Archive (7/24 Access):**
    - Sunucu tarafında (Edge/Cloud) ders materyallerinin ve linklerinin hash tabanlı yedeklenmesi.
    - DEBSIS çöktüğünde verilerin "Shadow Cache" üzerinden sunulması.
- [ ] **Video Download Manager:**
    - Canlı ders kayıtlarını (MP4/WebM) doğrudan yakalayıp yerel depolamaya (veya buluta) indirme fonksiyonu.
    - Arka planda indirme desteği ve indirme kuyruğu yönetimi.
- [ ] **Assignment Hub:**
    - Yaklaşan ödevler için push bildirimleri.
    - Uygulama içinden dosya seçimi ve ödev yükleme (Auto-Submit).
- [ ] **Live Session Bridge:**
    - Blackboard Collaborate linklerini yakalayıp uygulama içinden (In-App WebView with full control) canlı derse katılma.
    - Canlı ders sırasında "Shadow Chat" veya hızlı etkileşim butonları.
- [ ] **Unified Schedule:** OBS ve DEBSIS programlarını tek bir takvimde birleştirme.

---

## 🔴 Faz 7: Dağıtım ve Sertleştirme (Gelecek)
*Hedef: Projeyi son kullanıcıya ulaştırmak ve güvenliği maksimize etmek.*

- [ ] **Development Build (.apk/.app) Hazırlığı:** Gerçek cihazlarda test için Expo Dev Client buildleri.
- [ ] **Kod Sıkıştırma (Obfuscation):** Tersine mühendisliğe karşı koruma.
- [ ] **Beta Programı:** Kısıtlı kullanıcı grubu ile stres testi.
- [ ] **CI/CD Pipeline:** Github Actions ile otomatik build ve test süreçleri.

---

**Son Güncelleme:** 2026-02-15
**Durum:** Faz 1 - Aktif Geliştirme
