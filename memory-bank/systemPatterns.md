# System Patterns

## Architecture: The Shadow Proxy

```mermaid
graph TD
    User[Student Device] -->|HTTPS| CF[Cloudflare DNS]
    CF -->|Route| Streamlit[Streamlit App (Python)]
    
    subgraph Streamlit Backend
        UI[Streamlit UI]
        Session[Session State (RAM)]
        Scraper[Requests/BS4 Engine]
    end
    
    UI -->|Input Credentials| Session
    Session -->|Credentials| Scraper
    
    Scraper -->|HTTP POST| CAS[Fırat CAS/JASIG Login]
    CAS -->|Session Cookie| Scraper
    Scraper -->|Get Data| OBS[OBS System]
    OBS -->|HTML| Scraper
    Scraper -->|Parsed Data| UI
```

## Key Technical Decisions
1. **State Management:** `st.session_state` will be used to hold the session cookies during the user's visit.
2. **Security:** No database. Credentials are ephemeral.
3. **Error Handling:** Graceful degradation. If OBS is down, show "Source System Offline" instead of crashing.
4. **UI Pattern:** Single Page Application (SPA) feel with Streamlit components.

## Code Structure Pattern
- `app.py`: Main entry point (UI).
- `core/`:
    - `scraper.py`: All HTTP/Parsing logic.
    - `auth.py`: Login handling.
    - `constants.py`: URLs and Headers.
- `utils/`: Helper functions.
