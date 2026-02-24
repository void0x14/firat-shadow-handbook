# Fırat Shadow Handbook - Kapsamlı Implementation Plan

Sıfır bağımlılık, profesyonel mimari ile 4 fazlı MVP implementasyonu. **HARD-CODE STRING YASAK** - tüm metinler i18n sistemi üzerinden.

---

## 0. Kritik Mimari Prensipler

| Prensip | Uygulama |
|---------|----------|
| **Zero Hard-code String** | Tüm UI metinleri `data/i18n/tr.json` ve `data/i18n/en.json` dosyalarında |
| **i18n First** | Her component dil değişimine anlık tepki verir |
| **Reactive State** | `Proxy` + `CustomEvent` ile framework'siz reactivity |
| **Hexagonal Architecture** | Domain → Ports → Adapters katmanları |
| **Portable Binary** | Frontend assets Rust binary'sine embed edilir |

---

## 1. i18n Mimarisi (Zero Dependency)

### Dosya Yapısı
```
data/
├── i18n/
│   ├── tr.json          # Türkçe çeviriler
│   └── en.json          # İngilizce çeviriler
└── config.json          # Varsayılan dil ayarı
```

### i18n JSON Formatı
```json
{
  "app": {
    "title": "Fırat Shadow Handbook",
    "loading": "Yükleniyor..."
  },
  "auth": {
    "login": "Giriş Yap",
    "logout": "Çıkış Yap",
    "username": "Kullanıcı Adı",
    "password": "Şifre",
    "loginError": "Giriş başarısız"
  },
  "dashboard": {
    "title": "Kontrol Paneli",
    "todayClasses": "Bugünkü Dersler",
    "noClasses": "Bugün ders yok"
  },
  "recording": {
    "start": "Kaydı Başlat",
    "stop": "Kaydı Durdur",
    "quality": "Kalite"
  },
  "sazan": {
    "title": "Sazan.avi Modu",
    "level1": "Sadece Katılım",
    "level2": "Soru Algılama",
    "level3": "AI Cevaplar",
    "level4": "Tam Otonom"
  },
  "errors": {
    "networkError": "Ağ hatası",
    "authFailed": "Kimlik doğrulama başarısız",
    "recordingFailed": "Kayıt başlatılamadı"
  }
}
```

### Frontend i18n Module (web/js/i18n.js)
```javascript
/**
 * @module i18n
 * Zero-dependency internationalization system
 */

/**
 * @typedef {Object} I18nConfig
 * @property {string} defaultLang - Varsayılan dil
 * @property {string} currentLang - Aktif dil
 * @property {Object} translations - Çeviri objesi
 */

/**
 * @type {I18nConfig}
 */
const i18nState = {
  defaultLang: 'tr',
  currentLang: 'tr',
  translations: {}
};

/**
 * Dil değişikliğinde tetiklenen event
 * @event i18n:languageChanged
 * @type {CustomEvent}
 */

async function loadLanguage(lang) {
  const response = await fetch(`/data/i18n/${lang}.json`);
  i18nState.translations = await response.json();
  i18nState.currentLang = lang;
  document.dispatchEvent(new CustomEvent('i18n:languageChanged', {
    detail: { lang }
  }));
}

/**
 * Çeviri anahtarı çözümle
 * @param {string} key - Nokta notasyonu ile anahtar (örn: "auth.login")
 * @param {Object} [params] - Değişken parametreler
 * @returns {string}
 */
function t(key, params = {}) {
  const keys = key.split('.');
  let value = i18nState.translations;

  for (const k of keys) {
    if (value && value[k]) {
      value = value[k];
    } else {
      console.warn(`i18n: Missing key "${key}"`);
      return key;
    }
  }

  // Parametre değiştirme: {{name}} -> value
  if (typeof value === 'string') {
    return value.replace(/\{\{(\w+)\}\}/g, (_, p) => params[p] || '');
  }

  return value;
}

export { loadLanguage, t, i18nState };
```

### HTML'de Kullanım
```html
<button data-i18n="auth.login"></button>
<span data-i18n="dashboard.title"></span>
```

### Auto-Translate Script
```javascript
// DOM'daki tüm [data-i18n] elementlerini çevir
function translatePage() {
  document.querySelectorAll('[data-i18n]').forEach(el => {
    const key = el.getAttribute('data-i18n');
    el.textContent = t(key);
  });
}

document.addEventListener('i18n:languageChanged', translatePage);
```

---

## 2. Reactive State Management (Proxy + CustomEvent)

### State Module (web/js/state.js)
```javascript
/**
 * @module state
 * Zero-dependency reactive state with Proxy + CustomEvent
 */

/**
 * @template T
 * @param {T} initialState - Başlangıç state'i
 * @param {string} namespace - Event namespace'i
 * @returns {T}
 */
function createStore(initialState, namespace = 'app') {
  const handlers = {
    set(target, prop, value) {
      if (target[prop] === value) return true;
      target[prop] = value;
      document.dispatchEvent(new CustomEvent(`${namespace}:${prop}Changed`, {
        detail: { value, prop }
      }));
      document.dispatchEvent(new CustomEvent(`${namespace}:anyChanged`, {
        detail: { value, prop }
      }));
      return true;
    }
  };

  return new Proxy(initialState, handlers);
}

// Global app state
const appState = createStore({
  user: null,
  isAuthenticated: false,
  currentView: 'login',
  theme: 'dark',
  language: 'tr',
  courses: [],
  recordings: [],
  sazanLevel: 0
}, 'app');

export { createStore, appState };
```

### Component Usage
```javascript
import { appState } from './state.js';
import { t } from './i18n.js';

// State değişikliğini dinle
document.addEventListener('app:isAuthenticatedChanged', (e) => {
  if (e.detail.value) {
    showDashboard();
  } else {
    showLogin();
  }
});

// State'i güncelle
appState.isAuthenticated = true;
```

---

## 3. Dış Servis URL'leri

| Servis | URL | Not |
|--------|-----|-----|
| CAS Login | `https://jasig.firat.edu.tr/cas/login` | TGT/ST ticket |
| Debsis | `https://debsis.firat.edu.tr` | Moodle tabanlı |
| Collab Playback | `https://eu.bbcollab.com/...` | JWT auth token |
| OBS WebSocket | `ws://localhost:4455` | RFC 6455 |

---

## 4. HEXAGONAL ARCHITECTURE (Detaylı)

```
┌────────────────────────────────────────────────────────────────────────┐
│                         INTERFACE LAYER                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │
│  │ HTTP Server  │  │  WebSocket   │  │ Static Files │  │   API     │  │
│  │ (std::net)   │  │  (RFC 6455)  │  │  (embedded)  │  │  Routes   │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └─────┬─────┘  │
└─────────┼──────────────────┼──────────────────┼────────────────┼───────┘
          │                  │                  │                │
┌─────────┴──────────────────┴──────────────────┴────────────────┴───────┐
│                       APPLICATION LAYER (Use Cases)                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │
│  │  LoginUser   │  │ FetchCourses │  │ StartRecord  │  │ AutoJoin  │  │
│  │  UseCase     │  │  UseCase     │  │  UseCase     │  │ UseCase   │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └─────┬─────┘  │
└─────────┼──────────────────┼──────────────────┼────────────────┼───────┘
          │                  │                  │                │
┌─────────┴──────────────────┴──────────────────┴────────────────┴───────┐
│                           DOMAIN LAYER                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌───────────┐    │
│  │   User       │  │   Course     │  │  Recording   │  │  Session  │    │
│  │   Entity     │  │   Entity     │  │   Entity     │  │  Entity   │    │
│  └──────────────┘  └──────────────┘  └──────────────┘  └───────────┘    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                        PORTS (Traits)                           │   │
│  │  AuthPort │ ScraperPort │ StoragePort │ RecorderPort │ WSPort   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────┘
          │                  │                  │                │
┌─────────┴──────────────────┴──────────────────┴────────────────┴───────┐
│                      INFRASTRUCTURE LAYER (Adapters)                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │
│  │ CASAdapter   │  │CollabAdapter │  │FileStorage   │  │OBSAdapter │  │
│  │ (HTTP POST)  │  │(HTML parse)  │  │Adapter       │  │(WebSocket)│  │
│  └──────────────┘  └──────────────┘  └──────────────┘  └───────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 5. TAM DOSYA YAPISI

```
firat-shadow-handbook/
├── Cargo.toml                    # Rust config (zero deps)
├── build.rs                      # Asset embedding
├── src/
│   ├── main.rs                   # Entry point
│   ├── lib.rs                    # Library root
│   ├── config.rs                 # Server config
│   │
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── user.rs               # User entity
│   │   ├── course.rs             # Course entity
│   │   ├── recording.rs          # Recording entity
│   │   ├── session.rs            # Session entity
│   │   └── ports/
│   │       ├── mod.rs
│   │       ├── auth_port.rs      # Auth trait
│   │       ├── scraper_port.rs   # Scraper trait
│   │       ├── storage_port.rs   # Storage trait
│   │       └── recorder_port.rs  # Recorder trait
│   │
│   ├── application/
│   │   ├── mod.rs
│   │   ├── login_usecase.rs
│   │   ├── fetch_courses_usecase.rs
│   │   ├── start_recording_usecase.rs
│   │   └── auto_join_usecase.rs
│   │
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── cas_adapter.rs        # CAS HTTP client
│   │   ├── collab_adapter.rs     # Collab scraper
│   │   ├── file_storage.rs       # JSON file storage
│   │   └── obs_adapter.rs        # OBS WebSocket client
│   │
│   ├── interface/
│   │   ├── mod.rs
│   │   ├── http/
│   │   │   ├── mod.rs
│   │   │   ├── server.rs         # TcpListener server
│   │   │   ├── request.rs        # HTTP/1.1 parser
│   │   │   ├── response.rs       # Response builder
│   │   │   └── router.rs         # Route matching
│   │   ├── websocket/
│   │   │   ├── mod.rs
│   │   │   ├── frame.rs          # WS frame parser
│   │   │   └── handler.rs        # WS handler
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── static_handler.rs  # Serve embedded files
│   │       ├── api_handler.rs    # REST API
│   │       └── ws_handler.rs     # WebSocket upgrade
│   │
│   └── automation/
│       ├── mod.rs
│       ├── scheduler.rs           # Auto-join scheduler
│       └── sazan.rs               # Sazan.avi engine
│
├── web/
│   ├── index.html                 # SPA shell
│   ├── js/
│   │   ├── app.js                 # Entry point
│   │   ├── router.js              # Client-side routing
│   │   ├── state.js               # Proxy-based state
│   │   ├── i18n.js                # Internationalization
│   │   ├── api.js                 # HTTP client
│   │   ├── components/
│   │   │   ├── login.js
│   │   │   ├── dashboard.js
│   │   │   ├── courses.js
│   │   │   ├── recording.js
│   │   │   └── sazan.js
│   │   └── utils/
│   │       ├── dom.js             # DOM helpers
│   │       └── events.js          # Event helpers
│   ├── css/
│   │   ├── variables.css          # Design tokens
│   │   ├── reset.css              # CSS reset
│   │   ├── layout.css             # Grid system
│   │   ├── components.css         # UI components
│   │   └── themes/
│   │       ├── dark.css
│   │       └── light.css
│   └── assets/
│       └── icons/                 # SVG icons
│
├── data/
│   ├── config.json                # Runtime config
│   ├── i18n/
│   │   ├── tr.json                # Turkish
│   │   └── en.json                # English
│   ├── courses.json               # Manual course data
│   └── recordings/                # Recording metadata
│
└── tests/
    ├── http_test.rs
    ├── auth_test.rs
    └── scraper_test.rs
