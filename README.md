<p align="center">
  <img src="design/logo/final/blazon-horizontal.svg" alt="Blazon" height="96">
</p>

<p align="center">
  <b>Klasörlerine renk ver.</b> · <b>Give your folders some colour.</b><br>
  Windows klasör ikonlarını renklendir ve sembollerle özelleştir.
</p>

---

## Türkçe

**Blazon**, Windows klasörlerine renk ve sembol ekleyerek onları ilk bakışta ayırt etmeni sağlayan küçük bir
uygulama.

- Klasörü pencereye sürükle ya da seç
- 11 hazır renk veya istediğin özel renk
- 9 kategoride yaklaşık 150 sembol, aranabilir
- **Uygula** — ikon anında Explorer'da görünür. **Varsayılana döndür** ile her şey geri alınır.

İkon, klasörün içine gizli bir dosya olarak yazılır; klasörü taşısan veya kopyalasan da ikonu korunur.
Blazon kaldırılsa bile verdiğin ikonlar çalışmaya devam eder.

### Kurulum

[Releases](https://github.com/bilikenes/blazon/releases/latest) sayfasından `Blazon_x.y.z_x64-setup.exe`
dosyasını indirip çalıştır. Yönetici izni gerekmez. Windows 10 (1809+) ve Windows 11 desteklenir.

> İlk sürümler kod imzası taşımadığından Windows SmartScreen "Bilinmeyen yayımcı" uyarısı gösterebilir:
> **Ek bilgi → Yine de çalıştır**.

## English

**Blazon** is a tiny Windows app that colours your folders and stamps them with a symbol, so you can find them at a
glance. Drop a folder on the window, pick a colour and a symbol, hit **Apply**. **Reset** restores the default icon.
The icon is stored inside the folder, so it survives moves and copies — and keeps working even if you uninstall Blazon.

Download the installer from [Releases](https://github.com/bilikenes/blazon/releases/latest). No admin rights needed.

## Geliştirme / Development

Gereksinimler: [Node.js 20+](https://nodejs.org), [Rust](https://rustup.rs), Windows 10/11 (WebView2).

```bash
npm install
npm run tauri dev      # geliştirme
npm run tauri build    # kurulum dosyası: src-tauri/target/release/bundle/nsis/
cargo test --manifest-path src-tauri/Cargo.toml   # Windows shell testleri
```

| Klasör | İçerik |
|---|---|
| `src/` | React arayüzü (önizleme, renk ve sembol seçiciler) |
| `src-tauri/src/folder_icon.rs` | ICO üretimi, `desktop.ini`, Windows Shell API çağrıları |
| `design/logo/` | Logo kaynakları, üretim betikleri ve [kullanım kılavuzu](design/logo/BRAND.md) |
| `scripts/build-icons.mjs` | Sembol kataloğunu Gravity UI ikonlarından üretir |

## Lisans

[MIT](LICENSE) © 2026 Enes Bilik. Semboller [Gravity UI Icons](https://github.com/gravity-ui/icons) (MIT) kütüphanesinden gelir.
