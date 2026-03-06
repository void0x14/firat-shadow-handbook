# Fırat Shadow Handbook: Kökten Sağlamlaştırma Planı

## Amaç

Bu dokümanın amacı mevcut kod tabanındaki kök sorunları kısa vadeli patch mantığıyla değil, uzun vadeli bakım, hız, güvenilirlik ve genişleyebilirlik açısından kökten çözmektir.

Odak alanları:

- Auth mimarisi ve gerçek session sahipliği
- Spagetti kod yoğunluğu ve modülerlik
- Hardcode string problemi
- Bakım/fix/feature geliştirme standardı
- Dil ve stack seçiminin doğruluğu
- Zero-dependency ilkesinin pragmatik sınırları

---

## 1. Yönetici Özeti

Mevcut sistemde iyi niyetli bir mimari yönelim var:

- Rust backend
- Vanilla JS frontend
- Hexagonal mimari niyeti
- Sıfır veya çok düşük bağımlılık arzusu
- Security-first refleksi

Ama pratikte sistemin kritik kısmı birkaç büyük dosyada toplanmış durumda.

Başlangıç metrikleri:

- Gerçek kaynak dosyası sayısı: 23
- Toplam `.rs` + `.js` satırı: 4670
- İlk 3 dosya toplamı: 2419 satır
- İlk 6 dosya toplamı: 3569 satır
- İlk 6 dosyanın toplam mantık içindeki payı: yaklaşık %76.4

Bu, mimari niyet ile fiili kod organizasyonu arasında ciddi bir fark olduğunu gösterir.

En kritik bulgular:

1. Auth tarafında tek otoriter session üretim akışı yok
2. `main.rs` birden fazla katmanın sorumluluğunu taşıyor
3. `cas_adapter.rs` yalnız adapter değil, transport + parser + policy + util + test paketi gibi davranıyor
4. Frontend tarafında i18n sistemi var ama önemli bir kullanıcı metni yüzeyi hâlâ hardcoded
5. “Zero dependency” iddiası teknik olarak zaten mutlak değil; repo pratikte bazı bağımlılıklar kullanıyor
6. Sorun yalnız uzun dosya değil, sınırların delinmiş olması

Bu dokümanın varsayılan önerisi:

- Rust backend korunur
- Mevcut ürün doğrultusu korunur
- Tam yeniden yazım yapılmaz
- Önce sorumluluk ayrımı yapılır
- Zero-dependency dogması yerine “yüksek kaldıraçlı bağımlılık eşiği” modeli benimsenir
- Auth tek doğru akışa indirilir
- Hardcode string sorunu yönetişim ve otomasyon ile çözülür

---

## 2. Mevcut Durum Analizi

## 2.1 Auth Akışı

Mevcut sistemde iki farklı auth davranışı var:

### A. `/api/login` akışı

Bu akış:

- CAS login sayfasını çekiyor
- `lt` ve `execution` alanlarını parse ediyor
- Credential’ları CAS’a post ediyor
- Redirect zincirini takip ediyor
- Debsis tarafında `MoodleSession` cookie’sini elde etmeye çalışıyor

Bu, gerçek session acquisition niyeti taşıyan doğru akış.

### B. `/api/cas/callback` akışı

Bu akış:

- Ticket doğruluyor
- Ama gerçek Debsis session almıyor
- Bunun yerine rastgele token üretip `MoodleSession` diye set ediyor

Bu, sistem içinde “gerçek session” ile “uydurulmuş session” kavramlarını karıştırıyor.

### Auth kök problem tanımı

Kök problem şu:

> Sistemde tek otoriter session üreticisi yok.

Bunun sonuçları:

- Session state güvenilirliği düşer
- Frontend restore davranışı yanlış pozitif üretebilir
- Sonraki feature’larda auth bug’ları büyür
- Güvenlik ve debug maliyeti artar

---

## 2.2 Spagetti ve Modülerlik Analizi

Spagetti burada sadece “dosya çok uzun” demek değil.

Asıl problem:

- Bir dosya birden çok katmanın işini yapıyor
- Aynı dosyada bootstrap, policy, parsing, endpoint, utility, test birikiyor
- Değişiklik yapılan yer ile etkilediği yer arasında net sınır yok
- Dosya boyutları doğal büyümeden değil, sorumluluk yığılmasından şişiyor

### Yüksek riskli yoğunlaşma noktaları