```

---

## 6. FAZ BAZLI İMPLEMENTASYON (24 Saat Hedef)

### FAZ 0 — Core Skeleton (4-6 saat)

| Commit | Dosyalar | Açıklama |
|--------|----------|----------|
| `feat: project skeleton` | `Cargo.toml`, `main.rs`, `lib.rs` | Rust iskeleti |
| `feat: http server` | `http/server.rs`, `request.rs`, `response.rs` | std::net HTTP/1.1 |
| `feat: router` | `router.rs` | Basit route matching |
| `feat: static handler` | `static_handler.rs` | Embedded file serving |
| `feat: i18n system` | `i18n.js`, `tr.json`, `en.json` | Zero-dep i18n |
| `feat: reactive state` | `state.js` | Proxy + CustomEvent |
| `feat: frontend shell` | `index.html`, `app.js`, `variables.css` | SPA iskeleti |
| `feat: login component` | `login.js`, `components.css` | Mock auth UI |

### FAZ 1 — CAS Auth & Scraper (4-6 saat)

| Commit | Dosyalar | Açıklama |
|--------|----------|----------|
| `feat: domain entities` | `user.rs`, `course.rs`, `session.rs` | Domain modelleri |
| `feat: ports traits` | `auth_port.rs`, `scraper_port.rs` | Trait tanımları |
| `feat: CAS adapter` | `cas_adapter.rs` | TGT/ST ticket flow |
| `feat: collab scraper` | `collab_adapter.rs` | HTML parsing |
| `feat: file storage` | `file_storage.rs` | JSON persistence |
| `feat: login usecase` | `login_usecase.rs` | Auth orchestration |
| `feat: api endpoints` | `api_handler.rs` | REST routes |

### FAZ 2 — Live Engine & Media (6-8 saat)

| Commit | Dosyalar | Açıklama |
|--------|----------|----------|
| `feat: websocket frame` | `frame.rs` | RFC 6455 frame parser |
| `feat: ws handler` | `ws_handler.rs`, `handler.rs` | WS connection |
| `feat: OBS adapter` | `obs_adapter.rs` | OBS WebSocket client |
| `feat: recording entity` | `recording.rs` | Recording domain |
| `feat: recording usecase` | `start_recording_usecase.rs` | Recording orchestration |
| `feat: recording UI` | `recording.js` | Recording controls |
| `feat: media recorder` | Client-side MediaRecorder API | Browser recording |

### FAZ 3 — Automation & Deploy (4-6 saat)

| Commit | Dosyalar | Açıklama |
|--------|----------|----------|
| `feat: scheduler` | `scheduler.rs` | Cron-like scheduler |
| `feat: auto-join` | `auto_join_usecase.rs` | Auto class join |
| `feat: sazan engine` | `sazan.rs` | Q&A automation |
| `feat: sazan UI` | `sazan.js` | Sazan.avi panel |
| `feat: build script` | `build.rs` | Asset embedding |
| `feat: portable binary` | Cargo config | Cross-compile |
| `feat: final polish` | All files | QA & cleanup |

---

## 7. ACCEPTANCE CRITERIA (Tüm Fazlar)

### Faz 0
- [ ] `cargo run` → Server 8080'de başlar
- [ ] `curl localhost:8080` → index.html döner
- [ ] Browser → Responsive UI yüklenir
- [ ] Dark/Light mode çalışır
- [ ] Dil değişimi anlık çalışır
- [ ] Mock login flow tamamlanır

### Faz 1
- [ ] CAS login gerçek kimlik bilgileriyle çalışır
- [ ] TGT/ST ticket flow tamamlanır
- [ ] Session persistence çalışır
- [ ] Collab'den video URL çekilir

### Faz 2
- [ ] WebSocket bağlantısı kurulur
- [ ] OBS WebSocket ile iletişim kurulur
- [ ] Yüksek kalite kayıt başlar/durur
- [ ] Client-side MediaRecorder çalışır

### Faz 3
- [ ] Auto-join scheduler çalışır
- [ ] Sazan.avi mod aktif olur
- [ ] Portable binary tek dosya olarak çalışır
- [ ] Cross-platform test edilir

---

## 8. REVERSE ENGINEERING BULGULARI

### 8.1 CAS Login Flow (Detaylı)

**Adım Adım Protokol:**

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CAS AUTH FLOW                                 │
├─────────────────────────────────────────────────────────────────────┤
│  1. GET /cas/login?service={callback_url}                           │
│     → Response: HTML form + JSESSIONID cookie                        │
│     → Hidden fields: lt, execution, _eventId                         │
│                                                                      │
│  2. POST /cas/login                                                  │
│     → Body: username={user}&password={pass}&lt={ticket}             │
│            &execution=e1s1&_eventId=submit                           │
│     → Response: 302 Redirect to service URL with ticket param        │
│                                                                      │
│  3. GET {service_url}?ticket={ST}                                   │
│     → Response: Session cookie (MoodleSession)                       │
│                                                                      │
│  TGT (Ticket Granting Ticket) - Long-lived session ticket            │
│  ST (Service Ticket) - One-time use ticket per service               │
└─────────────────────────────────────────────────────────────────────┘
```

**CAS Login Form Hidden Fields:**
```html
<input type="hidden" name="lt" value="LT-1192339-CEe1mUXRoX5UeqKzQ5m5JV6zkuGpaD" />
<input type="hidden" name="execution" value="e1s1" />
<input type="hidden" name="_eventId" value="submit" />
```

**Rust Implementation:**
```rust
// 1. GET request to CAS login page
let response = http_get("https://jasig.firat.edu.tr/cas/login?service=...")?;
let jsesssionid = extract_cookie(&response, "JSESSIONID");
let lt = extract_hidden_field(&response.body, "lt");
let execution = extract_hidden_field(&response.body, "execution");

// 2. POST login
let body = format!(
    "username={}&password={}&lt={}&execution={}&_eventId=submit",
    username, password, lt, execution
);
let response = http_post("https://jasig.firat.edu.tr/cas/login", body, jsessionid)?;

// 3. Extract ticket from redirect URL
let ticket = extract_ticket_from_redirect(&response);

// 4. Validate ticket at service
let session = http_get(&format!("{}?ticket={}", service_url, ticket))?;
```

### 8.2 OBS WebSocket Protocol (v5)

**Bağlantı Akışı:**
```
┌─────────────────────────────────────────────────────────────────────┐
│                    OBS WEBSOCKET HANDSHAKE                           │
├─────────────────────────────────────────────────────────────────────┤
│  1. WebSocket Connect: ws://localhost:4455                          │
│                                                                      │
│  2. Server → Client (Hello Message):                                │
│     {                                                                │
│       "op": 0,                                                       │
│       "d": {                                                         │
│         "obsWebSocketVersion": "5.0.0",                              │
│         "rpcVersion": 1,                                             │
│         "authentication": {                                          │
│           "challenge": "abc123",                                     │
│           "salt": "def456"                                           │
│         }                                                            │
│       }                                                              │
│     }                                                                │
│                                                                      │
│  3. Client → Server (Identify Message):                             │
│     auth = base64(sha256(base64(sha256(password + salt)) + challenge))│
│     {                                                                │
│       "op": 1,                                                       │
│       "d": {                                                         │
│         "rpcVersion": 1,                                             │
│         "authentication": auth,                                      │
│         "eventSubscriptions": 1000                                    │
│       }                                                              │
│     }                                                                │
│                                                                      │
│  4. Server → Client (Identified Message):                            │
│     { "op": 2, "d": { "negotiatedRpcVersion": 1 } }                  │
└─────────────────────────────────────────────────────────────────────┘
```

**Recording Commands:**
```json
// Start Recording
{ "op": 6, "d": { "requestType": "StartRecord", "requestId": "123" } }

// Stop Recording
{ "op": 6, "d": { "requestType": "StopRecord", "requestId": "124" } }

// Get Recording Status
{ "op": 6, "d": { "requestType": "GetRecordStatus", "requestId": "125" } }

// Set Scene
{ "op": 6, "d": { "requestType": "SetCurrentProgramScene", "requestData": { "sceneName": "Scene1" } } }
```

**Rust Implementation:**
```rust
fn build_auth_string(password: &str, salt: &str, challenge: &str) -> String {
    let secret = base64_encode(sha256(password.to_string() + salt));
    let auth = base64_encode(sha256(secret + challenge.to_string()));
    auth
}

async fn obs_connect(host: &str, port: u16, password: &str) -> WebSocket {
    let ws = websocket_connect(format!("ws://{}:{}", host, port)).await;

    // Receive Hello
    let hello: HelloMessage = ws.receive_json().await;

    // Build auth and send Identify
    let auth = build_auth_string(password, &hello.d.authentication.salt, &hello.d.authentication.challenge);
    ws.send_json(IdentifyMessage {
        op: 1,
        d: IdentifyData {
            rpc_version: 1,
            authentication: Some(auth),
            event_subscriptions: 1000,
        }
    }).await;

    ws
}
```

### 8.3 Collab Video URL Yapısı (DETAYLI REVERSE ENGINEERING)

**URL Format:**
```
https://eu.bbcollab.com/collab/ui/session/playback/load/{recordingId}?authToken={jwt}
```

**JWT Token Yapısı:**
```json
{
  "sub": "bbCollabApi",
  "recordingUId": "7b443521ef5c4ae79f04c0c7f6d0ad55",
  "iss": "bbCollabApi",
  "exp": 1771960998,
  "type": 1,
  "iat": 1771957398,
  "consumer": "1285a4608a834dab8e26439e49af0264"
}
```

**Kritik API Endpoint (YENİ BULGU):**
```
GET https://eu.bbcollab.com/collab/api/csa/recordings/{recordingId}/data/secure
Headers:
  Authorization: Bearer {jwt}
  Accept: application/json
```

**API Response Yapısı (GERÇEK VERİ):**
```json
{
  "status": 3,
  "streams": {
    "WEB": "https://ultra-eu-prod-sms.cloudfront.cdn.bbcollab.com/content/{uuid}/{date}/video.mp4?Expires={ts}&Signature={sig}&Key-Pair-Id=K1WGYLWDGH4IS2"
  },
  "extStreams": [{
    "streamUrl": "...",
    "contentType": "video/mp4",
    "flavorCode": 1
  }],
  "cookies": "https://ultra-eu-prod-sms.cloudfront.cdn.bbcollab.com/v1/cookies/{jwt_base64}?Expires={ts}&Signature={sig}&Key-Pair-Id=...",
  "mediaDownloadUrl": "https://...mp4?cf=disabled&response-content-disposition=attachment...",
  "subtitlesInDownload": true,
  "aspectRatio": "16:9",
  "subtitles": [],
  "chats": [{
    "url": "https://ultra-eu-prod-sms.cloudfront.cdn.bbcollab.com/content/{uuid}/..._chat.json?Expires={ts}&Signature={sig}&Key-Pair-Id=..."
  }],
  "chaptering": { "enabled": false },
  "uuid": "97e1c8f3-c5af-4265-9f26-6d829e9d8541",
  "sessionInstanceUUID": "eacaa4cf-5712-44e6-ae1d-7af8f6334366",
  "profanityFilterEnabled": false,
  "name": "Canlı Ders - recording_1",
  "duration": 4470000,
  "created": "2026-02-24T17:43:33.402Z"
}
```

