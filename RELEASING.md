# Sürüm çıkarma

## Nasıl çalışıyor

```
npm run release -- 0.2.0  →  git push --follow-tags  →  GitHub Actions derler + imzalar
        →  taslak Release (setup.exe + latest.json)  →  sen "Publish" dersin  →  uygulamalar güncellemeyi görür
```

- Kurulu uygulamalar açıldıktan ~4 sn sonra
  `https://github.com/bilikenes/blazon/releases/latest/download/latest.json` adresine bakar.
- `latest.json` içindeki imza, `tauri.conf.json > plugins.updater.pubkey` ile doğrulanır. İmzası tutmayan
  hiçbir güncelleme kurulmaz.
- Sürüm numarasının tek kaynağı `package.json` (`tauri.conf.json` oradan okur). `npm run release` betiği
  `Cargo.toml`/`Cargo.lock`/`package-lock.json`'ı da eşitler.

## Bir kerelik kurulum

### 1. İmza anahtarını GitHub'a ekle

Anahtar çifti bu bilgisayarda üretildi:

| Dosya | Ne |
|---|---|
| `%USERPROFILE%\.tauri\blazon.key` | **Özel anahtar** — gizli, repoya asla girmez |
| `%USERPROFILE%\.tauri\blazon.key.pub` | Açık anahtar — `tauri.conf.json` içinde zaten var |

Özel anahtarı repo secret'ı olarak ekle (proje kökünde, Git Bash'te):

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/blazon.key
```

Ya da GitHub'da: **Settings → Secrets and variables → Actions → New repository secret**,
ad `TAURI_SIGNING_PRIVATE_KEY`, değer `blazon.key` dosyasının içeriği.

Anahtarın parolası yok, bu yüzden `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secret'ı gerekmez.

### 2. Anahtarı yedekle

`blazon.key` dosyasını bir parola yöneticisine veya şifreli bir yedeğe koy. **Kaybolursa mevcut kullanıcılara bir
daha güncelleme gönderilemez** — yeni anahtarla imzalanan sürümleri eski kurulumlar reddeder, herkesin yeni sürümü
elle indirmesi gerekir.

## Her sürümde

1. `main` dalında, temiz bir çalışma alanıyla:
   ```bash
   npm run release -- 0.2.0
   ```
   Betik sürümü günceller, `Release v0.2.0` commit'ini ve `v0.2.0` tag'ini oluşturur. Push etmez.
2. Yayınla:
   ```bash
   git push --follow-tags
   ```
3. **Actions** sekmesinde *Release* iş akışının bitmesini bekle (~10 dk).
4. **Releases** sayfasındaki taslağı aç: *Değişiklikler* bölümünü doldur, `Blazon_0.2.0_x64-setup.exe` ve
   `latest.json` dosyalarının yüklendiğini kontrol et, **Publish release** de.

Yayınlanana kadar taslak, kullanıcılara görünmez ve güncelleme olarak sunulmaz.

## Yerelde imzalı derleme (isteğe bağlı)

```bash
TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/blazon.key)" npm run tauri build
```

`createUpdaterArtifacts` açık olduğu için imza anahtarı olmadan `tauri build` imza adımında hata verir; yerelde
sadece denemek istiyorsan yukarıdaki gibi anahtarı verin.

## Kod imzalama (henüz yok)

Kurulum dosyası Authenticode ile imzalı değil; Windows SmartScreen ilk sürümlerde "Bilinmeyen yayımcı" uyarısı
gösterir. Kaldırmak için bir kod imzalama hizmeti (ör. Azure Trusted Signing) gerekir — ücretli, ayrı bir karar.