#### `src/main.rs`

Yaptığı işler:

- server bootstrap
- thread pool
- request parsing
- response writing
- rate limiting
- security headers
- static file serving
- route registration
- auth handler
- collab handler
- utility helpers
- testler

Bu dosya uygulamanın “entrypoint”i değil, mini framework gibi davranıyor.

#### `src/infrastructure/cas_adapter.rs`

Yaptığı işler:

- raw TLS transport
- raw HTTP request/response handling
- redirect resolution policy
- cookie handling
- CAS HTML hidden field parsing
- CAS XML parsing
- session validation policy
- test fixture ve testler

Bu dosya adapter sınırını geçmiş durumda.

#### `web/js/app.js`

Yaptığı işler:

- app bootstrap
- router orchestration
- session restore
- auth state
- layout davranışı
- route rendering
- role-based UI
- logout
- bazı ekran üretimleri

Bu dosya shell olmalıydı, ama feature controller’a dönüşmüş.

#### `web/js/components.js`

Yaptığı işler:

- login page markup
- login event flow
- modal descriptors
- UI copy
- hardcoded strings
- bileşen event binding

Bu dosya shared UI + feature UI + copy registry karışımı gibi davranıyor.

---

## 2.3 Spagetti Oranı

Bu projede “spagetti oranı” için resmi bir standart metrik yok. Bu yüzden çalışabilir bir ekip metriği tanımlanmalı.

Önerilen başlangıç değerlendirmesi:

### Spagetti Oranı v1

Bileşenler:

- Dosya yoğunlaşma oranı
- Sorumluluk karışma oranı
- Katman sınırı ihlal oranı
- Hardcoded behavior oranı
- Değişiklik yayılma riski

### Başlangıç yorumu

- Dosya yoğunlaşması: yüksek
- Katman karışması: yüksek
- Modüler sınır açıklığı: orta-alt
- Refactor ihtiyacı: yüksek
- Spagetti oranı başlangıç skoru: %68-%75 bandı

Bu kaba ama faydalı bir başlangıç referansıdır.

Hedef:

- İlk büyük refactor sonunda %45 altı
- Orta vadede %30-%35 bandı
- Uzun vadede %25 altı

---

## 2.4 Hardcode String Problemi

Bu problem LLM kaynaklı değil, sistem tasarımı kaynaklıdır.

LLM’ler neden hardcode üretir:

- Çünkü kod tabanı onlara nerede string tutulacağını net söylemiyor
- Çünkü tek zorunlu string ownership modeli yok
- Çünkü lint/test/CI bunun önünü kesmiyor
- Çünkü geliştirici için en kısa yol raw string yazmak

### Mevcut örnek yüzeyler

Frontend:

- login başlıkları
- açıklamalar
- form label’ları
- button text
- hata mesajları
- confirm metinleri
- modal mode açıklamaları

Backend:

- error response mesajları
- header isimleri
- cookie formatları
- route sabitleri
- güvenlik header içerikleri

### Kök problem tanımı

> String’lerin sahibi belli değil.

Bu çözülmeden hardcode sorunu çözülmez.

---

## 2.5 Dil ve Stack Seçimi

## Rust seçimi doğru mu?

### Güçlü tarafları

- CAS/TLS/HTTP akışını düşük seviyede kontrol etmek için güçlü
- Performans ve güvenlik açısından sağlam
- Single-binary dağıtım için uygun
- Uzun vadede kritik backend çekirdeği için iyi aday

### Zayıf tarafları

- Pure `std::net` düzeyinde custom HTTP sunucusu yazmak bakım maliyetini büyütür
- Her low-level karar ekip yükünü artırır
- Reverse engineering için mantıklı olan seviye, ürün delivery için her zaman mantıklı olmayabilir

### Karar

Rust backend çekirdek için doğru sayılabilir. Ama mevcut problem Rust değil; Rust etrafında çok fazla şeyi elle inşa etme tercihi.

## Vanilla JS seçimi doğru mu?

### Güçlü tarafları

- Düşük kurulum maliyeti
- Build zinciri olmadan hızlı başlangıç
- Küçük ve stabil UI yüzeyi için yeterli olabilir

### Zayıf tarafları

- UI büyüdükçe state, copy, feature boundaries ve event orchestration kolayca dağılır
- Büyük app shell + feature karışımında bakım maliyeti yükselir
- Kod standardı yoksa LLM/insan ikisi de hızlıca string ve logic yığar