**CloudFront Signed URL Parametreleri:**
| Parametre | Açıklama | Örnek |
|-----------|----------|-------|
| `Expires` | Unix timestamp (saniye) | `1771972571` |
| `Signature` | AWS CloudFront imzası | `LWWtZJK0DkRAjm~RWuspCFtCBFk3...` |
| `Key-Pair-Id` | AWS Key Pair ID | `K1WGYLWDGH4IS2` |

**Önemli Notlar:**
1. JWT token Debsis/CAS login sonrası Collab sayfasından çekilmeli
2. Token'ın süresi var (`exp` field) - yaklaşık 1 saat geçerli
3. CloudFront URL'ler belirli bir süre için geçerli (Expires parametresi)
4. Chat kayıtları JSON formatında indirilebilir
5. Video MP4 formatında, doğrudan download URL'si mevcut

### 8.4 Debsis/Moodle Cookie Yapısı

**Login Sonrası Cookies:**
- `MoodleSession` - Ana session cookie (örn: `fgir9qe9j44rjgumdlpmo6u2t2`)
- `MOODLEID_` - User ID (optional)
- `JSESSIONID` - CAS session (geçici)

### 8.5 Moodle AJAX Web Service API (YENİ BULGU)

**Endpoint:**
```
POST https://debsis.firat.edu.tr/lib/ajax/service.php?sesskey={SESSKEY}&info={method_name}
```

**Gerekli Headers:**
```
Content-Type: application/json
X-Requested-With: XMLHttpRequest
Cookie: MoodleSession={session}
```

**Request Body Format:**
```json
[
  {
    "index": 0,
    "methodname": "core_course_get_enrolled_courses_by_timeline_classification",
    "args": {
      "offset": 0,
      "limit": 24,
      "classification": "all",
      "sort": "fullname",
      "customfieldname": "",
      "customfieldvalue": ""
    }
  }
]
```

**Mevcut API Methodları:**
| Method | Açıklama | Kullanım |
|--------|----------|----------|
| `core_course_get_enrolled_courses_by_timeline_classification` | Kayıtlı dersleri listeler | Dashboard course cards |
| `core_course_get_recent_courses` | Son erişilen dersler | Recently accessed block |
| `core_calendar_get_calendar_monthly_view` | Aylık takvim görünümü | Calendar widget |
| `core_calendar_get_action_events_by_timesort` | Zaman sıralı etkinlikler | Timeline block |
| `core_message_get_unread_conversations_count` | Okunmamış mesaj sayısı | Message badge |

**Response Örneği (Courses):**
```json
[{
  "error": false,
  "data": {
    "courses": [{
      "id": 240684,
      "fullname": "AIT102 Ataturk Ilkeleri ve Inklap Tarihi II...",
      "shortname": "2099252F26B",
      "idnumber": "2099252F26B",
      "startdate": 1771309200,
      "enddate": 1802845200,
      "visible": true,
      "viewurl": "https://debsis.firat.edu.tr/course/view.php?id=240684",
      "courseimage": "data:image/svg+xml;base64,..."
    }]
  }
}]
```

**sesskey Nasıl Alınır:**
1. Login sonrası her sayfada `<script>` tag içinde `sesskey: "xxx"` şeklinde mevcut
2. Veya HTML'de hidden input: `<input name="sesskey" value="xxx">`
3. Veya JavaScript global: `M.cfg.sesskey`

### 8.6 Debsis UI Element Yapısı (YENİ BULGU)

**Dashboard Block Yapısı:**
```
┌─────────────────────────────────────────────────────────────┐
│  [Header] Fırat Üniversitesi Uzaktan Eğitim Portalı        │
│  [Nav] Derslerim ▼ | Mesajlaşma 🔔 | Bildirimler 🔔         │
├─────────────────────────────────────────────────────────────┤
│  [Breadcrumb] Ana sayfa / Kontrol paneli                    │
│  [Title] FUZEM: Kontrol paneli                              │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│  │ Course Card │ │ Course Card │ │ Course Card │  (Slider)  │
│  │ [Image]     │ │ [Image]     │ │ [Image]     │            │
│  │ Category    │ │ Category    │ │ Category    │            │
│  │ Course Name │ │ Course Name │ │ Course Name │            │
│  └─────────────┘ └─────────────┘ └─────────────┘            │
├─────────────────────────────────────────────────────────────┤
│  [Timeline Block]                                           │
│  │ Yaklaşan etkinlikler                                     │
│  │ • Collab Session 1 - 24 Şubat 18:00                     │
│  │ • Collab Session 2 - 25 Şubat 10:00                     │
│  └─────────────────────────────────────────────────────────│
│  [Calendar Block]                                           │
│  │ ◄ February 2026 ►                                        │
│  │ P S Ç P C C P                                             │
│  │     1 2 3 4 5 6 7                                         │
│  │ ... [16] [17] [18] [19] [20] ...                         │
│  │     •  •   •   •                                          │
│  └─────────────────────────────────────────────────────────│
└─────────────────────────────────────────────────────────────┘
```

**Course Card HTML Yapısı:**
```html
<div class="course-card">
  <a href="/course/view.php?id=240709" class="course-image-link">
    <div class="course-image">...</div>
  </a>
  <div class="course-info">
    <div class="course-category">Bilgisayar Programcılığı(UZAKTAN)</div>
    <a href="/course/view.php?id=240709" class="course-name">
      BTB1104-BTB108 INTERNET PROGRAMCILIGI I...
    </a>
  </div>
</div>
```

**Collab Session Link Yapısı:**
```html
<a href="/mod/collaborate/view.php?id=1467632" class="event-link">
  1. Ders
</a>
<!-- id parametresi Collab session ID'si değil, Moodle mod ID'si -->
```

### 8.7 Collab Player UI Yapısı (YENİ BULGU)

**Player Element Yapısı:**
```
┌─────────────────────────────────────────────────────────────┐
│  [Title] Canlı Ders - recording_1                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│                    [Video Area]                              │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│  [◀◀] [▶ Play] [▶▶] [━━━━━━━━━━━━━━○────] [🔊] [⚡] [CC] [⛶]│
│         Skip    Slider (0:00 / 1:14:30)                      │
├─────────────────────────────────────────────────────────────┤
│  [☰ Panel] → Chat | Attendees | Share | Settings             │
└─────────────────────────────────────────────────────────────┘
```

**Player Controls:**
- Play/Pause button
- Skip back/forward 10 seconds
- Progress slider (value, valuemax, valuemin)
- Volume control (expandable menu)
- Playback speed (0.5x, 1x, 1.5x, 2x)
- Closed Captions toggle
- Fullscreen mode
- Collaborate Panel (chat, attendees, etc.)

---

## 9. ADIM ADIM IMPLEMENTASYON REHBERİ

### Faz 0 - Commit 1: Rust Project Skeleton

```bash
# Dosyalar: Cargo.toml, src/main.rs, src/lib.rs
```

**Cargo.toml:**
```toml
[package]
name = "firat-shadow-handbook"
version = "0.1.0"
edition = "2021"

[dependencies]
# Zero external dependencies - only std library

[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Strip symbols
```

**src/main.rs:**
```rust
mod lib;

fn main() {
    println!("Fırat Shadow Handbook v0.1.0");
    // TODO: Start HTTP server
}
```

### Faz 0 - Commit 2: HTTP Server

**src/interface/http/server.rs:**
```rust
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

pub struct HttpServer {
    address: String,
    port: u16,
}

impl HttpServer {
    pub fn new(port: u16) -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port,
        }
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.address, self.port))
            .expect("Failed to bind");

        println!("Server listening on {}:{}", self.address, self.port);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(|| handle_connection(stream));
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Parse request
    let request = String::from_utf8_lossy(&buffer[..]);
    let (method, path, _headers) = parse_request(&request);

    // Route
    let response = match (method.as_str(), path.as_str()) {
        ("GET", "/") => serve_index(),
        ("GET", path) if path.starts_with("/api/") => handle_api(path),
        _ => not_found(),
    };

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn parse_request(request: &str) -> (String, String, Vec<(String, String)>) {
    let lines: Vec<&str> = request.lines().collect();
    let first_line: Vec<&str> = lines[0].split_whitespace().collect();

    let method = first_line[0].to_string();
    let path = first_line[1].to_string();

    let headers: Vec<(String, String)> = lines[1..]
        .iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect();

    (method, path, headers)
}

fn serve_index() -> String {
    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<h1>Hello</h1>".to_string()
}

fn not_found() -> String {
    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
}

fn handle_api(path: &str) -> String {
    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}".to_string()
}
```

### Faz 1 - CAS Auth Implementation

**src/infrastructure/cas_adapter.rs:**
```rust
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct CasAdapter {
    base_url: String,
    service_url: String,
}

impl CasAdapter {
    pub fn new() -> Self {
        Self {
            base_url: "https://jasig.firat.edu.tr/cas".to_string(),
            service_url: "https://debsis.firat.edu.tr/login/index.php?authCAS=CAS".to_string(),
        }
    }

    pub fn login(&self, username: &str, password: &str) -> Result<String, String> {
        // Step 1: GET login page, extract hidden fields
        let (jsessionid, lt, execution) = self.get_login_form()?;

        // Step 2: POST credentials
        let ticket = self.post_credentials(username, password, &lt, &execution, &jsessionid)?;

        // Step 3: Validate ticket
        let session = self.validate_ticket(&ticket)?;

        Ok(session)
    }

    fn get_login_form(&self) -> Result<(String, String, String), String> {
        // TCP connection to jasig.firat.edu.tr:443
        // TLS handshake (native-tls or rustls - minimal crate)
        // GET /cas/login?service=...
        // Parse HTML for lt, execution values
        // Extract JSESSIONID from Set-Cookie header
        todo!("Implement HTTP over TLS")
    }

    fn post_credentials(&self, username: &str, password: &str, lt: &str, execution: &str, jsessionid: &str) -> Result<String, String> {
        // POST /cas/login with form data
        // Extract ticket from 302 redirect Location header
        todo!("Implement POST with credentials")
    }

    fn validate_ticket(&self, ticket: &str) -> Result<String, String> {
        // GET service_url?ticket=...
        // Extract MoodleSession from Set-Cookie
        todo!("Implement ticket validation")
    }
}
```

---

## 10. SORULAR (TAMAMEN ÇÖZÜLDÜ - MCP İLE DOĞRULANDI)

| # | Soru | Durum | Detay |
|---|------|-------|-------|
| 1 | CAS Login URL | ✓ | `https://jasig.firat.edu.tr/cas/login` |
| 2 | Collab URL | ✓ | `eu.bbcollab.com` JWT ile |
| 3 | Ders Programı | ✓ | Moodle AJAX API + Calendar block |
| 4 | OBS WebSocket Port | ✓ | 4455, RFC 6455 |
| 5 | CAS TGT/ST flow | ✓ | **Bölüm 8.1'de detaylı** |
| 6 | OBS Auth Protocol | ✓ | **Bölüm 8.2'de detaylı** |
| 7 | Collab JWT Yapısı | ✓ | **Bölüm 8.3'te detaylı** |
| 8 | Collab Video Stream URL | ✓ | **CloudFront Signed URL (8.3)** |
| 9 | Collab Chat Log URL | ✓ | **JSON format (8.3)** |
| 10 | Moodle Web Service API | ✓ | **Bölüm 8.5'de detaylı** |
| 11 | Debsis UI Elementleri | ✓ | **Bölüm 8.6'de detaylı** |
| 12 | Collab Player UI | ✓ | **Bölüm 8.7'de detaylı** |

---

