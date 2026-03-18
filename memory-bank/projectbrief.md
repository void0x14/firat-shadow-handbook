# Project Brief — Fırat Shadow Handbook

Fırat Üniversitesi öğrenci ve öğretmenlerinin Debsis/Collab sistemindeki somut sorunlarını çözen, sıfır bütçeyle çalışan, IT bağımsız bir "shadow companion" web uygulaması.

---

## Proje Amacı

Fırat Üniversitesi'nin uzaktan eğitim platformu Debsis (Moodle tabanlı) ve Collab (BigBlueButton) sistemlerindeki eksiklikleri, üniversite IT departmanına bağımlı olmadan, tamamen client/server-side çözümlerle gidermek.

## Hedef Kullanıcılar

- **Öğretmenler** (birincil): Özellikle iki bilgisayar kullanmak zorunda kalan, ses/kayıt sorunları yaşayan öğretim elemanları
- **Öğrenciler** (ikincil): Erken bağlanma sorunları yaşayan, bildirimleri kaçıran, kayıtlara geç erişen öğrenciler

## Temel Gereksinimler

1. **Sıfır bütçe** — tüm servisler ücretsiz tier'da çalışmalı
2. **IT bağımsız** — BBB sunucu konfigürasyonu, API anahtarı, webhook erişimi gerektirmemeli
3. **All-in-one** — tek web sitesi, tarayıcı uzantısı yok, ek uygulama yok
4. **Otonom** — kullanıcıdan mümkün olan en az şey istenmeli
5. **Web öncelikli** — mobil responsive; native app sonraki aşama
6. **Ölçeklenebilir** — 1-2 yıllık proje; modüler mimari

## Kapsam Dışı (Şimdilik)

- Native mobil uygulama
- BBB sunucu tarafı değişiklikleri
- Üniversite IT entegrasyonu
- Ücretli servisler

## Başarı Kriterleri

- Öğretmen tek bilgisayarla ders yapabilir
- Öğrenci tek tıkla Collab'a katılır
- Kayıt ders biter bitmez erişilebilir (BBB'nin saatler süren işlemesine karşı)
- Mesaj bildirimleri anlık çalışır
- Sistem Debsis çökse bile veriler korunur
