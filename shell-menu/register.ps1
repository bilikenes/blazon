<#
  Sağ tık menüsünü bu bilgisayara kaydeder (geliştirme için, imzasız).
  Geliştirici modu açık ve Blazon kurulu olmalı. Yönetici izni gerekmez.
  Geri almak için: unregister.ps1
#>
param([string]$InstallDir = "$env:LOCALAPPDATA\Blazon")

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$repo = Split-Path $root -Parent

if (-not (Test-Path "$InstallDir\blazon.exe")) { throw "Blazon bulunamadı: $InstallDir\blazon.exe" }

Push-Location $root   # .cargo/config.toml (crt-static) çalışma klasöründen okunur
try {
    cargo build --release --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'cargo build başarısız' }
} finally { Pop-Location }

& "$root\unregister.ps1" -InstallDir $InstallDir

Copy-Item "$root\target\x86_64-pc-windows-msvc\release\blazon_shell_menu.dll" $InstallDir -Force
Copy-Item "$root\blazon-menu.ico" $InstallDir -Force

$package = "$InstallDir\menu-package"
New-Item -ItemType Directory -Force "$package\Assets" | Out-Null
foreach ($name in 'StoreLogo', 'Square150x150Logo', 'Square44x44Logo') {
    Copy-Item "$repo\src-tauri\icons\$name.png" "$package\Assets" -Force
}
Copy-Item "$root\package\AppxManifest.xml" $package -Force

Add-AppxPackage -Register "$package\AppxManifest.xml" -ExternalLocation $InstallDir

# Klasik kayıtlar silinmez, yalnızca gizlenir; "Daha fazla seçenek göster"de ikinci kez görünmesin.
foreach ($key in 'Directory\shell\Blazon', 'Directory\Background\shell\Blazon') {
    $path = "HKCU:\Software\Classes\$key"
    if (Test-Path $path) { Set-ItemProperty $path -Name LegacyDisable -Value '' }
}

Get-AppxPackage EnesBilik.Blazon | Select-Object PackageFullName, Status, IsDevelopmentMode | Format-List
Write-Host 'Kaydedildi. Menüde görünmezse Explorer penceresini kapatıp yeniden aç.'