## 11. UI DESIGN YAKLAŞIMI (Zero Framework)

### 11.1 Tasarım Prensipleri

| Prensip | Uygulama |
|---------|----------|
| **Mobile-First** | 320px → 768px → 1024px → 1440px breakpoints |
| **Dark Mode Default** | Fırat Üniversitesi renkleri (#9B2335 kırmızı, #1a1a2e koyu) |
| **Component-Based CSS** | BEM naming convention |
| **No CSS Framework** | Pure CSS variables + Grid + Flexbox |
| **Accessible** | ARIA labels, keyboard navigation |

### 11.2 Design Tokens (CSS Variables)

```css
/* web/css/variables.css */
:root {
  /* Fırat University Colors */
  --color-primary: #9B2335;      /* Fırat Kırmızısı */
  --color-primary-light: #B8344A;
  --color-primary-dark: #7A1C29;

  --color-secondary: #1a1a2e;    /* Koyu mavi */
  --color-accent: #00B894;       /* Yeşil vurgu */

  /* Dark Theme (Default) */
  --bg-primary: #0f0f1a;
  --bg-secondary: #1a1a2e;
  --bg-card: #252538;
  --text-primary: #ffffff;
  --text-secondary: #a0a0b0;
  --border-color: #3a3a4a;

  /* Spacing */
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;

  /* Typography */
  --font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  --font-size-xs: 12px;
  --font-size-sm: 14px;
  --font-size-md: 16px;
  --font-size-lg: 20px;
  --font-size-xl: 24px;

  /* Radius */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-full: 9999px;

  /* Shadows */
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.2);
  --shadow-md: 0 4px 6px rgba(0,0,0,0.3);
  --shadow-lg: 0 10px 15px rgba(0,0,0,0.4);
}

/* Light Theme */
[data-theme="light"] {
  --bg-primary: #f5f5f5;
  --bg-secondary: #ffffff;
  --bg-card: #ffffff;
  --text-primary: #1a1a2e;
  --text-secondary: #666666;
  --border-color: #e0e0e0;
}
```

### 11.3 Component Examples

**Course Card Component:**
```css
/* web/css/components.css */
.course-card {
  background: var(--bg-card);
  border-radius: var(--radius-lg);
  overflow: hidden;
  transition: transform 0.2s, box-shadow 0.2s;
}

.course-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-lg);
}

.course-card__image {
  width: 100%;
  aspect-ratio: 16/9;
  object-fit: cover;
}

.course-card__content {
  padding: var(--space-md);
}

.course-card__category {
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.course-card__title {
  font-size: var(--font-size-md);
  color: var(--text-primary);
  margin-top: var(--space-xs);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
```

**Live Session Indicator:**
```css
.live-indicator {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-xs) var(--space-sm);
  background: rgba(155, 35, 53, 0.2);
  border-radius: var(--radius-full);
  font-size: var(--font-size-xs);
  color: var(--color-primary);
}

.live-indicator::before {
  content: '';
  width: 8px;
  height: 8px;
  background: var(--color-primary);
  border-radius: 50%;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(1.2); }
}
```

### 11.4 Responsive Grid System

```css
/* web/css/layout.css */
.container {
  width: 100%;
  max-width: 1440px;
  margin: 0 auto;
  padding: 0 var(--space-md);
}

.grid {
  display: grid;
  gap: var(--space-md);
}

.grid--courses {
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
}

.grid--dashboard {
  grid-template-columns: 1fr;
}

@media (min-width: 768px) {
  .grid--dashboard {
    grid-template-columns: 2fr 1fr;
  }
}

@media (min-width: 1024px) {
  .grid--dashboard {
    grid-template-columns: 3fr 1fr;
  }
}
```

---

## 12. KAPSAMLI UX TASARIMI (Apple Tarzı Minimalist)

### 12.1 Tasarım Felsefesi

**Debsis/Collab Problemleri:**
- Kalabalık, gereksiz elementler
- Yavaş yüklenme
- Mobil uyumsuzluk
- Karmaşık navigasyon
- Düşük kalite video kayıtları
- Manuel kayıt açma sorunu

**Bizim Yaklaşımımız:**
```
┌─────────────────────────────────────────────────────────────────────┐
│                    APPLE-INSPIRED DESIGN PRINCIPLES                 │
├─────────────────────────────────────────────────────────────────────┤
│  1. REMOVE: Gereksiz her şeyi sil                                   │
│  2. ORGANIZE: Doğal gruplandırma                                    │
│  3. CLARIFY: Net, anlaşılır içerik                                  │
│  4. REDUCE: En az tıklama ile hedef                                 │
│  5. PREVENT: Hataları önle, değil düzelt                            │
└─────────────────────────────────────────────────────────────────────┘
```

### 12.2 Uygulama Akışı (Zero Friction)

```
┌─────────────────────────────────────────────────────────────────────┐
│                         USER JOURNEY                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐           │
│  │  LOGIN  │───▶│ DASHBOARD│───▶│  CLASS  │───▶│ RECORD  │           │
│  │  (CAS)  │    │ (Cards)  │    │ (Join)  │    │ (Auto)  │           │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘           │
│       │              │              │              │                 │
│       │              ▼              ▼              ▼                 │
│       │         ┌─────────┐   ┌─────────┐   ┌─────────┐            │
│       │         │ PROFILE │   │  CHAT   │   │ PLAYBACK │            │
│       │         │(Settings)│   │(Synced) │   │ (High-Q) │            │
│       │         └─────────┘   └─────────┘   └─────────┘            │
│       │              │              │              │                 │
│       └──────────────┴──────────────┴──────────────┘                 │
│                           │                                         │
│                    ┌──────▼──────┐                                  │
│                    │  SAZAN.AVI  │                                  │
│                    │ (Auto Mode) │                                  │
│                    └─────────────┘                                  │
└─────────────────────────────────────────────────────────────────────┘
```

### 12.3 Sayfa Tasarımları

#### 12.3.1 Login Screen (Minimal)

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│                                                                      │
│                      ┌────────────────────┐                          │
│                      │   🎓 FIRAT SHADOW  │                          │
│                      │     HANDBOOK       │                          │
│                      └────────────────────┘                          │
│                                                                      │
│                      ┌────────────────────┐                          │
│                      │ Kullanıcı Adı      │                          │
│                      │ 255196045          │                          │
│                      └────────────────────┘                          │
│                                                                      │
│                      ┌────────────────────┐                          │
│                      │ Şifre              │                          │
│                      │ •••••••••••        │                          │
│                      └────────────────────┘                          │
│                                                                      │
│                      ┌────────────────────┐                          │
│                      │      GİRİŞ YAP     │                          │
│                      └────────────────────┘                          │
│                                                                      │
│                      ┌────────────────────┐                          │
│                      │  CAS ile Bağlan   │  ← Tek buton, otomatik   │
│                      └────────────────────┘                          │
│                                                                      │
│              ─────────────────────────────────────                   │
│                      Dark Mode • Türkçe ▼                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Özellikler:**
- Tek form field, otomatik CAS redirect
- Dark mode default
- Dil seçimi alt köşede
- Hata mesajları inline, kırmızı border

#### 12.3.2 Dashboard (Apple Cards Style)

```
┌─────────────────────────────────────────────────────────────────────┐
│ ☰  Fırat Shadow Handbook                    🔔 3    👤 Abdullah     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  🔴 CANLI DERS                                              │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  Internet Programcılığı I - Şu an aktif                     │    │
│  │  Hoca: Doç. Dr. Ahmet Yılmaz                                │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │    │
│  │  │   🔴 KATIL   │  │  📹 KAYDET   │  │  🤖 SAZAN    │       │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘       │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  📅 BUGÜN                                                    │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  18:00  Internet Programcılığı I      [🔴 Katıl]            │    │
│  │  19:30  Türk Dili II                   [⏳ Bekliyor]         │    │
│  │  21:00  Atatürk İlkeleri              [⏳ Bekliyor]          │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  📚 DERSLERİM                              ← Tümünü Gör →   │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │    │
│  │  │ 🖼️      │ │ 🖼️      │ │ 🖼️      │ │ 🖼️      │        │    │
│  │  │ Internet │ │ Türk Dili│ │ Atatürk  │ │ Yapay Zek│        │    │
│  │  │ Prog. I  │ │   II     │ │ İlk. II  │ │ Araçlar  │        │    │
│  │  │ 3 kayıt  │ │ 2 kayıt  │ │ 1 kayıt  │ │ 0 kayıt  │        │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘        │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  🎥 SON KAYITLAR                                            │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  ┌──────────────────────────────────────────────────────┐   │    │
│  │  │ 🖼️  Internet Prog. I - 24 Şubat    1:06:00   ▶ İZLE  │   │    │
│  │  └──────────────────────────────────────────────────────┘   │    │
│  │  ┌──────────────────────────────────────────────────────┐   │    │
│  │  │ 🖼️  Türk Dili II - 23 Şubat       0:45:00   ▶ İZLE  │   │    │
│  │  └──────────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Tasarım Prensipleri:**
- Card-based layout (iOS tarzı)
- Canlı ders en üstte, vurgulu
- Minimum tıklama: 1 tık = derse katıl
- Gereksiz footer, sidebar YOK
- Responsive: Mobilde tek kolon

#### 12.3.3 Class View (Live Session)

```
┌─────────────────────────────────────────────────────────────────────┐
│ ← Internet Programcılığı I                              🔴 CANLI    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                                                              │    │
│  │                                                              │    │
│  │                    📹 VIDEO AREA                             │    │
│  │                    (Collab Embed                             │    │
│  │                     veya kendi player)                       │    │
│  │                                                              │    │
│  │                                                              │    │
│  ├─────────────────────────────────────────────────────────────┤    │
│  │  ▶ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━○━━━━━ 45:00 / 1:30:00  🔊 ⛶│    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌───────────────────┐  ┌───────────────────────────────────────┐    │
│  │ 🤖 SAZAN.AVI      │  │ 💬 CHAT                              │    │
│  │ ─────────────────│  │ ─────────────────────────────────────│    │
│  │                   │  │                                      │    │
│  │ [ ] Sadece Katıl  │  │ 18:05  Abdullah: Merhaba hocam     │    │
│  │ [ ] Soru Algıla   │  │ 18:06  Hoca: Merhaba, hoş geldiniz  │    │
│  │ [●] AI Cevaplar   │  │ 18:07  Nagihan: İyi akşamlar ↩️     │    │
│  │ [ ] Tam Otonom    │  │        └─ Reply to: İyi akşamlar    │    │
│  │                   │  │                                      │    │
│  │ ─────────────────│  │ ┌──────────────────────────────────┐ │    │
│  │ Durum: Aktif      │  │ │ Mesaj yaz...              📎 ➤│ │    │
│  │ Yanıtlanan: 3     │  │ └──────────────────────────────────┘ │    │
│  │ Bekleyen: 0       │  │                                      │    │
│  └───────────────────┘  └───────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  📹 KAYIT DURUMU                                            │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  Collab Kaydı: 🔴 Aktif (Otomatik)                          │    │
│  │  Bizim Kaydımız: 🔴 Aktif (1080p, OBS)                      │    │
│  │  Süre: 45:00                                                │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 12.4 Video Player (Custom)

```
┌─────────────────────────────────────────────────────────────────────┐
│ ← Internet Programcılığı I - 24 Şubat 2026                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                                                              │    │
│  │                                                              │    │
│  │                    📹 HIGH QUALITY VIDEO                     │    │
│  │                    (CloudFront MP4)                          │    │
│  │                                                              │    │
│  │                                                              │    │
│  ├─────────────────────────────────────────────────────────────┤    │
│  │  ◀◀  ▶/❚❚  ▶▶   ━━━━━━━━━━━━━━━━━━━○━━━━━━━  45:00/1:30:00 │    │
│  │                  1x ▼   🔊 ▼   CC ▼   ⛶                    │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  📥 İNDİR                                                   │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  [ 1080p MP4 ]  [ 720p MP4 ]  [ Sadece Ses ]  [ Chat JSON ] │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌───────────────────┐  ┌───────────────────────────────────────┐    │
│  │ 📋 BÖLÜMLER       │  │ 💬 CHAT (Senkronize)                  │    │
│  │ ─────────────────│  │ ─────────────────────────────────────│    │
│  │                   │  │                                      │    │
│  │ 00:00 Giriş       │  │ Video ile senkronize chat           │    │
│  │ 15:30 Diziler     │  │ Tıkla → o anki chat gösterilir       │    │
│  │ 32:00 Fonksiyonlar│  │                                      │    │
│  │ 45:00 Örnek Uygul.│  │ ─────────────────────────────────────│    │
│  │ 1:10:00 Sorular   │  │ Filtre: [Tümü] [Hoca] [Ben] [Önemli]│    │
│  │                   │  │                                      │    │
│  └───────────────────┘  └───────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 12.5 Chat Sistemi (Gelişmiş)

#### 12.5.1 Chat Özellikleri

| Özellik | Açıklama |
|---------|----------|
| **Reply** | Her mesaja reply → thread oluşur |
| **Edit** | Mesajı düzenle (3 dk içinde) |
| **Delete** | Mesajı sil (sadece kendi) |
| **React** | 👍 👎 😂 ❤️ 🎉 emoji reaksiyonlar |
| **Pin** | Önemli mesajları sabitle |
| **Color** | Kullanıcı bazlı renk otomatik atanır |
| **Sync** | Collab'a eş zamanlı gönderilir |

#### 12.5.2 Chat Renk Ağacı

```javascript
// Kullanıcı renk atama algoritması
const USER_COLORS = [
  '#FF6B6B', // Kırmızı
  '#4ECDC4', // Turkuaz
  '#45B7D1', // Mavi
  '#96CEB4', // Yeşil
  '#FFEAA7', // Sarı
  '#DDA0DD', // Mor
  '#98D8C8', // Mint
  '#F7DC6F', // Altın
  '#BB8FCE', // Lavanta
  '#85C1E9', // Gökyüzü
];

function getUserColor(username) {
  // Hash-based color assignment
  let hash = 0;
  for (let i = 0; i < username.length; i++) {
    hash = username.charCodeAt(i) + ((hash << 5) - hash);
  }
  return USER_COLORS[Math.abs(hash) % USER_COLORS.length];
}
```

#### 12.5.3 Chat UI Detayı

```
┌─────────────────────────────────────────────────────────────────────┐
│ 💬 Internet Programcılığı I - Chat                          📌 ⚙️   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ 18:05                                                          │  │
│  │ ┌─────────────────────────────────────────────────────────┐    │  │
│  │ │ 🔴 Abdullah Ulaç Yılmaz                         📌 ÖNEMLİ│    │  │
│  │ │ Merhaba arkadaşlar, bugün PHP dizilerini işleyeceğiz    │    │  │
│  │ │ ─────────────────────────────────────────────────────── │    │  │
│  │ │ 👍 5  😂 2  ↩️ 3 yanıtlar                               │    │  │
│  │ └─────────────────────────────────────────────────────────┘    │  │
│  │   └─ ▼ Yanıtlar (3)                                            │  │
│  │       ┌─────────────────────────────────────────────────┐      │  │
│  │       │ 🔵 Nagihan Ak                                     │      │  │
│  │       │   Hocam dizilerle ilgili bir sorum var            │      │  │
│  │       │   ✏️ 🗑️                                            │      │  │
│  │       └─────────────────────────────────────────────────┘      │  │
│  │       ┌─────────────────────────────────────────────────┐      │  │
│  │       │ 🟢 Mahizer Erkuş                                 │      │  │
│  │       │   Aynen hocam ben de merak ediyorum              │      │  │
│  │       └─────────────────────────────────────────────────┘      │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ 18:10                                                          │  │
│  │ ┌─────────────────────────────────────────────────────────┐    │  │
│  │ │ 🟣 Levent Sertkaya                                      │    │  │
│  │ │ Hocam $ işareti değişken isminde kullanılabilir mi?     │    │  │
│  │ │ ✏️ 🗑️                                                    │    │  │
│  │ └─────────────────────────────────────────────────────────┘    │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ ┌─────────────────────────────────────────────────────────┐   │  │
│  │ │ 🔴 Hoca (Doç. Dr. Ahmet Yılmaz)                  📌 ÖNEMLİ│   │  │
│  │ │ Evet, $ işareti ile başlayan değişkenler geçerlidir     │   │  │
│  │ │ ancak _ ile başlayanlar daha yaygın kullanılır          │   │  │
│  │ └─────────────────────────────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ────────────────────────────────────────────────────────────────── │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ Mesaj yaz...                                          📎 ➤  │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 12.6 Profil Sayfası

```
┌─────────────────────────────────────────────────────────────────────┐
│ ← Geri                                    Fırat Shadow Handbook     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                      👤 PROFİL                               │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │                                                              │    │
│  │         ┌─────────┐                                          │    │
│  │         │   🖼️    │    Abdullah Ulaç Yılmaz                 │    │
│  │         │  AVATAR │    255196045@firat.edu.tr                │    │
│  │         └─────────┘    Bilgisayar Programcılığı              │    │
│  │                       2. Sınıf                               │    │
│  │                       ─────────────────────                  │    │
│  │                       📷 Fotoğraf Değiştir                   │    │
│  │                                                              │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  ⚙️ AYARLAR                                                  │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │                                                              │    │
│  │  🌙 Tema                    [Dark] ▼                         │    │
│  │  🌍 Dil                     [Türkçe] ▼                       │    │
│  │  🔔 Bildirimler             [Açık] ○────●                    │    │
│  │  🔊 Ses                     [Açık] ○────●                    │    │
│  │  📹 Otomatik Kayıt          [Açık] ○────●                    │    │
│  │  🤖 Sazan.avi Varsayılan    [Kapalı] ○────●                  │    │
│  │                                                              │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  📹 KAYIT AYARLARI                                           │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │                                                              │    │
│  │  Kalite                     [1080p] ▼                       │    │
│  │  Format                     [MP4] ▼                          │    │
│  │  Depolama                   [Yerel] ▼                         │    │
│  │  OBS WebSocket              ws://localhost:4455              │    │
│  │  OBS Şifre                  ••••••••                          │    │
│  │                                                              │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  📊 İSTATİSTİKLER                                            │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │                                                              │    │
│  │  Toplam Ders Saati          45 saat                         │    │
│  │  İzlenen Kayıt              12 video                         │    │
│  │  Gönderilen Mesaj           156 adet                         │    │
│  │  Sazan.avi Kullanımı        8 saat                           │    │
│  │                                                              │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  🚪 OTURUMU KAPAT                                            │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 12.7 Kayıt Sistemi (Dual Recording)

```
┌─────────────────────────────────────────────────────────────────────┐
│                    DUAL RECORDING ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                     CANLI DERS BAŞLADI                       │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                              │                                       │
│              ┌───────────────┴───────────────┐                      │
│              ▼                               ▼                      │
│  ┌───────────────────────┐     ┌───────────────────────┐            │
│  │   COLLAB KAYDI        │     │   BİZİM KAYDIMIZ      │            │
│  │   (Otomatik)          │     │   (OBS + High-Q)      │            │
│  │   ───────────────────│     │   ───────────────────│            │
│  │   • Hoca açar/kapar   │     │   • Otomatik başlar   │            │
│  │   • Düşük kalite      │     │   • 1080p/4K seçenek  │            │
│  │   • Collab sunucusu   │     │   • Yerel depolama    │            │
│  │   • Süreli erişim     │     │   • Kalıcı arşiv      │            │
│  └───────────────────────┘     └───────────────────────┘            │
│              │                               │                      │
│              ▼                               ▼                      │
│  ┌───────────────────────┐     ┌───────────────────────┐            │
│  │   CloudFront CDN      │     │   Yerel Disk          │            │
│  │   (Süreli URL)         │     │   /recordings/        │            │
│  └───────────────────────┘     └───────────────────────┘            │
│              │                               │                      │
│              └───────────────┬───────────────┘                      │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    KULLANICI İZLER                          │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  Seçenek 1: Collab Kaydı (düşük kalite, hızlı)             │    │
│  │  Seçenek 2: Bizim Kayıt (yüksek kalite, indirme var)       │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 12.8 Chat Senkronizasyonu

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CHAT SYNC ARCHITECTURE                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    BİZİM SİSTEM                              │    │
│  │  ┌─────────────────────────────────────────────────────┐    │    │
│  │  │  Kullanıcı mesaj yazıyor: "Merhaba hocam"           │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  │                          │                                   │    │
│  │              ┌───────────┴───────────┐                      │    │
│  │              ▼                       ▼                      │    │
│  │  ┌───────────────────┐   ┌───────────────────┐              │    │
│  │  │  Local Display    │   │  Collab API       │              │    │
│  │  │  (Anında göster)  │   │  (Eş zamanlı)     │              │    │
│  │  └───────────────────┘   └───────────────────┘              │    │
│  │                                  │                           │    │
│  │                                  ▼                           │    │
│  │  ┌───────────────────────────────────────────────────────┐  │    │
│  │  │  Collab Chat API (WebSocket veya REST)                │  │    │
│  │  │  POST /collab/api/csa/session/{id}/chat               │  │    │
│  │  │  Body: { "message": "Merhaba hocam", "userId": "..." }│  │    │
│  │  └───────────────────────────────────────────────────────┘  │    │
│  │                                  │                           │    │
│  │                                  ▼                           │    │
│  │  ┌───────────────────────────────────────────────────────┐  │    │
│  │  │  Collab'da da görünür                                 │  │    │
│  │  │  (Tüm katılımcılar görür)                             │  │    │
│  │  └───────────────────────────────────────────────────────┘  │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    COLLAB'DAN BİZE                          │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  WebSocket Subscribe: /collab/api/csa/session/{id}/chat     │    │
│  │  → Her mesaj bize de düşer                                 │    │
│  │  → Chat JSON'a kaydedilir                                  │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 12.9 Sazan.avi Modu (Otomasyon)

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SAZAN.AVI LEVELS                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Level 0: KAPALI                                                    │
│  ────────────────────────────────────────────────────────────────── │
│  • Manuel kontrol                                                   │
│  • Tüm işlemler kullanıcı tarafından                               │
│                                                                      │
│  Level 1: SADECE KATILIM                                            │
│  ────────────────────────────────────────────────────────────────── │
│  • Ders başladığında otomatik katıl                                 │
│  • Kayıt otomatik başlat                                            │
│  • Ders bittiğinde otomatik çık                                     │
│                                                                      │
│  Level 2: SORU ALGILAMA                                             │
│  ────────────────────────────────────────────────────────────────── │
│  • Level 1 +                                                        │
│  • Chat'teki soruları algıla                                        │
│  • "Hoca", "soru", "?" keywordleri                                  │
│  • Bildirim gönder: "Soru var!"                                     │
│                                                                      │
│  Level 3: AI CEVAPLAR                                               │
│  ────────────────────────────────────────────────────────────────── │
│  • Level 2 +                                                        │
│  • Basit sorulara AI cevap ver                                      │
│  • "Evet hocam", "Hayır hocam", "Anlaşıldı"                        │
│  • Yoklama için "Buradayım"                                         │
│                                                                      │
│  Level 4: TAM OTONOM                                                │
│  ────────────────────────────────────────────────────────────────── │
│  • Level 3 +                                                        │
│  • Kompleks sorulara AI cevap                                       │
│  • Ders içeriğini takip et                                          │
│  • Not tut (otomatik özet)                                          │
│  • Ödev tarihlerini hatırlat                                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 12.10 Responsive Breakpoints

```css
/* Mobile First Approach */

/* Base: 320px - 767px (Mobile) */
.container { padding: 16px; }
.grid { grid-template-columns: 1fr; }
.card { margin-bottom: 16px; }

/* Tablet: 768px - 1023px */
@media (min-width: 768px) {
  .container { padding: 24px; }
  .grid--dashboard { grid-template-columns: 2fr 1fr; }
  .grid--courses { grid-template-columns: repeat(2, 1fr); }
}

/* Desktop: 1024px - 1439px */
@media (min-width: 1024px) {
  .container { padding: 32px; max-width: 1200px; }
  .grid--dashboard { grid-template-columns: 3fr 1fr; }
  .grid--courses { grid-template-columns: repeat(3, 1fr); }
}

/* Large Desktop: 1440px+ */
@media (min-width: 1440px) {
  .container { max-width: 1440px; }
  .grid--courses { grid-template-columns: repeat(4, 1fr); }
}
```

### 12.11 Ücretsiz Kaynaklar (0 ₺ Bütçe)

| Kaynak | Kullanım | Ücretsiz Limit |
|--------|----------|----------------|
| **Cloudflare R2** | Video storage | 10 GB/month free |
| **GitHub Pages** | Static hosting | Unlimited |
| **Vercel** | Serverless functions | 100 GB bandwidth |
| **Supabase Free** | PostgreSQL + Auth | 500 MB database |
| **Firebase Free** | Realtime DB | 1 GB storage |
| **OBS Studio** | Local recording | Free forever |
| **FFmpeg** | Video processing | Free forever |
| **Local Storage** | Browser storage | ~5-10 MB |
| **IndexedDB** | Browser database | ~50-500 MB |

**Önerilen Stack (0 ₺):**
```
┌─────────────────────────────────────────────────────────────────────┐
│                    ZERO COST ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Frontend:                                                           │
│  • Vanilla JS (CDN yok, local)                                      │
│  • Pure CSS (framework yok)                                         │
│  • Local Storage + IndexedDB                                        │
│                                                                      │
│  Backend:                                                            │
│  • Rust std::net (hosting yok, local binary)                        │
│  • SQLite (database yok, local file)                                │
│                                                                      │
│  Video Storage:                                                      │
│  • Yerel disk (cloud yok)                                           │
│  • Cloudflare R2 (opsiyonel, 10 GB free)                            │
│                                                                      │
│  Recording:                                                          │
│  • OBS Studio (free)                                                │
│  • FFmpeg (free)                                                    │
│  • MediaRecorder API (browser native)                               │
│                                                                      │
│  Deployment:                                                         │
│  • Single binary executable                                         │
│  • Portable USB drive'da çalışabilir                                │
│  • GitHub Releases (distribution)                                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 13. DEBSİS/COLLAB KARŞILAŞTIRMA

| Özellik | Debsis/Collab | Fırat Shadow Handbook |
|---------|---------------|----------------------|
| **Tasarım** | Kalabalık, eski | Minimalist, Apple tarzı |
| **Mobil** | Kısmen uyumlu | Mobile-first responsive |
| **Video Kalitesi** | Düşük (720p max) | Yüksek (1080p/4K) |
| **Kayıt** | Manuel açma | Otomatik çift kayıt |
| **Chat** | Temel özellikler | Reply, Edit, Delete, React, Pin |
| **Otomasyon** | Yok | Sazan.avi 4 seviye |
| **Hız** | Yavaş (Moodle yükü) | Hızlı (vanilla JS) |
| **Offline** | Yok | Kayıtlar yerel |
| **Bildirim** | Email only | Push + Desktop + Sound |
| **Profil** | Karmaşık | Basit, temiz |
| **Tema** | Sadece açık | Dark + Light |
| **Dil** | TR/EN | TR/EN + i18n ready |

---

## 14. ROL BAZLI ARAYÜZ (Öğrenci vs Öğretmen)

### 14.1 Rol Tespiti

```javascript
// Moodle API'den rol tespiti
async function detectUserRole() {
  const response = await fetch('/lib/ajax/service.php', {
    method: 'POST',
    body: JSON.stringify([{
      index: 0,
      methodname: 'core_user_get_users_by_field',
      args: {
        field: 'id',
        values: [currentUserId]
      }
    }])
  });
  const user = await response.json();
  // Moodle role: student, teacher, editingteacher, admin
  return user.roles; // ['student'] veya ['teacher', 'editingteacher']
}
```

### 14.2 Öğrenci Arayüzü

```
┌─────────────────────────────────────────────────────────────────────┐
│ ☰  Fırat Shadow Handbook                    🔔 3    👤 Abdullah     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  🔴 CANLI DERS                                              │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  Internet Programcılığı I - Şu an aktif                     │    │
│  │  Hoca: Doç. Dr. Ahmet Yılmaz                                │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  ┌──────────────┐  ┌──────────────┐                         │    │
│  │  │   🔴 KATIL   │  │  🤖 SAZAN    │  ← Sadece öğrencide!   │    │
│  │  └──────────────┘  └──────────────┘                         │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  NOT: Kaydet butonu YOK - Öğrenci kayıt başlatamaz                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 14.3 Öğretmen Arayüzü

```
┌─────────────────────────────────────────────────────────────────────┐
│ ☰  Fırat Shadow Handbook              🔔 3    👤 Doç. Dr. Ahmet     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  🔴 CANLI DERS (ÖĞRETMEN)                                   │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  Internet Programcılığı I                                    │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │    │
│  │  │   🔴 DERS AÇ │  │  📹 KAYDET   │  │  ⚙️ AYARLAR  │       │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘       │    │
│  │                                                              │    │
│  │  ┌──────────────────────────────────────────────────────┐   │    │
│  │  │  Kayıt Durumu:                                       │   │    │
│  │  │  • Collab Kaydı: 🔴 Aktif                            │   │    │
│  │  │  • Otomatik Kayıt: ✅ Açık (1080p)                   │   │    │
│  │  └──────────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  NOT: Sazan.avi YOK - Öğretmen otomasyon kullanamaz                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 14.4 Sazan.avi Öğrenci Eleme Sistemi

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SAZAN.AVI ELEME SİSTEMİ                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  SENARYO: Hoca Sazan.avi kullanan öğrencileri yakalıyor            │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Hoca'nın Gördüğü (Collab Chat):                            │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  18:05  Abdullah: Merhaba hocam                             │    │
│  │  18:06  Nagihan: İyi akşamlar                              │    │
│  │  18:07  Mahizer: Evet hocam ✓                ← Şüpheli!     │    │
│  │  18:08  Levent: Anlaşıldı ✓                  ← Şüpheli!     │    │
│  │  18:09  Esra: Buradayım ✓                    ← Şüpheli!     │    │
│  │                                                              │    │
│  │  ┌──────────────────────────────────────────────────────┐   │    │
│  │  │  ⚠️ ŞÜPHELİ YANITLAR                                │   │    │
│  │  │  • Mahizer: "Evet hocam" - 0.5s içinde              │   │    │
│  │  │  • Levent: "Anlaşıldı" - 0.3s içinde                │   │    │
│  │  │  • Esra: "Buradayım" - Yoklama anında               │   │    │
│  │  │                                                      │   │    │
│  │  │  [ 🚨 Raporla ]  [ 📋 Listeye Ekle ]                │   │    │
│  │  └──────────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  BİZİM SİSTEMİN YAPMASI GEREKENLER:                                 │
│  ────────────────────────────────────────────────────────────────── │
│  1. AI cevapları DOĞAL ZAMANLAMA ile göndermeli                    │
│     • Rastgele gecikme: 2-8 saniye                                 │
│     • İnsan benzeri yazma hızı simülasyonu                         │
│                                                                      │
│  2. Cevap çeşitliliği                                               │
│     • Aynı cevabı tekrar kullanma                                  │
│     • "Evet hocam", "Anlaşıldı hocam", "Tamamdır hocam"            │
│     • Bazen emoji ekle: "Evet hocam 👍"                             │
│                                                                      │
│  3. Yoklama için özel davranış                                      │
│     • Hoca "yoklama" dediğinde 5-10 saniye bekle                   │
│     • Sonra "Buradayım" veya "🤚" gönder                           │
│                                                                      │
│  4. Tehlike algılama                                                │
│     • Hoca "kim" dediğinde sessiz kal                              │
│     • "Sazan" keyword'ü varsa PANIK modu                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 15. SES TRANSKRİPSİYONU (Whisper AI)

### 15.1 Whisper AI Entegrasyonu

```
┌─────────────────────────────────────────────────────────────────────┐
│                    WHISPER AI - OFFLINE TRANSCRIPTION               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  TEKNOLOJİ: OpenAI Whisper (MIT Lisanslı, Ücretsiz)                 │
│  ────────────────────────────────────────────────────────────────── │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  SEÇENEKLER:                                                │    │
│  │                                                              │    │
│  │  1. WHISPER.CPP (Rust entegrasyonu)                        │    │
│  │     • whisper.cpp native library                            │    │
│  │     • Rust FFI ile çağrılır                                 │    │
│  │     • CPU only, GPU optional                                │    │
│  │     • Model boyutu: 75MB - 2.9GB                            │    │
│  │                                                              │    │
│  │  2. WHISPER WASM (Browser'de çalışır)                      │    │
│  │     • WebAssembly ile browser'da                            │    │
│  │     • GPU gerektirmez                                        │    │
│  │     • Model: whisper-base (74MB)                            │    │
│  │                                                              │    │
│  │  3. WHISPER ONNX (Browser'de, ONNX Runtime)                │    │
│  │     • Microsoft ONNX Runtime Web                            │    │
│  │     • WebGPU ile hızlandırılabilir                          │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  KULLANIM:                                                          │
│  ────────────────────────────────────────────────────────────────── │
│  • Ders sırasında hoca'nın konuşması gerçek zamanlı transkripsiyon │
│  • Kayıttan sonra otomatik alt yazı oluşturma                      │
│  • Chat'e "Hoca şunu dedi: ..." otomatik reply                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 15.2 Rust Whisper Entegrasyonu

```rust
// src/infrastructure/whisper_adapter.rs
use std::process::Command;

pub struct WhisperTranscriber {
    model_path: String,
    language: String,
}

impl WhisperTranscriber {
    pub fn new(model_path: &str) -> Self {
        Self {
            model_path: model_path.to_string(),
            language: "tr".to_string(), // Türkçe
        }
    }

    /// Transcribe audio file to text
    pub fn transcribe(&self, audio_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("whisper")
            .arg(audio_path)
            .arg("--model")
            .arg(&self.model_path)
            .arg("--language")
            .arg(&self.language)
            .arg("--output_format")
            .arg("json")
            .arg("--output_dir")
            .arg("/tmp/transcripts")
            .output()?;

        if output.status.success() {
            // Read the generated JSON file
            let json_path = audio_path.replace(".wav", ".json");
            let content = std::fs::read_to_string(json_path)?;
            Ok(content)
        } else {
            Err(format!("Whisper failed: {:?}", output.stderr).into())
        }
    }

    /// Real-time transcription from stream
    pub fn transcribe_stream(&self, audio_chunk: &[f32]) -> String {
        // Whisper.cpp real-time API
        // Bu kısım whisper.cpp FFI binding gerektirir
        todo!("Implement real-time transcription")
    }
}
```

### 15.3 Browser Whisper (WASM)

```javascript
// web/js/whisper.js
class WhisperTranscriber {
    constructor() {
        this.model = null;
        this.isLoaded = false;
    }

    async loadModel(modelUrl = '/models/whisper-base.wasm') {
        // Load Whisper WASM model
        const response = await fetch(modelUrl);
        const wasmBuffer = await response.arrayBuffer();
        // Initialize Whisper WASM
        this.model = await WhisperModule(wasmBuffer);
        this.isLoaded = true;
    }

    async transcribe(audioBlob) {
        if (!this.isLoaded) {
            throw new Error('Model not loaded');
        }

        const audioContext = new AudioContext();
        const audioBuffer = await audioContext.decodeAudioData(await audioBlob.arrayBuffer());

        // Run transcription
        const result = await this.model.transcribe(audioBuffer.getChannelData(0));
        return result.text;
    }

    // Real-time transcription from MediaStream
    startRealTimeTranscription(stream, onTranscript) {
        const audioContext = new AudioContext();
        const source = audioContext.createMediaStreamSource(stream);
        const processor = audioContext.createScriptProcessor(4096, 1, 1);

        let chunks = [];

        processor.onaudioprocess = (e) => {
            chunks.push(e.inputBuffer.getChannelData(0));

            // Process every 5 seconds of audio
            if (chunks.length > 10) {
                this.processChunks(chunks, onTranscript);
                chunks = [];
            }
        };

        source.connect(processor);
        processor.connect(audioContext.destination);
    }

    async processChunks(chunks, callback) {
        // Combine chunks and transcribe
        const combined = this.combineChunks(chunks);
        const transcript = await this.transcribe(combined);
        callback(transcript);
    }
}
```

---

## 16. VİDEO CHAPTER SİSTEMİ (YouTube Tarzı)

### 16.1 Chapter Oluşturma Akışı

```
┌─────────────────────────────────────────────────────────────────────┐
│                    AUTOMATIC CHAPTER GENERATION                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  GİRDİLER:                                                          │
│  ────────────────────────────────────────────────────────────────── │
│  1. Video transcript (Whisper'dan)                                  │
│  2. Timestamps (her cümlenin zamanı)                               │
│  3. Slide değişimleri (opsiyonel)                                  │
│                                                                      │
│  SÜREÇ:                                                             │
│  ────────────────────────────────────────────────────────────────── │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  ADIM 1: Transcript Al                                      │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  [00:00] Merhaba arkadaşlar, bugün PHP dizilerini...        │    │
│  │  [05:30] Şimdi dizilerin tanımına geçelim...                │    │
│  │  [12:45] Fonksiyonlar konusuna geldik...                    │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                          │                                          │
│                          ▼                                          │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  ADIM 2: LLM ile Segmentasyon                               │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  Prompt:                                                    │    │
│  │  "Bu ders transcriptını anlamsal bölümlere ayır.            │    │
│  │   Her bölüm için başlık ve timestamp ver."                  │    │
│  │                                                              │    │
│  │  Output:                                                    │    │
│  │  1. Giriş ve Konu Özeti (00:00)                             │    │
│  │  2. Dizilerin Tanımı (05:30)                                │    │
│  │  3. Dizi İşlemleri (12:45)                                  │    │
│  │  4. Fonksiyonlar (25:00)                                    │    │
│  │  5. Örnek Uygulama (40:00)                                  │    │
│  │  6. Soru Cevap (55:00)                                      │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                          │                                          │
│                          ▼                                          │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  ADIM 3: Chapter UI                                         │    │
│  │  ───────────────────────────────────────────────────────────│    │
│  │  ┌──────────────────────────────────────────────────────┐   │    │
│  │  │ 📋 BÖLÜMLER                                           │   │    │
│  │  │ ─────────────────────────────────────────────────────│   │    │
│  │  │ 00:00 ──▶ Giriş ve Konu Özeti                        │   │    │
│  │  │ 05:30 ──▶ Dizilerin Tanımı                           │   │    │
│  │  │ 12:45 ──▶ Dizi İşlemleri                             │   │    │
│  │  │ 25:00 ──▶ Fonksiyonlar                               │   │    │
│  │  │ 40:00 ──▶ Örnek Uygulama                             │   │    │
│  │  │ 55:00 ──▶ Soru Cevap                                 │   │    │
│  │  └──────────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 16.2 Ücretsiz LLM Seçenekleri

| LLM | Boyut | RAM | Türkçe | Ücretsiz |
|-----|-------|-----|--------|----------|
| **Llama 3.2 3B** | 2GB | 6GB | ✅ | ✅ |
| **Gemma 2 2B** | 1.4GB | 4GB | ✅ | ✅ |
| **Phi-3 Mini** | 2.3GB | 4GB | ⚠️ | ✅ |
| **Qwen 2.5 3B** | 2GB | 6GB | ✅ | ✅ |
| **Mistral 7B** | 4.1GB | 8GB | ✅ | ✅ |

**Önerilen:** Llama 3.2 3B veya Qwen 2.5 3B (Türkçe performansı iyi)

### 16.3 Chapter Extraction Kodu

```javascript
// web/js/chapters.js
class ChapterGenerator {
    constructor(llmEndpoint = 'http://localhost:8080/v1/chat/completions') {
        this.llmEndpoint = llmEndpoint;
    }

    async generateChapters(transcript) {
        const prompt = `
Bu bir ders transcriptıdır. İçeriği anlamsal bölümlere ayır ve her bölüm için:
1. Başlık (kısa ve açıklayıcı)
2. Başlangıç timestamp'i

Format:
[timestamp] | Başlık

Transcript:
${transcript}

Sadece bölümleri listele, başka açıklama yapma.
`;

        const response = await fetch(this.llmEndpoint, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                model: 'llama3.2',
                messages: [{ role: 'user', content: prompt }],
                temperature: 0.3,
                max_tokens: 500
            })
        });

        const data = await response.json();
        return this.parseChapters(data.choices[0].message.content);
    }

    parseChapters(text) {
        const chapters = [];
        const lines = text.split('\n');

        for (const line of lines) {
            // Parse: "00:00 | Giriş ve Konu Özeti"
            const match = line.match(/\[(\d{2}:\d{2})\]\s*\|\s*(.+)/);
            if (match) {
                chapters.push({
                    timestamp: this.parseTimestamp(match[1]),
                    title: match[2].trim()
                });
            }
        }

        return chapters;
    }

    parseTimestamp(ts) {
        const [min, sec] = ts.split(':').map(Number);
        return min * 60 + sec;
    }
}
```

---

## 17. WHITEBOARD SİSTEMİ (Excalidraw)

### 17.1 Collab Beyaz Tahta Özellikleri

| Özellik | Collab | Excalidraw |
|---------|--------|------------|
| **Çizim** | ✅ Temel | ✅ Gelişmiş |
| **Şekiller** | ✅ | ✅ Otomatik |
| **Yazı** | ✅ | ✅ |
| **Paylaşım** | ✅ Anlık | ✅ WebSocket |
| **Export** | ❌ | ✅ PNG/SVG/JSON |
| **Offline** | ❌ | ✅ LocalStorage |
| **Templates** | ❌ | ✅ |

### 17.2 Excalidraw Entegrasyonu

```
┌─────────────────────────────────────────────────────────────────────┐
│                    EXCALIDRAW WHITEBOARD                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  KURULUM:                                                           │
│  ────────────────────────────────────────────────────────────────── │
│  1. Excalidraw NPM paketi (veya CDN)                               │
│  2. Storage Backend (collaboration için)                           │
│  3. WebSocket Server (gerçek zamanlı sync)                         │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  ARCHITECTURE:                                              │    │
│  │                                                              │    │
│  │  ┌──────────────┐     WebSocket      ┌──────────────┐       │    │
│  │  │  Browser A   │ ◀───────────────▶ │              │       │    │
│  │  │  (Excalidraw) │                   │   Rust WS    │       │    │
│  │  └──────────────┘                   │   Server     │       │    │
│  │                                      │              │       │    │
│  │  ┌──────────────┐     WebSocket      │              │       │    │
│  │  │  Browser B   │ ◀───────────────▶ │              │       │    │
│  │  │  (Excalidraw) │                   └──────────────┘       │    │
│  │  └──────────────┘                          │                 │    │
│  │                                             │                 │    │
│  │                                      ┌──────▼──────┐        │    │
│  │                                      │   SQLite    │        │    │
│  │                                      │  (Storage)  │        │    │
│  │                                      └─────────────┘        │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 17.3 Excalidraw Frontend

