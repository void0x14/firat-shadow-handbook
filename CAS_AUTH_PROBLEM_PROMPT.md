# Fırat Shadow Handbook - CAS Auth Problemi

## Mevcut Durum
- Rust backend çalışıyor, `127.0.0.1:8080`
- Test login: `curl -X POST http://127.0.0.1:8080/api/login -H "Content-Type: application/json" -d '{"username":"test","password":"test"}'`
- Sonuç: `{"error":"Network error","success":false}`

## Problem 1: CAS'a Ulaşamıyor
Backend, CAS server'a (`https://jasig.firat.edu.tr/cas`) bağlanamıyor. Olası nedenler:
- Firewall engelliyor (port 443 çıkış yok)
- URL yanlış
- Network/Routing sorunu

## Problem 2: Eski Spagetti Kod
- ShadowSession, CSRF-Token, AppSession memory store kullanılıyordu
- F5 atınca session gidiyordu (memory'de tutuluyordu)
- Ben düzelttim: direkt MoodleSession cookie kullanılıyor artık

## Yapılması Gerekenler

### 1. CAS Bağlantısını Test Et
```bash
# CAS server'a direkt erişim test et
curl -v https://jasig.firat.edu.tr/cas/login

# DNS kontrol
nslookup jasig.firat.edu.tr
```

### 2. URL'leri Kontrol Et
Fırat'ın güncel CAS adresi ne? `composition.rs`'daki URL'leri güncelle.

### 3. Alternatif: Frontend Redirect
Frontend'de login olunca direkt Debsis'e yönlendir, biz sadece proxy yapalım:
- `window.location.href = "https://debsis.firat.edu.tr/login/index.php?authCAS=CAS"`
- Return URL olarak kendi uygulamamızı ekle

### 4. Veya: nginx reverse proxy
Sunucu dışarıya çıkamıyorsa, nginx ile CAS proxy yapılandır.

## Dosyalar
- `src/main.rs` - HTTP server, route'lar, handler'lar
- `src/infrastructure/cas_adapter.rs` - CAS authentication logic
- `src/application/composition.rs` - Adapter config (URL'ler burada)
- `web/js/app.js` - Frontend login
- `web/js/components.js` - LoginPage component

## Not
- Veritabanı YOK - biz sadece CAS session'ı proxy'liyoruz
- Frontend hiçbir kullanıcı bilgisi saklamıyor
- Güvenlik: sadece HttpOnly cookie'ler
