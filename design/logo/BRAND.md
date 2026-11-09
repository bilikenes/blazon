# Blazon — Logo kullanım kılavuzu

**Fikir:** Ön kapağından taze boya damlayan bir klasör — *klasörüne renk ver.*

## Dosyalar (`final/`)

| Dosya | Ne zaman |
|---|---|
| `blazon-symbol.svg` | Ana işaret, 40 px ve üstü |
| `blazon-symbol-small.svg` | 16–32 px (favicon, tepsi, küçük ikon) — kalın boşluk, iki damla |
| `blazon-symbol-black.svg` / `-white.svg` | Tek renk baskı, koyu zemin, filigran |
| `blazon-horizontal.svg` / `-white.svg` | Varsayılan logo: site başlığı, README |
| `blazon-stacked.svg` / `-white.svg` | Kare alanlar: sosyal medya, kurulum ekranı |
| `blazon-wordmark.svg` / `-white.svg` | Yalnızca yazı |
| `web/` | favicon.ico, PNG ikon seti, `site.webmanifest`, `head-snippet.html` |

Kaynak geometri `build_logo.py` içinde; SVG'leri elle değil, betiği düzenleyip yeniden çalıştırarak değiştirin.
Uygulama ikonları: `npx tauri icon design/logo/final/blazon-symbol-1024.png` → ardından `python design/logo/build_icons.py`
(`icon.ico` içindeki 16–32 px kareleri küçük kesimden kurar).

## Renkler

| Ad | HEX | RGB | Kullanım |
|---|---|---|---|
| Erik | `#9B6BDF` | 155 107 223 | Arka kapak |
| Mercan | `#EE7A64` | 238 122 100 | Ön kapak, damlalar, vurgu rengi |
| Mürekkep | `#23262F` | 35 38 47 | Yazı, koyu zemin |
| Kâğıt | `#FFFFFF` | 255 255 255 | Koyu zeminde yazı |

## Kurallar

- **Boşluk:** Logonun çevresinde en az *ön kapak yüksekliğinin yarısı* kadar boş alan bırakın.
- **En küçük boyut:** Sembol 16 px (küçük kesimle), yatay logo 96 px genişlik.
- **Zeminler:** Beyaz, açık gri, Mürekkep ve koyu zeminlerde renkli sembol kullanılabilir.
  **Mercan veya Erik zeminde** renkli sembol kaybolur → beyaz tek renk versiyonu kullanın.
  Fotoğraf üzerinde yeterli kontrast yoksa sembolü Mürekkep karo üzerine alın.
- **Yapmayın:** Renkleri yer değiştirmek, damla eklemek/çıkarmak, gölge/degrade eklemek, logoyu esnetmek veya
  döndürmek, yazıyı başka bir fontla yeniden yazmak.

## Notlar

- Yazı, hazır bir font kullanılmadan yuvarlak uçlu geometrik çizgilerle kuruldu (lisans sorunu yok). Dosyalarda
  kontur (`stroke`) olarak duruyor; matbaa için Inkscape'te *Path → Stroke to Path* ile dış hatta çevrilebilir.
- Marka tescili garanti değildir; yayından önce TÜRKPATENT ve EUIPO veritabanlarında "Blazon" adıyla bir ön arama
  yapılması önerilir.
