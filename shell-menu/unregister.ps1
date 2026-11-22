# register.ps1'in yaptığı kaydı geri alır ve klasik menü kayıtlarını yeniden gösterir.
param([string]$InstallDir = "$env:LOCALAPPDATA\Blazon")

$ErrorActionPreference = 'Stop'
$dll = "$InstallDir\blazon_shell_menu.dll"

Get-AppxPackage EnesBilik.Blazon | Remove-AppxPackage

foreach ($key in 'Directory\shell\Blazon', 'Directory\Background\shell\Blazon') {
    $path = "HKCU:\Software\Classes\$key"
    if (Test-Path $path) { Remove-ItemProperty $path -Name LegacyDisable -ErrorAction Ignore }
}

# DLL'i yükleyen vekil süreç onu kilitler; yalnızca bu DLL'i kullanan dllhost kapatılır.
Get-Process dllhost -ErrorAction Ignore |
    Where-Object { try { $_.Modules.FileName -contains $dll } catch { $false } } |
    ForEach-Object { Stop-Process -Id $_.Id -Force; $_.WaitForExit(3000) | Out-Null }

if (Test-Path $dll) { Remove-Item $dll -Force }
if (Test-Path "$InstallDir\blazon-menu.ico") { Remove-Item "$InstallDir\blazon-menu.ico" -Force }
$package = "$InstallDir\menu-package"
if (Test-Path "$package\AppxManifest.xml") { Remove-Item $package -Recurse -Force }