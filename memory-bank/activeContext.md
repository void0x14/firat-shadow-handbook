# Active Context — Fırat Shadow Handbook

## Güncel Durum (Architecture Lock)
Proje, "Pure Metal" felsefesine kilitlendi. Öğrenme sürecini maksimize etmek için Rust'ın en düşük seviye kütüphaneleri (`std::net`) ve tarayıcının saf gücü kullanılacak.

## Yapılanlar
- [x] Tüm modern "dependency ibnelikleri" temizlendi.
- [x] Mimari: Hexagonal + Zero-Dependency olarak revize edildi.
- [x] Frontend tipleme yöntemi: JSDoc (No build step) olarak seçildi.
- [x] `docs/mvp-roadmap.md` saf metal ruhuna göre güncellendi.

## Odak Noktası
- Rust tarafında `TcpListener` tabanlı temel server iskeleti.
- JSDoc ile tipleme yapılmış ilk frontend modülleri.
- Moodle/CAS REST akışının ham HTTP paketleri ile tasarımı.

## Kararlar
1. **No Framework**: Hem backend hem frontend'de framework yok.
2. **JSDoc Safety**: TypeScript'in bağımlılıklarını kurmadan tip güvenliği.
3. **Modular Portability**: Modüler yapı sayesinde her an native katmanlara geçiş imkanı.
