# Tech Context

## Core Stack
- **Language:** Python 3.10+
- **Frontend/Backend Framework:** Streamlit
- **HTTP Client:** `requests` (with `requests.Session`)
- **HTML Parser:** `BeautifulSoup4` (`lxml` parser)

## External Services
- **Hosting:** Streamlit Cloud (Community)
- **DNS/Proxy:** Cloudflare
- **Source System:** `https://technic.firat.edu.tr` & `https://obs.firat.edu.tr`

## Development Setup
- **Dependencies:** `streamlit`, `requests`, `beautifulsoup4`, `lxml`
- **Linter:** `ruff` or `flake8`
- **Version Control:** Git (GitHub)

## Constraints
- **CORS:** Handled by server-side requests (Streamlit).
- **Rate Limiting:** Must be careful not to spam school servers (implement basic delays if needed).
