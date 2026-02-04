# Araştırma ve Mimari Planı: Operasyon Gölge Vekil (Shadow Proxy) - Final Master v15 (Operational)

## 1. Teknik Gerçeklik (Evasion & Survival)
- **Survival:** iOS 30sn kısıtlaması.
- **Data Safety:** Yarım kalan işlem = Veri kaybı riski.
- **Maintenance:** Büyüyen DB ve ölü session'lar performansı düşürür.

## 2. Mimari: "The Autonomous Shadow"

### Sütun A: "Identity & Access" (Kimlik ve Erişim)
- **Safe UA Pinning:** Cihazın kendi OS ailesinden seçilen ve `Sec-Ch-Ua` ile uyumlu UA.
- **Proxy Gateway:** Cloudflare Worker, trafiği gerektiğinde Residential Proxy (Konut IP'leri) üzerinden çevirir.
- **Kill Switch:** CF Worker KV üzerinden yönetilen "Acil Durdurma Anahtarı". (Tepki süresi: 10ms).

### Sütun B: "The 29-Second Rush" (Zamanla Yarış)
*iOS Kısıtlamasına Çözüm: "Heartbeat Protocol"*
- **Mekanizma:**
    1.  **Delta Check (xxHash):** 3 saniyede değişim kontrolü.
    2.  **Heartbeat:** Her 5 saniyede bir MMKV'ye "Yaşıyorum: [timestamp]" yazar.
    3.  **Telemetry:** Bir sonraki uyanışta, "Last Heartbeat" ile "Start Time" farkına bakarak iOS'un bizi kaçıncı saniyede öldürdüğünü raporlar.
    4.  **Chunked Write:** Veriler 5'erli paketler halinde commit edilir.

### Sütun C: "The Self-Healing Vault" (Kendi Kendini Onaran Depo)
- **Storage:** `react-native-mmkv` (Encrypted).
- **Janitor Service:** Her başarılı işlem sonrası `created_at < 30_Days` verileri otomatik silinir.
- **Offline-First:** UI, WatermelonDB'ye reaktif (Observable) bağlıdır. İnternet yoksa bile spinner dönmez, son veri ışık hızında gelir.

## 3. Yol Haritası (Execution Roadmap)

### Faz 1: "The Core Identity" (Faz 1)
*Hedef: Güvenli Giriş & UA Yönetimi*
1.  **Safe UA Generator:** Doğru OS maskelemesi.
2.  **MMKV Encryption:** Güvenli session saklama.
3.  **Visual Monitor:** Gizli WebView debug arayüzü.

### Faz 2: "The Resurrector" (Faz 2)
*Hedef: Uyanış ve Proxy*
1.  **SLC Service:** Konum tabanlı uyanış.
2.  **Kill Switch & Jitter:** Cloudflare KV entegrasyonu.

### Faz 3: "The Vault" (Faz 3)
*Hedef: Veri ve Temizlik*
1.  **Hash Scanner:** xxHash delta kontrolü.
2.  **Janitor:** Otomatik temizlikçi.

## 5. Uygulama Onayı
Plan; Kill Switch, Telemetry, Janitor ve Offline-First Reactive UI ile tam otonom hale gelmiştir.
Onay sonrası **Faz 1: Prototip** kodlamasına başlanacaktır.
