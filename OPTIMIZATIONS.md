# Optimizasyon Denetim Raporu — Fırat Shadow Handbook

**Tarih**: 2026-03-04  
**Kapsam**: Rust Backend (`src/`) + Frontend (`web/`)  
**Felsefe**: Zero-Dependency, Pure Metal

---

## 1) Optimizasyon Özeti

**Genel Sağlık Durumu**: Orta-Yüksek Risk  
Proje güvenlik odaklı ve doğru çalışıyor, ancak ölçeklenebilirlik ve performans için kritik darboğazlar mevcut. Özellikle thread-per-connection modeli ve rate limiter implementasyonu yüksek yük altında sorun çıkarabilir.

**En Yüksek Etkili 3 İyileştirme**:
1. Thread havuzu ile bağlantı yönetimi (ölçeklenebilirlik için kritik)
2. Rate limiter temizleme mantığının optimize edilmesi (CPU yükü)
3. Security header duplikasyonunun kaldırılması (memory + CPU)

**Değişiklik Yapılmazsa En Büyük Risk**: 
Yüksek eşzamanlı bağlantı sayısında (100+) server yanıt veremez hale gelebilir - thread sayısı sınırsız büyür ve OOM crash oluşabilir.

---

## 2) Bulgular (Öncelikli)

### Finding 1: Thread-Per-Connection Modeli

* **Title**: Sınırsız Thread Spawning
* **Category**: Concurrency
* **Severity**: Critical
* **Impact**: Memory, CPU, Reliability - yük altında crash riski
* **Evidence**: `src/main.rs:93-95`
```rust
thread::spawn(move || {
    handle_connection(stream, addr, &router, &rate_limiter);
});
```
* **Why it's inefficient**: Her bağlantı için yeni thread oluşturuluyor. Thread stack'i ~2MB (Linux default), 100 eşzamanlı bağlantı = 200MB+ memory. Thread creation overhead ~10-50µs. Context switch overhead artar.
* **Recommended fix**: Thread pool (rayon-style veya custom) kullan. Mevcut `std::thread::spawn` yerine, önceden oluşturulmuş N thread havuzu kullan. N = cpu_cores * 2-4.
* **Tradeoffs / Risks**: Thread pool implementasyonu kod karmaşıklığını artırır. Ancak std::sync::mpsc + thread::spawn ile basit bir pool yapılabilir.
* **Expected impact estimate**: Memory -80%, Latency -30%, Throughput +300%
* **Removal Safety**: Safe
* **Reuse Scope**: service-wide

---

### Finding 2: Rate Limiter Cleanup Hot Path'te

* **Title**: Rate Limiter'da O(n) Temizleme Her İstekte
* **Category**: CPU
* **Severity**: High
* **Impact**: CPU kullanımı, latency
* **Evidence**: `src/main.rs:50-51`
```rust
// Clean up old entries
requests.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.window);
```
* **Why it's inefficient**: `HashMap::retain()` O(n) operation. Her istekte tüm map taranıyor. 1000 IP = 1000 comparison her istekte. Rate limit window 60 saniye, bu demek ki 60 saniye içinde tüm entry'ler taranıyor.
* **Recommended fix**: 
  1. Lazy cleanup: Sadece yeni entry eklerken veya expired entry bulunduğunda temizle
  2. Background thread ile periyodik temizleme (her 30 saniyede bir)
  3. Veya TTL-based data structure kullan (BTreeMap + timestamp key)
