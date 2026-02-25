# Progressive Mirror of Fırat University

University IT systems are broken. Legacy designs, dependency hells, and inefficiency. This is the shadow companion.

### Pure Metal Philosophy
- **Zero Dependencies**: No `node_modules`, no bloated frameworks.
- **Absolute Control**: Rust `std::net` backend. Pure Vanilla ESM frontend.
- **Type Safety via JSDoc**: Development-time confidence without build-time burdens.
- **Modular Hexagonal**: Pluggable architecture designed to avoid technical debt.

### 🛡️ Security-First Design
This project implements **comprehensive security hardening** from day one:

**Backend Security (Rust):**
- ✅ **Path Traversal Prevention** - All file access validated and sanitized
- ✅ **Rate Limiting** - 100 req/min per IP (DoS protection)
- ✅ **Input Validation** - Path, headers, body size limits
- ✅ **Secure Headers** - CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy
- ✅ **CORS Restriction** - Same-origin by default (no wildcard)
- ✅ **Information Leakage Prevention** - No sensitive data in logs

**Frontend Security (Vanilla JS):**
- ✅ **XSS Prevention** - `escapeHtml()` utility, no unsafe `innerHTML`
- ✅ **Parameter Sanitization** - All dynamic content escaped
- ✅ **Zero Framework** - No client-side dependencies to exploit

**See:** [`docs/SECURITY_AUDIT_REPORT.md`](docs/SECURITY_AUDIT_REPORT.md) for full audit.

### Core Mission
Enable students and teachers to conduct courses without the friction of Debsis or Collab. High-performance recording, instant messaging, and autonomous monitoring.

---
"Talk is cheap. Show me the code." — Linus Torvalds
