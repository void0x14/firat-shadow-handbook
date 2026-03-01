---
story_id: 3-1
title: Native WebSocket
eptic: Live Engine & Media
status: ready-for-dev
created: 2026-02-28
---

# Story 3.1: Native WebSocket

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a backend developer,
I want an RFC 6455 compliant WebSocket implementation,
so that real-time bidirectional communication can be established without external dependencies.

## Acceptance Criteria

1. **RFC 6455 Handshake Compliance**: Sunucu, WebSocket el sıkışma (handshake) isteklerini RFC 6455 standardına uygun olarak kabul etmeli ve `Sec-WebSocket-Accept` header'ını doğru şekilde hesaplamalıdır.
2. **Frame Parsing & Encoding**: WebSocket frame'leri (text, binary, close, ping, pong) doğru şekilde parse edilebilmeli ve oluşturulabilmelidir. Masking/unmasking mantığı çalışmalıdır.
3. **Connection Lifecycle Management**: WebSocket bağlantıları açılıp kapatılabilmeli, bağlantı durumu takip edilebilmeli ve temiz bir şekilde sonlandırılabilmelidir.
4. **Message Routing**: Gelen WebSocket mesajları, uygulama katmanına (application use cases) yönlendirilebilmeli ve cevaplar geri gönderilebilmelidir.
5. **Zero External Dependency**: WebSocket implementasyonu `std::net` kullanılarak yapılmalı, harici crate (örn: `tokio-tungstenite`, `websocket`) kullanılmamalıdır.
6. **Hexagonal Architecture Compliance**: WebSocket altyapısı, mevcut hexagonal mimariye uygun şekilde port-adapter pattern'i ile organize edilmelidir.

## Tasks / Subtasks

- [ ] WebSocket handshake implementasyonu (AC: 1)
  - [ ] HTTP upgrade request parsing
  - [ ] Sec-WebSocket-Key validation ve Sec-WebSocket-Accept hesaplama
  - [ ] 101 Switching Protocols response
- [ ] WebSocket frame parser/encoder (AC: 2)
  - [ ] Frame header parsing (FIN, RSV, opcode, MASK, payload length)
  - [ ] Masking/unmasking algorithm
  - [ ] Text, binary, close, ping, pong frame desteği
- [ ] Connection management (AC: 3)
  - [ ] Connection state enum (Connecting, Open, Closing, Closed)
  - [ ] Connection lifecycle event handling
  - [ ] Graceful shutdown
- [ ] Hexagonal port-adapter yapısı (AC: 4, 6)
  - [ ] `domain::ports::WebSocketPort` trait tanımlama
  - [ ] `infrastructure::WebSocketAdapter` implementasyonu
  - [ ] `application` katmanında message routing/use case entegrasyonu
- [ ] Testing (AC: 1-6)
  - [ ] Handshake unit testleri
  - [ ] Frame encode/decode testleri
  - [ ] Integration test: Gerçek WebSocket client ile bağlantı testi
  - [ ] Cargo test suite: Tüm testler yeşil

## Dev Notes

### 🔬 CRITICAL ARCHITECTURAL CONTEXT

Bu story, **zero-dependency** prensibine sıkı sıkıya bağlı kalmalıdır. Mevcut proje yapısında harici async runtime (tokio/async-std) yoktur ve eklenmemelidir. WebSocket implementasyonu, mevcut multi-threaded `std::net::TcpListener` yapısına entegre edilmelidir.

### WebSocket RFC 6455 Özeti

**Handshake:**
```
Client Request:
GET /chat HTTP/1.1
Host: server.example.com
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13

Server Response:
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

**Key Accept Algoritması:**
1. Client key'i al (base64 encoded 16-byte random)
2. Magic string ekle: "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
3. SHA-1 hash
4. Base64 encode

**Frame Yapısı:**
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-------+-+-------------+-------------------------------+
|F|R|R|R| opcode|M| Payload len |    Extended payload length    |
|I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
|N|V|V|V|       |S|             |   (if payload len==126/127)   |
| |1|2|3|       |K|             |                               |
+-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
|     Extended payload length continued, if payload len == 127  |
+ - - - - - - - - - - - - - - - +-------------------------------+
|                               |Masking-key, if MASK set to 1  |
+-------------------------------+-------------------------------+
| Masking-key (continued)       |          Payload Data         |
+-------------------------------- - - - - - - - - - - - - - - - +
:                     Payload Data continued ...                :
+ - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - +
|                     Payload Data continued ...                |
+---------------------------------------------------------------+
```

### Technical Requirements

- **Opcode Değerleri:**
  - `0x1`: Text frame
  - `0x2`: Binary frame
  - `0x8`: Connection close
  - `0x9`: Ping
  - `0xA`: Pong

- **Payload Length:**
  - `0-125`: Doğrudan değer
  - `126`: Sonraki 2 byte (u16)
  - `127`: Sonraki 8 byte (u64)

- **Masking:** Client → Server mesajları maskelenir (MASK bit = 1). Algoritma: `decoded[i] = encoded[i] XOR mask_key[i % 4]`

### Architecture Compliance

**Hexagonal Sınır:**
- `domain::ports::WebSocketPort`: Trait tanımı (send/receive message contract)
- `infrastructure::WebSocketAdapter`: TCP/WebSocket implementasyonu
- `application`: Use case'ler port üzerinden çalışır

