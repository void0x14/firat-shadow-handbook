# Active Context

## Current Focus
Initial Project Setup & Memory Bank Initialization.

## Recent Changes
- Created GitHub repository `void0x14/firat-shadow-handbook`.
- Established `.gitignore` rules to strictly keep AI docs and config local.
- Defined the "Streamlit Proxy" architecture.

## Active Decisions
1. **Architecture:** Python Backend (Streamlit) will be used to handle `requests` and `BS4` logic.
2. **Deployment:** Streamlit Cloud will host the backend. Cloudflare will manage DNS (`firat.fettanego.net`).
3. **Design:** "Professional Analytics" style (Dark Mode, OLED Black, No Glassmorphism).
4. **Login Logic:** Direct HTTP Requests (Reverse Engineering JASIG/CAS), no Headless Browser (for performance).

## Next Steps
1. Implement the generic `Scraper` class logic.
2. Reverse engineer the JSIG CAS login flow.
3. Build the first "Announcements" module in Streamlit.