### Karar

Vanilla JS küçük ve kontrollü yüzeyde mantıklıdır. Ama bu repo artık “küçük statik arayüz” seviyesini geçmiş durumda.

## Hibrit yaklaşım gerekir mi?

Evet, tartışmaya açık.

En güçlü hibrit aday şudur:

- Backend çekirdek Rust kalır
- Frontend ya daha sert modüler vanilla yapıya geçer ya da daha üretken tipli bir katmana geçer
- Ama bu geçiş yalnızca ilk refactor sonrası hâlâ büyük sürtünme varsa yapılır

---

## 3. Temel Mimari Prensipler

Bu planın dayandığı prensipler:

1. Tek doğru auth akışı
2. Tek dosya, tek sorumluluk kümesi
3. String sahipliği yazılı olmalı
4. Production kodu ile test yardımcıları ayrılmalı
5. Yeni feature var olan god file’ı büyütmemeli
6. Sınır ihlali teknik borç olarak ölçülmeli
7. Bağımlılık reddi dini değil ekonomik karar olmalı
8. Refactor başarı kriteri “daha güzel görünmesi” değil, bakım maliyetinin düşmesi olmalı

---

## 4. Fazlı Dönüşüm Planı

## Faz 0: Başarı Metriklerini Kilitle

Bu fazın amacı kod yazmak değil, neyin başarılı sayılacağını belirlemek.

### Hedef kalite sınırları

- Production dosyası hedef üst sınır: 300 LOC
- Uyarı eşiği: 250 LOC
- Tek dosyada en fazla 1 ana sorumluluk kümesi
- İlk %20 dosyada toplanan logic hedefte %45 altına inmeli
- User-facing hardcoded string oranı %5 altına inmeli
- Auth için tek session sahibi olmalı

### Ölçümler

- Spagetti oranı v1
- Modülerlik skoru v1
- String governance skoru v1
- Feature başına değişen dosya sayısı
- Bugfix başına etkilenen bounded module sayısı

---

## Faz 1: Auth’ı Tek Doğru Akışa İndir

### Hedef

Gerçek session yalnız tek yerden üretilebilsin.

### Varsayılan karar

- `/api/login` ana akış kalsın
- `/api/cas/callback` sahte session üreten yol olarak üretimden çıkarılsın
- Callback yalnız gerçekten gerekli ise sonra tekrar tasarlansın

### Yapılacaklar

- Auth controller katmanını ayır
- Session issuance mantığını ayrı servis yap
- Cookie set etme işini ayrı policy katmanına taşı
- Session validation tek policy üzerinden yürüsün
- Frontend fallback auth restore davranışı sertleştirilsin

### İki alternatif

#### Varsayılan alternatif

Callback kapatılır, yalnız server-side login kalır.

#### İkinci alternatif

Callback gerçek ticket sonrası gerçek Debsis session exchange yapan tam akışa dönüştürülür.

Bu ikinci yol ancak product tarafında tarayıcı yönlendirmeli deneyim zorunluysa seçilmeli.

### Faz 1 başarı kriterleri

- Rastgele `MoodleSession` üretimi yok
- Session restore fake auth state açmıyor
- Auth bug’larının kökü tek akış içinde izlenebiliyor
- Debug kolaylaşıyor

---

## Faz 2: `main.rs` ve `app.js` God File Yapısını Kır

## Backend hedef ayrımı

### `main.rs` içinde kalması gerekenler

- bootstrap
- config load
- server start
- dependency wiring start

### Ayrılması gerekenler

- request parse
- response write
- route registry
- auth handlers
- collab handlers
- static serving
- security middleware benzeri kontroller
- cookie policy
- helper functions
- tests

### Önerilen klasörleşme

- `src/server/`
- `src/http_api/`
- `src/security/`
- `src/static_assets/`
- `src/auth/`
- `src/shared/`

## Frontend hedef ayrımı

### `app.js` içinde kalması gerekenler

- app bootstrap
- high-level orchestration
- shell init

### Ayrılması gerekenler

- auth state
- session restore policy
- role visibility logic
- layout event logic
- page-specific rendering
- logout behavior
- error mapping

### Önerilen ayrım

- `web/js/core/`
- `web/js/features/auth/`
- `web/js/features/layout/`
- `web/js/features/sazan/`
- `web/js/shared/ui/`
- `web/js/shared/services/`