```html
<!-- web/whiteboard.html -->
<!DOCTYPE html>
<html lang="tr">
<head>
    <meta charset="UTF-8">
    <title>Beyaz Tahta - Fırat Shadow Handbook</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        #app { width: 100vw; height: 100vh; }
    </style>
</head>
<body>
    <div id="app"></div>
    <script type="module">
        import { Excalidraw } from '/js/excalidraw.min.js';

        const excalidrawAPI = await Excalidraw.create({
            container: document.getElementById('app'),
            initialData: {
                elements: [],
                appState: {
                    viewBackgroundColor: '#1e1e1e', // Dark mode
                    gridSize: null,
                }
            },
            onChange: (elements, appState) => {
                // Sync to WebSocket
                syncToServer(elements, appState);
            }
        });

        // WebSocket sync
        const ws = new WebSocket('ws://localhost:8080/whiteboard');
        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            excalidrawAPI.updateScene({
                elements: data.elements
            });
        };

        function syncToServer(elements, appState) {
            ws.send(JSON.stringify({
                type: 'update',
                elements: elements,
                appState: appState
            }));
        }
    </script>
</body>
</html>
```

---

## 18. OBS-ALTERNATİFİ (MediaRecorder API)

### 18.1 Browser Tabanlı Kayıt

```
┌─────────────────────────────────────────────────────────────────────┐
│                    BROWSER-BASED RECORDING                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  AVANTAJLAR:                                                        │
│  ────────────────────────────────────────────────────────────────── │
│  • OBS kurulumu gerektirmez                                        │
│  • Her bilgisayarda çalışır                                        │
│  • Sıfır external dependency                                       │
│  • Browser native API                                              │
│                                                                      │
│  DEZAVANTAJLAR:                                                     │
│  ────────────────────────────────────────────────────────────────── │
│  • Sadece browser içi kayıt                                        │
│  • Ekran kaydı için izin gerekli                                   │
│  • GPU encoding yok                                                │
│  • Maksimum 1080p                                                  │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  KULLANIM SENARYOLARI:                                      │    │
│  │                                                              │    │
│  │  1. TAB KAYDI (Collab Player)                              │    │
│  │     • getDisplayMedia() ile tab seçimi                     │    │
│  │     • Sadece Collab tab'ini kaydet                         │    │
│  │     • Audio capture dahil                                   │    │
│  │                                                              │    │
│  │  2. WINDOW KAYDI                                            │    │
│  │     • Browser penceresini kaydet                            │    │
│  │     • Tüm aktivite kaydedilir                               │    │
│  │                                                              │    │
│  │  3. SCREEN KAYDI                                            │    │
│  │     • Tüm ekranı kaydet                                     │    │
│  │     • Multi-monitor desteği                                 │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 18.2 MediaRecorder Implementation

```javascript
// web/js/recorder.js
class BrowserRecorder {
    constructor() {
        this.mediaRecorder = null;
        this.recordedChunks = [];
        this.stream = null;
    }

