# Product Context — Fırat Shadow Handbook

## Neden Bu Proje Var?

Fırat Üniversitesi'nin uzaktan eğitim sistemi Debsis (Open LMS / Moodle tabanlı) ve Collab (BigBlueButton) platformu, öğrenci ve öğretmenler için ciddi kullanılabilirlik sorunları yaratıyor. Üniversite IT departmanı bu sorunları çözmüyor veya çözemiyor. Bu proje, IT'ye bağımlı olmadan bu sorunları client/server-side çözümlerle gideriyor.

## Çözülen Sorunlar

### Öğretmen Sorunları
| Sorun | Etki | Çözüm |
|-------|------|-------|
| İki bilgisayar kullanmak zorunda | Fiziksel yük, kurulum karmaşıklığı | OBS WebSocket ile tek bilgisayar |
| Tam ekran slayt paylaşımında ses bozuluyor | Ders kalitesi düşüyor | OBS bağımsız mikrofon kanalı |
| Kayıtlar saatler sonra, 720p geliyor | Öğrenciler tekrar edemez | MediaRecorder/OBS ile anında, yüksek kalite |
| DM bildirimi gelmiyor | Öğrenci sorularını kaçırıyor | Web Push + e-posta bildirimi |

### Öğrenci Sorunları
| Sorun | Etki | Çözüm |
|-------|------|-------|
| Erken girince ses/görüntü gelmiyor | Dersin başını kaçırıyor | Supabase Realtime + auto-reconnect |
| Debsis UI karmaşık, yavaş | Derse geç katılım | Tek tıkla Collab açma |
| Kayıtlara geç erişim | Tekrar yapamıyor | Anında erişilebilir kayıt arşivi |
| Mesaj bildirimi yok | Öğretmenden cevap alamıyor | Realtime chat + push bildirim |

### Sistem Sorunları
| Sorun | Etki | Çözüm |
|-------|------|-------|
| Sınav haftası DB çöküyor | Veri kaybı | Bağımsız Supabase + günlük yedek |

## Nasıl Çalışmalı?

### Kullanıcı Deneyimi Hedefleri
- **Öğrenci:** Dashboard açılır → bugünkü dersler görünür → [Katıl] → Collab açılır. 3 tıkla derse giriş.
- **Öğretmen:** Dashboard → [Dersi Başlat] → Collab açılır + kayıt başlar. 2 tıkla ders başlatma.
- **Bildirim:** Öğrenci soru sorar → öğretmenin telefonuna anlık push bildirim gelir.
- **Kayıt:** Ders biter → 2 dakika içinde kayıt izlenebilir durumda.

### Auth Deneyimi
- Kullanıcı OBS (Öğrenci Bilgi Sistemi) kullanıcı adı + şifresini girer
- Sistem arka planda CAS'a authenticate olur, Moodle session açar
- Kullanıcı bir daha şifre girmez (JWT ile oturum devam eder)
- Debsis'e ayrıca giriş yapmak gerekmez

### Tasarım Prensipleri
- Türkçe öncelikli UI
- Fırat renkleri: lacivert `#1a3a6b`, kırmızı `#c0392b`
- Dark mode (sistem tercihine göre otomatik)
- Mobil responsive (büyük dokunma hedefleri)
- WhatsApp/Slack benzeri chat UI
- Her zaman görünür durum göstergeleri ("Ders canlı 🔴", "Kayıt hazır ✓")