**Integration Pattern:**
Mevcut HTTP server yapısına şu şekilde entegre edilmeli:
1. HTTP router'da WebSocket upgrade path tanımla (`/ws`)
2. Upgrade isteği geldiğinde, connection WebSocket adapter'a devredilir
3. WebSocket adapter, domain port üzerinden application use case'lere mesaj iletir

**Threading Model:**
- Her WebSocket connection için yeni thread spawn edilebilir (mevcut HTTP modeli ile tutarlı)
- Alternatif: Non-blocking I/O ile mevcut thread pool'da yönetim

### Library / Framework Requirements

**Zero New Dependencies:** Mevcut bağımlılıklar yeterlidir:
- `serde` / `serde_json`: Mesaj serialization/deserialization
- `thiserror`: Error handling
- `chrono`: Timestamp handling
- `rustls` / `webpki-roots`: Gerekirse WSS (WebSocket Secure) için

**Not:** WSS implementasyonu bu story kapsamında **opsiyoneldir**. Temel WebSocket önceliklidir.

### File Structure Requirements

**Yeni Dosyalar:**
```
src/
├── domain/
│   ├── ports/
│   │   ├── mod.rs          # WebSocketPort export ekle
│   │   └── websocket_port.rs    # YENİ
│   └── websocket/
│       └── mod.rs          # YENİ - Domain types (Message, CloseCode, etc.)
├── infrastructure/
│   ├── mod.rs              # WebSocketAdapter export ekle
│   └── websocket_adapter.rs     # YENİ - RFC 6455 implementasyonu
├── application/
│   └── ws_message_usecase.rs    # YENİ - Message routing use case
└── handler.rs              # WebSocket upgrade route ekle
```

**Değiştirilmesi Beklenen:**
- `src/handler.rs`: WebSocket upgrade route tanımı
- `src/main.rs`: WebSocket adapter initialization
- `src/application/mod.rs`: Yeni use case export
- `src/domain/ports/mod.rs`: WebSocketPort export
- `src/infrastructure/mod.rs`: WebSocketAdapter export

### Testing Requirements

**Unit Tests:**
- Handshake key accept calculation doğrulama
- Frame encode/decode round-trip testleri
- Masking/unmasking doğrulama
- Close frame parsing

**Integration Test:**
- Gerçek browser WebSocket client ile bağlantı testi
- veya `websocat` CLI tool ile test
- Echo server testi: Client mesaj gönderir, server aynı mesajı geri gönderir

**Acceptance Criteria Coverage:**
- Her AC için en az 1 test case
- `cargo test` çalıştırıldığında tüm testler yeşil olmalı
- Code coverage: %80+ hedeflenebilir (bu story için zorunlu değil)

### Previous Story Intelligence (Epic 2)

**Story 2-3 (Hexagonal Adapters) Learnings:**
- Port trait'leri `Clone` trait'i implement etmeli (testler için kolay mock oluşturma)
- Composition root'ta adapter seçimi tek noktada yapılmalı
- `Box<dyn Port>` kullanımı dependency injection için etkili

**Story 2-2 (Collab Scraper) Learnings:**
- HTML parsing'de edge case'ler önemli (case-insensitive, quote-aware parsing)
- Error handling'de `thiserror` kullanımı standardized
- HTTP timeout ve redirect handling kritik

**Git Intelligence Summary:**
- Son commitler: `fix: Resolve all compiler warnings`, `fix(2-3-hexagonal-adapters): code review issues resolved`
- Proje, compiler warning'lerin sıfır olmasına önem veriyor
- Code review süreci aktif kullanılıyor
- Tüm değişiklikler `cargo test` ile validate ediliyor

### Project Structure Notes

**Alignment:**
- Mevcut `src/` yapısı (`domain`, `application`, `infrastructure`) korunmalı
- WebSocket implementasyonu da bu yapıya uymalı
- `std::net` tabanlı implementasyon mevcut HTTP server ile uyumlu

**Naming Conventions:**
- Port trait'leri: `XxxPort` (örn: `WebSocketPort`)
- Adapter implementasyonları: `XxxAdapter` (örn: `WebSocketAdapter`)
- Use case'ler: `XxxUsecase` (örn: `WsMessageUsecase`)

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-3-Live-Engine--Media]
- [Source: _bmad-output/planning-artifacts/project-context.md]
- [Source: src/main.rs]
- [Source: src/domain/ports/mod.rs]
- [Source: src/infrastructure/mod.rs]
- [RFC 6455: The WebSocket Protocol](https://tools.ietf.org/html/rfc6455)

## Dev Agent Record

### Agent Model Used

(To be filled during implementation)

### Debug Log References

- create-story workflow: story auto-discovery from sprint-status (`3-1-native-websocket`)
- artifact analysis: epics + project-context + source tree
- epic-3 is first story in epic, marking as in-progress

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story focuses on zero-dependency WebSocket implementation with RFC 6455 compliance
- Hexagonal architecture pattern to be followed (established in Story 2-3)

### File List

(To be filled during implementation)

---

**Next Steps After Implementation:**
1. Run `cargo test` to verify all tests pass
2. Test with real WebSocket client (browser or websocat)
3. Run code-review workflow
4. Update this story file with completion notes
