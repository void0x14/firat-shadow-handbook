# Fırat Shadow Handbook - Project Context

## Proje Hakkında
Zero-dependency, portable binary olarak çalışan Rust backend + Vanilla JS frontend ile otonom shadow companion.

## Teknoloji Stack
- **Backend**: Rust (std::net) - Zero external crates
- **Frontend**: Vanilla JS (ESM + JSDoc) - No frameworks
- **Styling**: Native CSS (Variables & Grid)
- **Storage**: Filesystem-based (JSON)

## Önemli URL'ler
| Servis | URL |
|--------|-----|
| CAS Login | `https://jasig.firat.edu.tr/cas/login` |
| Debsis | `https://debsis.firat.edu.tr` |
| Collab | `https://eu.bbcollab.com` |
| OBS WS | `ws://localhost:4455` |

## Dosya Yapısı
```
firat-shadow-handbook/
├── src/                    # Rust backend
│   ├── main.rs            # Entry point, TCP listener
│   ├── http.rs            # HTTP/1.1 request parser
│   ├── handler.rs         # Router & handlers
│   └── config.rs          # Server config
├── web/                    # Frontend (SPA)
│   ├── index.html         # Main HTML
│   ├── js/                # JavaScript modules
│   │   ├── app.js         # Entry point
│   │   ├── components.js  # UI components
│   │   ├── store.js       # Reactive state
│   │   ├── router.js      # Hash routing
│   │   └── i18n.js        # Internationalization
│   └── css/               # Stylesheets
│       ├── base.css       # Reset + variables
│       ├── components.css # UI components
│       └── tokens.css     # Design tokens
├── data/i18n/             # Translation files
│   ├── tr.json
│   └── en.json
└── docs/plan.md          # Implementation plan (detaylı)
```

## Frontend Mimarisi
- **State**: Proxy + CustomEvent (framework'siz reactivity)
- **i18n**: Zero dependency, JSON tabanlı
- **Routing**: Hash-based (#/path)
- **Components**: Template literal + innerHTML

## Backend Mimarisi
- **HTTP Server**: std::net::TcpListener
- **Request Parsing**: Manual HTTP/1.1
- **Routing**: Pattern matching
- **Static Files**: Embedded/FS serving

## Son Güncelleme
2026-02-24 - Faz 0 (Core Skeleton) development