* **Tradeoffs / Risks**: Lazy cleanup memory'de biraz daha fazla entry tutabilir ama CPU'da büyük kazanç sağlar.
* **Expected impact estimate**: CPU -40% (rate limit hot path'te)
* **Removal Safety**: Safe
* **Reuse Scope**: local file

---

### Finding 3: Security Headers Duplikasyonu

* **Title**: Security Headers İki Kez Ekleniyor
* **Category**: CPU, Memory
* **Severity**: Medium
* **Impact**: Memory allocation, CPU (string operations)
* **Evidence**: 
  - `src/http.rs:79-104` - `Response::add_security_headers()`
  - `src/main.rs:280-295` - `send_response_raw()` içinde aynı header'lar tekrar kontrol ediliyor
```rust
// main.rs:280-295
if !response.headers.iter().any(|(k, _)| k == "X-Frame-Options") {
    headers_string.push_str("X-Frame-Options: DENY\r\n");
}
// ... aynı kontrol http.rs'de de var
```
* **Why it's inefficient**: Her response için 5 header için 10 kez linear search (`iter().any()`). String allocation çift yapılıyor.
* **Recommended fix**: Security headers'ı sadece bir yerde ekle - ya `Response::new()` sırasında ya da `send_response_raw()` sırasında. İkisinde de değil.
* **Tradeoffs / Risks**: Refactoring gerekli ama basit.
* **Expected impact estimate**: CPU -5%, Memory -10% per response
* **Removal Safety**: Safe
* **Reuse Scope**: service-wide

---

### Finding 4: Router Pattern Matching O(n)

* **Title**: Router'da Linear Pattern Scan
* **Category**: Algorithm
* **Severity**: Medium
* **Impact**: Request latency (routing)
* **Evidence**: `src/handler.rs:44-49`
```rust
// Pattern matching for dynamic routes
for (pattern, handler) in routes.iter() {
    if self.match_pattern(pattern, &request.path) {
        return handler(request);
    }
}
```
* **Why it's inefficient**: Exact match sonrası, tüm route'lar linear olarak taranıyor. 20 route = 20 comparison. Her istekte bu yapılıyor.
* **Recommended fix**: 
  1. Prefix trie kullan (static prefix'ler için)
  2. Veya route'ları prefix'e göre grupla, sadece ilgili grubu tara
  3. Wildcard route'ları ayrı bir HashMap'te tut (key = prefix)
* **Tradeoffs / Risks**: Trie implementasyonu kod karmaşıklığı artırır. Basit prefix gruplama yeterli olabilir.
* **Expected impact estimate**: Latency -10% (routing), ölçeklenebilirlik artar
* **Removal Safety**: Safe
* **Reuse Scope**: local file

---

### Finding 5: Cookie Parsing Her İstekte Yeni HashMap

* **Title**: Cookie Parsing'de Gereksiz Allocation
* **Category**: Memory
* **Severity**: Low
* **Impact**: Memory allocation, GC pressure
* **Evidence**: `src/main.rs:627-639`
```rust
fn parse_cookie_header(req: &Request) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    // ... her çağrıda yeni HashMap
}
```
* **Why it's inefficient**: `get_cookie()` her çağrıda yeni HashMap oluşturuyor. Bir request'te 3-4 kez çağrılabilir (MoodleSession, ShadowUser, etc).
* **Recommended fix**: 
  1. Cookie'leri Request struct'ında parse et ve cache'le
  2. Veya `Lazy<HashMap>` pattern kullan
  3. Veya `parse_cookie_header` sonucunu caller'da cache'le
* **Tradeoffs / Risks**: Request struct'ını değiştirmek gerekli.
* **Expected impact estimate**: Memory -5% per request, allocation count -3
* **Removal Safety**: Safe
* **Reuse Scope**: local file

---

### Finding 6: web_root() Dosya Sistemi Kontrolü

* **Title**: Her Static Dosya İsteğinde Path Existence Check
* **Category**: I/O
* **Severity**: Medium
* **Impact**: I/O latency, filesystem calls
* **Evidence**: `src/main.rs:853-865`
```rust
fn web_root() -> PathBuf {
    let from_root = PathBuf::from("web");
    if from_root.exists() {  // filesystem call
        return from_root;
    }
    let from_src = PathBuf::from("../web");
    if from_src.exists() {  // filesystem call
        return from_src;
    }
    PathBuf::from("web")
}
```
* **Why it's inefficient**: Her static dosya isteğinde 2 `exists()` call = 2 filesystem syscall. Statik dosyalar sık istenir (CSS, JS, images).
* **Recommended fix**: 
  1. `web_root()` sonucunu `lazy_static` veya `OnceCell` ile cache'le
  2. Veya startup'ta bir kez hesapla ve global değişkende tut
* **Tradeoffs / Risks**: Basit fix, güvenli.
* **Expected impact estimate**: I/O -100% (bu fonksiyon için), Latency -1-2ms per static request
* **Removal Safety**: Safe
* **Reuse Scope**: service-wide

---

### Finding 7: to_hex() Format String Loop

* **Title**: Hex Encoding'de Format String Kullanımı
* **Category**: CPU
* **Severity**: Low
* **Impact**: CPU (format string parsing)
* **Evidence**: `src/main.rs:671-677`
```rust
fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));  // format! per byte
    }
    out
}
```
* **Why it's inefficient**: `format!` macro her byte için string parsing yapıyor. 24 byte = 24 format! call.
* **Recommended fix**: Lookup table kullan:
```rust
fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}
```
* **Tradeoffs / Risks**: Yok, basit ve daha hızlı.
* **Expected impact estimate**: CPU -50% (bu fonksiyon için)
* **Removal Safety**: Safe
* **Reuse Scope**: local file

---

### Finding 8: Response Headers Linear Search

* **Title**: Header Kontrolü İçin Linear Search
* **Category**: Algorithm
* **Severity**: Low
* **Impact**: CPU (per response)
* **Evidence**: `src/http.rs:81-104` ve `src/main.rs:266-295`
```rust
if !self.headers.iter().any(|(k, _)| k == "X-Frame-Options") {
    // ...
}
```
* **Why it's inefficient**: Her header kontrolü O(n). 5 header = 5 linear search. Response headers genelde az sayıda ama hot path.
* **Recommended fix**: 
  1. Headers için `HashSet` kullan (ama sıra önemli değilse)
  2. Veya header ekleme sırasında flag tut
  3. Veya security headers'ı her zaman ekle, kontrolü kaldır
* **Tradeoffs / Risks**: En basit: kontrolü kaldır, her zaman ekle. HTTP duplicate header'lar genelde sorun değil (veya son değer geçerli olur).
* **Expected impact estimate**: CPU -2% per response
* **Removal Safety**: Safe
* **Reuse Scope**: service-wide

---

### Finding 9: HTML Unescape Gecikmeli Allocation

* **Title**: html_unescape'te Gereksiz String Allocation
* **Category**: Memory
* **Severity**: Low
* **Impact**: Memory allocation
* **Evidence**: `src/infrastructure/collab_scraper_adapter.rs:346-404`
```rust
fn html_unescape(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    // ...
    let mut entity = String::new();  // inner allocation
    // ...
}
```
* **Why it's inefficient**: Entity string her `&` karakterinde yeniden allocate ediliyor. Büyük HTML'lerde (50KB+) bu önemli.
* **Recommended fix**: 
  1. Entity için `SmallVec` veya stack-allocated buffer kullan
  2. Veya entity'yi char iterator ile direkt işle
* **Tradeoffs / Risks**: Daha karmaşık kod ama memory kazancı var.
* **Expected impact estimate**: Memory -10% (HTML parsing sırasında)
* **Removal Safety**: Safe
* **Reuse Scope**: local file

---

### Finding 10: CompositionRoot Her İstekte Yeniden Oluşturuluyor

* **Title**: CompositionRoot Singleton Olmalı
* **Category**: Memory, CPU
* **Severity**: Medium
* **Impact**: Allocation, initialization overhead
* **Evidence**: `src/main.rs:364-366`, `src/main.rs:456-458`, `src/main.rs:596-598`
```rust
// handle_login
let composition = CompositionRoot::new(AdapterConfig::Production);
// validate_session
let composition = CompositionRoot::new(AdapterConfig::Production);
// handle_collab_scrape  
let composition = CompositionRoot::new(AdapterConfig::Production);
```
* **Why it's inefficient**: Her API isteğinde CompositionRoot yeniden oluşturuluyor. Bu da her seferinde yeni adapter instance'ları oluşturuyor. Rustls config her seferinde yeniden build ediliyor (root certs load).
* **Recommended fix**: 
  1. CompositionRoot'u `lazy_static` veya `OnceLock` ile singleton yap
  2. Veya `main()`'de bir kez oluştur ve `Arc` ile paylaş
* **Tradeoffs / Risks**: Adapter'lar stateful ise dikkat gerekli. Mevcut adapter'lar stateless görünüyor.
* **Expected impact estimate**: Memory -30%, CPU -15% (auth endpoints için)
* **Removal Safety**: Needs Verification (adapter state kontrolü gerekli)
* **Reuse Scope**: service-wide

---

### Finding 11: Dead Code - Response::redirect()

* **Title**: Kullanılmayan redirect() Fonksiyonu
* **Category**: Dead Code
* **Severity**: Low
* **Impact**: Maintenance burden, binary size
* **Evidence**: `src/http.rs:66-76`
```rust
#[allow(dead_code)]
pub fn redirect(url: &str) -> Self {
    // ...
}
```
* **Why it's inefficient**: `#[allow(dead_code)]` ile işaretlenmiş, kullanılmıyor. Binary'de yer kaplar (eğer linker kaldırmazsa).
* **Recommended fix**: 
  1. Kullanılacaksa kullan
  2. Kullanılmayacaksa kaldır
* **Tradeoffs / Risks**: Şu an için risk yok, temizlik için yapılabilir.
* **Expected impact estimate**: Minimal
* **Removal Safety**: Safe
* **Reuse Scope**: local file

---

### Finding 12: CAS Adapter'da String Allocation Flood

* **Title**: HTTP Request Building'de Multiple String Concatenation
* **Category**: Memory, CPU
* **Severity**: Medium
* **Impact**: Memory allocation, latency
* **Evidence**: `src/infrastructure/cas_adapter.rs:87-103`
```rust
let mut request = format!(
    "{} {} HTTP/1.1\r\nHost: {}\r\n...",
    method, parsed.path, parsed.authority
);
for (k, v) in headers {
    request.push_str(&format!("{}: {}\r\n", k, v));  // format! per header
}
```
* **Why it's inefficient**: Her header için `format!` allocation. 10 header = 10 allocation + string resize.
* **Recommended fix**: 
  1. `String::with_capacity()` ile başta yeterli kapasite ayır
  2. Veya `write!` macro kullan (in-place write)
```rust
use std::fmt::Write;
let mut request = String::with_capacity(estimated_size);
write!(&mut request, "{} {} HTTP/1.1\r\n...", ...).unwrap();
for (k, v) in headers {
    write!(&mut request, "{}: {}\r\n", k, v).unwrap();
}
```
* **Tradeoffs / Risks**: API değişikliği yok, sadece internal.
* **Expected impact estimate**: Memory -20%, allocation count -10 per CAS request
* **Removal Safety**: Safe
* **Reuse Scope**: local file

---

## 3) Quick Wins (Önce Yap)

| # | Değişiklik | Tahmini Süre | Etki |
|---|-----------|--------------|------|
| 1 | Security header duplikasyonunu kaldır | 15 dk | CPU -5%, Memory -10% |
| 2 | `web_root()` cache'le (OnceLock) | 10 dk | I/O -100% (bu fn) |
| 3 | `to_hex()` lookup table ile değiştir | 10 dk | CPU -50% (bu fn) |
| 4 | CompositionRoot singleton yap | 20 dk | Memory -30%, CPU -15% |
| 5 | Rate limiter lazy cleanup | 20 dk | CPU -40% (rate limit) |

**Toplam Quick Win Etkisi**: CPU ~-30%, Memory ~-20%, I/O azalma

---

## 4) Daha Derin Optimizasyonlar (Sonra Yap)

| # | Değişiklik | Tahmini Süre | Risk | Etki |
|---|-----------|--------------|------|------|
| 1 | Thread pool implementasyonu | 2-4 saat | Medium | Ölçeklenebilirlik kritik |
| 2 | Router prefix trie | 1-2 saat | Low | Latency -10% |
| 3 | Request struct'ta cookie cache | 30 dk | Low | Allocation -3/request |
| 4 | HTML unescape optimization | 30 dk | Low | Memory -10% (HTML) |
| 5 | CAS request builder refactor | 30 dk | Low | Allocation -10/request |

---

## 5) Doğrulama Planı

### Benchmark Stratejisi

1. **Baseline Ölçümü**:
   ```bash
   # wrk ile yük testi
   wrk -t4 -c100 -d30s http://localhost:8080/api/health
   wrk -t4 -c100 -d30s http://localhost:8080/ -s post.lua  # login test
   ```

2. **Profil Araçları**:
   - `cargo flamegraph` - CPU hotspots
   - `valgrind --tool=massif` - Memory allocation
   - `perf record -g` - System-level profiling

3. **Ölçülecek Metrikler**:
   - Requests/sec (throughput)
   - Latency p50, p95, p99
   - Memory usage (RSS)
   - Thread count
   - CPU utilization

4. **Test Senaryoları**:
   - Static file serving (1000 concurrent)
   - API endpoints (auth flow)
   - Rate limiter stress test
   - Long-running stability test (1 hour)

### Before/After Karşılaştırma

| Metrik | Before (Expected) | After Target |
|--------|-------------------|--------------|
| Throughput (req/s) | ~500 | ~2000+ |
| Latency p99 | ~100ms | ~30ms |
| Memory (100 conn) | ~200MB | ~50MB |
| Thread count | Unlimited | ~8-16 |

---

## 6) Örnek Patch'ler

### Patch 1: web_root() Cache

```rust
use std::sync::OnceLock;

static WEB_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn web_root() -> PathBuf {
    WEB_ROOT.get_or_init(|| {
        let from_root = PathBuf::from("web");
        if from_root.exists() {
            return from_root;
        }
        let from_src = PathBuf::from("../web");
        if from_src.exists() {
            return from_src;
        }
        PathBuf::from("web")
    }).clone()
}
```

### Patch 2: to_hex() Lookup Table

```rust
fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}
```

### Patch 3: Rate Limiter Lazy Cleanup

```rust
impl RateLimiter {
    fn allow(&self, ip: IpAddr) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        // Get current entry
        if let Some((count, timestamp)) = requests.get(&ip) {
            // Check if expired
            if now.duration_since(*timestamp) >= self.window {
                // Lazy cleanup: remove expired entry
                requests.remove(&ip);
            } else if *count >= self.limit {
                return false;
            }
        }
        
        // Periodic cleanup: only every N requests
        if requests.len() > self.limit as usize * 2 {
            requests.retain(|_, (_, ts)| now.duration_since(*ts) < self.window);
        }
        
        let count = requests.get(&ip).map(|(c, _)| *c).unwrap_or(0);
        requests.insert(ip, (count + 1, now));
        true
    }
}
```

---

## Ek Notlar

### Zero-Dependency Kısıtı

Proje "zero-dependency" felsefesine sahip, bu nedenle:
- Thread pool için `rayon` veya `tokio` kullanılamaz
- Lazy static için `lazy_static` crate kullanılamaz (ancak `std::sync::OnceLock` var - Rust 1.70+)
- HTTP parser için external crate kullanılamaz

Bu kısıtlar altında optimizasyonlar std library ile yapılmalıdır.

### Güvenlik Önceliği

Proje güvenlik odaklı olduğundan, optimizasyonlar güvenlik özelliklerini zayıflatmamalı:
- Rate limiting davranışı korunmalı
- Security headers eksiksiz kalmalı
- Input validation (path, headers) korunmalı
- CSRF koruması korunmalı

---

**Raporu Hazırlayan**: Cascade Optimization Auditor  
**Versiyon**: 1.0