### Faz 2 başarı kriterleri

- `main.rs` 200-300 LOC bandına iner
- `app.js` shell dosyasına dönüşür
- `components.js` feature bazlı ayrılır
- Yeni feature eklemek için god file açma ihtiyacı biter

---

## Faz 3: CAS ve Scraper Adapter’larını Gerçek Adapter Seviyesine İndir

## CAS adapter için hedef

`cas_adapter.rs` yalnız adapter koordinasyonu yapsın.

### Ayrılması gereken alt parçalar

- TLS transport
- raw HTTP request builder
- raw HTTP response parser
- redirect policy
- cookie jar mantığı
- HTML hidden field parser
- CAS XML parser
- URL utils

### Neden?

Şu an adapter dosyası bir “auth subsystem monolith” gibi davranıyor.

## Collab scraper için hedef

`collab_scraper_adapter.rs` içinde:

- HTML scanning
- parsing heuristics
- URL allowlist policy
- text extraction
- entity decode

bunlar birbirinden ayrılmalı.

### Önerilen ayrım

- parser helpers
- extraction helpers
- validation policy
- collab domain mapping

### Faz 3 başarı kriterleri

- Adapter dosyaları orchestration odaklı olur
- Parser’lar ayrı testlenebilir hale gelir
- Policy değişiklikleri business akıştan ayrılır

---

## Faz 4: Hardcode String Sorununu Kökten Bitir

## Hedef

LLM veya insan fark etmeksizin yanlış yere string yazılamasın.

## String sahipliği modeli

### Tür 1: UI metinleri

Sahibi: i18n katalogları

### Tür 2: API response ve kullanıcıya açık hata metinleri

Sahibi: response message registry

### Tür 3: Teknik sabitler

Sahibi: constants modülleri

Örnekler:

- route path
- host
- cookie adı
- header adı
- content type
- security header value

### Tür 4: Log/diagnostic metinleri

Sahibi: logging katmanı

## Frontend kuralları

- User-facing tüm metinler `t(...)` üzerinden gelir
- Component template içinde raw Türkçe/İngilizce metin yasak
- Modal seçenekleri bile catalogue-driven olur
- Confirm mesajları da i18n katmanına taşınır

## Backend kuralları

- JSON error response’lar typed helper ile üretilir
- Cookie adları sabit modülden gelir
- Host ve route sabitleri merkezi olur
- Header içerikleri ayrı policy/constant katmanında tutulur

## Otomasyon

Bu sorunu kökten çözmenin tek yolu otomasyondur.

### Gerekli guardrail’ler

- User-facing raw string detector
- Whitelist tabanlı teknik string istisna listesi
- CI fail kuralı
- Review checklist
- LLM prompt standardı

### LLM prompt standardı

Gelecekte herhangi bir LLM’e şu kural zorunlu verilmeli:

- Yeni kullanıcı metni doğrudan koda yazma
- Yeni route/cookie/header string’ini constants modülü dışında tanımlama
- Yeni feature eklerken önce mevcut ownership pattern’ini ara
- Ownership pattern yoksa önce onu oluştur, sonra feature yaz

### Faz 4 başarı kriterleri

- Login ekranı tamamen katalog tabanlı olur
- Confirm ve toast metinleri katalog tabanlı olur
- Yeni hardcoded UI string’ler CI’da fail edilir
- API response metinleri helper dışından üretilmez

---

## Faz 5: Bakım, Fix ve Feature Geliştirme İçin İdeal Standart

## İdeal ne olmalı?

### Bug fix

- Tek bounded module içinde çözülmeli
- En fazla birkaç dosyaya dokunmalı
- Fix için unrelated katmanlara girme zorunluluğu olmamalı

### Yeni feature

- Bir vertical slice olarak eklenmeli
- Mevcut god file büyütmemeli
- UI metinleri, domain contract, handler ve service ayrık ilerlemeli

### Refactor

- Behavior-preserving olmalı
- “daha şık oldu” değil “değişiklik yayılımı azaldı” diye ölçülmeli

### Review standardı

Review’de şu sorular sorulmalı:

- Bu değişiklik yeni sorumluluk karışması üretti mi?
- Yeni string yanlış ownership katmanında mı?
- Yeni logic mevcut monolith dosyayı büyüttü mü?
- Test production wiring dosyasına mı sızdı?
- Yeni feature bir bounded slice açtı mı?