    async startScreenRecording(options = {}) {
        const {
            videoBitsPerSecond = 5000000, // 5 Mbps
            mimeType = 'video/webm;codecs=vp9',
            audio = true
        } = options;

        // Get display media
        this.stream = await navigator.mediaDevices.getDisplayMedia({
            video: {
                displaySurface: 'browser', // Tab, window, or monitor
                width: { ideal: 1920 },
                height: { ideal: 1080 },
                frameRate: { ideal: 30 }
            },
            audio: audio ? {
                echoCancellation: true,
                noiseSuppression: true
            } : false,
            preferCurrentTab: true // Prefer current tab
        });

        // Create MediaRecorder
        const options_recorder = {
            mimeType: MediaRecorder.isTypeSupported(mimeType) ? mimeType : 'video/webm',
            videoBitsPerSecond: videoBitsPerSecond
        };

        this.mediaRecorder = new MediaRecorder(this.stream, options_recorder);

        this.mediaRecorder.ondataavailable = (event) => {
            if (event.data.size > 0) {
                this.recordedChunks.push(event.data);
            }
        };

        this.mediaRecorder.onstop = () => {
            this.saveRecording();
        };

        // Start recording
        this.mediaRecorder.start(1000); // Collect data every second

        // Handle stream end (user stops sharing)
        this.stream.getVideoTracks()[0].onended = () => {
            this.stopRecording();
        };
    }

