<#
  Menü paketini (blazon-menu.msix) oluşturur ve yalnızca bu iş için üretilen bir sertifikayla imzalar.
  İmzadan hemen sonra gizli anahtar silinir: aynı sertifikayla bir daha hiçbir şey imzalanamaz.
  Yalnızca AppxManifest.xml veya logolar değişince çalıştırılır. Yeni sertifika yeni kullanıcı onayı
  demektir; kurulu menüler eski paketle çalışmayı sürdürür.
#>
$ErrorActionPreference = 'Stop'
$repo = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$out = "$PSScriptRoot\signed"
$stage = Join-Path ([IO.Path]::GetTempPath()) "blazon-menu-$([guid]::NewGuid())"
$kit = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.*\x64\makeappx.exe" |
    Sort-Object { [version]$_.Directory.Parent.Name } | Select-Object -Last 1
if (-not $kit) { throw 'Windows SDK (makeappx) bulunamadı' }
$bin = $kit.DirectoryName

[xml]$manifest = [IO.File]::ReadAllText("$PSScriptRoot\AppxManifest.xml")
$publisher = $manifest.Package.Identity.Publisher

New-Item -ItemType Directory -Force $out, "$stage\Assets" | Out-Null
$cert = $null
try {
    Copy-Item "$PSScriptRoot\AppxManifest.xml" $stage
    foreach ($name in 'StoreLogo', 'Square150x150Logo', 'Square44x44Logo') {
        Copy-Item "$repo\src-tauri\icons\$name.png" "$stage\Assets"
    }
    # /nv: blazon.exe pakette değil, kurulum klasöründe.
    & "$bin\makeappx.exe" pack /d $stage /p "$out\blazon-menu.msix" /nv /o | Out-Null
    if ($LASTEXITCODE) { throw 'makeappx başarısız' }

    # Yalnızca kod imzalama; CA değil, anahtar dışa aktarılamaz.
    $cert = New-SelfSignedCertificate -Type Custom -Subject $publisher -FriendlyName 'Blazon sağ tık menüsü' `
        -KeyUsage DigitalSignature -KeyAlgorithm RSA -KeyLength 3072 -HashAlgorithm SHA256 `
        -KeyExportPolicy NonExportable -NotAfter (Get-Date).AddYears(10) -CertStoreLocation 'Cert:\CurrentUser\My' `
        -TextExtension '2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}'
    & "$bin\signtool.exe" sign /fd SHA256 /sha1 $cert.Thumbprint "$out\blazon-menu.msix"
    if ($LASTEXITCODE) { throw 'signtool başarısız' }
    Export-Certificate -Cert $cert -FilePath "$out\blazon-menu.cer" -Type CERT | Out-Null
} finally {
    if ($cert) {
        Remove-Item "Cert:\CurrentUser\My\$($cert.Thumbprint)" -DeleteKey
        # New-SelfSignedCertificate, ara CA deposuna da bir kopya bırakır.
        Remove-Item "Cert:\CurrentUser\CA\$($cert.Thumbprint)" -ErrorAction Ignore
    }
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
}
Write-Host "İmzalandı, gizli anahtar silindi: $($cert.Thumbprint)"