---

## Faz 6: Zero-Dependency İlkesini Yeniden Tanımla

Mevcut durumda proje zaten mutlak zero-dependency değil.

Kullanılan örnek bağımlılıklar:

- `serde`
- `serde_json`
- `thiserror`
- `chrono`
- `rustls`
- `webpki-roots`

Dolayısıyla doğru soru şu değildir:

> bağımlılık var mı yok mu?

Doğru soru şudur:

> Bu bağımlılık bize aylar seviyesinde kalıcı hız ve güvenilirlik kazandırıyor mu?

## Yeni bağımlılık kabul kuralı

Bir bağımlılık ancak şu durumda alınmalı:

- En az haftalar değil aylar seviyesinde toplam iş yükü azaltıyorsa
- Kod standardını otomatikleştiriyorsa
- Güvenilir, kararlı ve dar kapsamlıysa
- Ayrılabilirliği varsa
- Sahiplik netse

## Reddedilmesi gereken bağımlılıklar

- Sırf daha modern görünüyor diye alınanlar
- Küçük hız kazancı için büyük yüzey açanlar
- Ekosistem/upgrade borcu yüksek olanlar
- Ekipte kimsenin gerçekten sahiplenmeyeceği araçlar

---

## Faz 7: Stack Karar Ağacı

## Varsayılan yol

- Backend Rust kalır
- İlk büyük iş: modülerlik refactor’u
- Frontend ilk dalgada daha iyi ayrıştırılmış vanilla kalabilir
- Bağımlılık çıtası çok yüksek tutulur

## Hibrit yol ne zaman mantıklı olur

Aşağıdaki koşullardan en az ikisi sürerse:

- Frontend tarafında state/copy/bileşen sınırları tekrar tekrar dağılıyorsa
- Yeni ekranlar yine `app.js` ve `components.js` çevresinde şişiyorsa
- Auth/realtime gibi alanlarda type safety eksikliği sürekli bug üretiyorsa
- Onboarding ve review maliyeti kabul edilemez düzeyde yüksekse

Bu durumda:

- Rust backend çekirdek korunur
- Frontend daha üretken ama kontrol edilebilir bir yapıya taşınabilir

## Farklı backend dili ne zaman düşünülür

Ancak şu koşullarda:

- Rust low-level avantajı ürün değerine dönmüyorsa
- Custom HTTP/TLS yükü ürün delivery’yi boğuyorsa
- Reverse engineering bölümü stabilize olup asıl sorun artık ürün iterasyonu haline geldiyse

Bu şu an için birincil öneri değildir.

---

## 5. Uygulama Öncelik Sırası

Önerilen sıra:

1. Faz 0 ölçümleri ve guardrail hedefleri
2. Faz 1 auth tek akış
3. Faz 2 god file parçalama
4. Faz 3 adapter sadeleştirme
5. Faz 4 anti-hardcode governance
6. Faz 5 delivery standardı
7. Faz 6 dependency policy
8. Faz 7 stack yeniden değerlendirme

---

## 6. Nihai Başarı Tanımı

Bu dönüşüm başarılı sayılacaksa:

- Auth sistemi tek doğru session akışına sahip olacak
- Rastgele/fake `MoodleSession` davranışı kalmayacak
- `main.rs` ve `app.js` shell seviyesine inecek
- Adapter’lar gerçek adapter davranışı gösterecek
- UI string’leri katalog tabanlı olacak
- Yeni raw kullanıcı metni CI tarafından engellenecek
- Yeni feature eklemek artık korku değil rutin olacak
- Spagetti oranı belirgin biçimde düşecek
- Modülerlik skoru gözle görülür artacak

---

## 7. Son Karar

Bu proje için en doğru yaklaşım:

- Mevcut sistemi çöpe atmak değil
- Ama mevcut organizasyonu kutsamak da değil
- Rust çekirdeği koruyup mimari sınırları sertleştirmek
- Hardcode ve spagettiyi “insan hatası” değil “sistem tasarımı açığı” olarak görmek
- Gerekirse çok seçici bağımlılık kabul ederek bakım ve hız kazancını maksimize etmek

En kritik zihinsel dönüşüm şudur:

> “Zero dependency” hedef değil, araçtır.  
> “Bakım maliyeti düşük, güvenilir, hızlı evrilen sistem” hedeftir.