    stopRecording() {
        if (this.mediaRecorder && this.mediaRecorder.state !== 'inactive') {
            this.mediaRecorder.stop();
        }
        if (this.stream) {
            this.stream.getTracks().forEach(track => track.stop());
        }
    }

    saveRecording() {
        const blob = new Blob(this.recordedChunks, { type: 'video/webm' });
        const url = URL.createObjectURL(blob);

        // Create download link
        const a = document.createElement('a');
        a.href = url;
        a.download = `recording_${Date.now()}.webm`;
        a.click();

        // Or save to IndexedDB
        this.saveToIndexedDB(blob);

        // Cleanup
        URL.revokeObjectURL(url);
        this.recordedChunks = [];
    }

    async saveToIndexedDB(blob) {
        const dbName = 'FiratShadowRecordings';
        const storeName = 'recordings';

        return new Promise((resolve, reject) => {
            const request = indexedDB.open(dbName, 1);

            request.onupgradeneeded = (event) => {
                const db = event.target.result;
                if (!db.objectStoreNames.contains(storeName)) {
                    db.createObjectStore(storeName, { keyPath: 'id', autoIncrement: true });
                }
            };

            request.onsuccess = (event) => {
                const db = event.target.result;
                const transaction = db.transaction(storeName, 'readwrite');
                const store = transaction.objectStore(storeName);

                const recording = {
                    blob: blob,
                    timestamp: Date.now(),
                    size: blob.size,
                    type: 'webm'
                };

                store.add(recording);
                resolve();
            };

            request.onerror = () => reject(request.error);
        });
    }

    // Pause/Resume
    pauseRecording() {
        if (this.mediaRecorder && this.mediaRecorder.state === 'recording') {
            this.mediaRecorder.pause();
        }
    }

    resumeRecording() {
        if (this.mediaRecorder && this.mediaRecorder.state === 'paused') {
            this.mediaRecorder.resume();
        }
    }

    // Get recording status
    getStatus() {
        return {
            isRecording: this.mediaRecorder?.state === 'recording',
            isPaused: this.mediaRecorder?.state === 'paused',
            duration: this.recordedChunks.length, // Approximate
            size: this.recordedChunks.reduce((acc, chunk) => acc + chunk.size, 0)
        };
    }
}

// Usage
const recorder = new BrowserRecorder();

// Start recording
document.getElementById('recordBtn').addEventListener('click', async () => {
    await recorder.startScreenRecording({
        videoBitsPerSecond: 8000000, // 8 Mbps for higher quality
        audio: true
    });
});

// Stop recording
document.getElementById('stopBtn').addEventListener('click', () => {
    recorder.stopRecording();
});
```

### 18.3 OBS vs Browser Recording Karşılaştırma

| Özellik | OBS Studio | MediaRecorder API |
|---------|-----------|-------------------|
| **Kurulum** | External app | Browser native |
| **Kalite** | 4K 60fps | 1080p 30fps |
| **Encoding** | GPU (NVENC/AMF) | CPU (VP9/AV1) |
| **Audio** | Multi-source | Single source |
| **Overlay** | ✅ | ❌ |
| **Scene Switch** | ✅ | ❌ |
| **Hotkeys** | ✅ Global | ❌ Tab only |
| **File Size** | Smaller (H.264) | Larger (VP9) |
| **Format** | MP4/MKV/FLV | WebM |
| **Offline** | ✅ | ✅ |
| **Zero Config** | ❌ | ✅ |

**Öneri:** OBS varsa OBS kullan, yoksa MediaRecorder fallback

---

## 19. DEBSİS-STYLE CARD SİSTEMİ (Responsive)

### 19.1 Card Grid Layout

```css
/* web/css/cards.css */

/* CSS Variables */
:root {
  --card-bg: #1e1e1e;
  --card-border: #333;
  --card-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
  --card-radius: 12px;
  --card-padding: 16px;
  --card-hover-transform: translateY(-4px);
  --card-hover-shadow: 0 8px 25px rgba(0, 0, 0, 0.4);
}

/* Card Grid */
.card-grid {
  display: grid;
  gap: 20px;
  padding: 20px;
}

/* Course Cards - Responsive */
.card-grid--courses {
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
}

/* Dashboard Cards */
.card-grid--dashboard {
  grid-template-columns: 1fr;
}

/* Card Base */
.card {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: var(--card-radius);
  padding: var(--card-padding);
  box-shadow: var(--card-shadow);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  cursor: pointer;
  overflow: hidden;
}

.card:hover {
  transform: var(--card-hover-transform);
  box-shadow: var(--card-hover-shadow);
}

/* Card Image */
.card__image {
  width: 100%;
  height: 140px;
  object-fit: cover;
  border-radius: 8px;
  margin-bottom: 12px;
}

/* Card Content */
.card__title {
  font-size: 1.1rem;
  font-weight: 600;
  color: #fff;
  margin-bottom: 8px;
  line-height: 1.3;
}

.card__subtitle {
  font-size: 0.85rem;
  color: #888;
  margin-bottom: 12px;
}

.card__meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 0.8rem;
  color: #666;
}

/* Live Card (Special) */
.card--live {
  border: 2px solid #ff4444;
  animation: pulse-border 2s infinite;
}

@keyframes pulse-border {
  0%, 100% { border-color: #ff4444; }
  50% { border-color: #ff6666; }
}

.card--live .card__badge {
  background: #ff4444;
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
}

/* Recording Card */
.card--recording {
  display: flex;
  align-items: center;
  gap: 16px;
}

.card--recording .card__thumbnail {
  width: 120px;
  height: 68px;
  object-fit: cover;
  border-radius: 6px;
  flex-shrink: 0;
}

.card--recording .card__info {
  flex: 1;
  min-width: 0;
}

.card--recording .card__duration {
  font-size: 0.75rem;
  color: #888;
}

/* Responsive Breakpoints */
@media (min-width: 480px) {
  .card-grid--courses {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (min-width: 768px) {
  .card-grid--dashboard {
    grid-template-columns: 2fr 1fr;
  }

  .card-grid--courses {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (min-width: 1024px) {
  .card-grid--dashboard {
    grid-template-columns: 3fr 1fr;
  }

  .card-grid--courses {
    grid-template-columns: repeat(4, 1fr);
  }
}

@media (min-width: 1440px) {
  .card-grid--courses {
    grid-template-columns: repeat(5, 1fr);
  }
}
```

### 19.2 Card HTML Template

```html
<!-- Course Card -->
<div class="card card--course" data-course-id="240709">
  <img class="card__image" src="/images/course-thumb.jpg" alt="Ders görseli">
  <div class="card__content">
    <span class="card__category">Bilgisayar Programcılığı</span>
    <h3 class="card__title">Internet Programcılığı I</h3>
    <p class="card__subtitle">BTB1104-BTB108</p>
    <div class="card__meta">
      <span class="card__recordings">📹 3 kayıt</span>
      <span class="card__progress">20% tamamlandı</span>
    </div>
  </div>
</div>

<!-- Live Course Card -->
<div class="card card--live" data-course-id="240709">
  <div class="card__header">
    <span class="card__badge">🔴 CANLI</span>
    <span class="card__time">18:00 - 19:30</span>
  </div>
  <div class="card__content">
    <h3 class="card__title">Internet Programcılığı I</h3>
    <p class="card__subtitle">Doç. Dr. Ahmet Yılmaz</p>
  </div>
  <div class="card__actions">
    <button class="btn btn--primary">Katıl</button>
    <button class="btn btn--secondary btn--sazan" data-user-role="student">🤖 Sazan</button>
  </div>
</div>

<!-- Recording Card -->
<div class="card card--recording" data-recording-id="abc123">
  <img class="card__thumbnail" src="/images/thumb.jpg" alt="Kayıt">
  <div class="card__info">
    <h4 class="card__title">Internet Programcılığı I - 24 Şubat</h4>
    <span class="card__duration">1:06:00</span>
  </div>
  <button class="btn btn--play">▶ İzle</button>
</div>
```

---

## 20. TAM ÖZET - TÜM ÖZELLİKLER

| Kategori | Özellik | Durum |
|----------|---------|-------|
| **Auth** | CAS Login | ✅ Planlandı |
| **Dashboard** | Card-based, responsive | ✅ Planlandı |
| **Ders** | Canlı katılım | ✅ Planlandı |
| **Kayıt** | OBS + MediaRecorder fallback | ✅ Planlandı |
| **Kayıt** | Dual recording (Collab + Biz) | ✅ Planlandı |
| **Kayıt** | Otomatik başlat | ✅ Planlandı |
| **Video** | CloudFront playback | ✅ Planlandı |
| **Video** | Chapter sistemi | ✅ Planlandı |
| **Video** | İndirme seçenekleri | ✅ Planlandı |
| **Chat** | Reply, Edit, Delete | ✅ Planlandı |
| **Chat** | React, Pin | ✅ Planlandı |
| **Chat** | Renk ağacı | ✅ Planlandı |
| **Chat** | Collab sync | ✅ Planlandı |
| **Ses** | Whisper transkripsiyon | ✅ Planlandı |
| **Ses** | Hoca konuşması algılama | ✅ Planlandı |
| **Otomasyon** | Sazan.avi 4 seviye | ✅ Planlandı |
| **Otomasyon** | Öğrenci eleme koruması | ✅ Planlandı |
| **Whiteboard** | Excalidraw entegrasyonu | ✅ Planlandı |
| **Rol** | Öğrenci/Öğretmen arayüzü | ✅ Planlandı |
| **Tema** | Dark/Light mode | ✅ Planlandı |
| **Dil** | TR/EN i18n | ✅ Planlandı |
| **Maliyet** | 0 ₺ | ✅ Doğrulandı |

---

**Plan TAMAMLANMIŞTIR. Tüm özellikler araştırıldı ve planlandı.**
