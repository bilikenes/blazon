; Klasör sağ tık menüsüne "Blazon ile ikonu değiştir" ekler (HKCU, yönetici izni gerekmez).
; Windows 11'de kullanıcı isterse komut ana menüye de eklenir; mantığı src/menu.rs'te.

!define BLAZON_MENU_DIR "${__FILEDIR__}\..\..\shell-menu"

!macro BLAZON_TEXT VAR TR EN
  ${If} $LANGUAGE = 1055
    StrCpy ${VAR} "${TR}"
  ${Else}
    StrCpy ${VAR} "${EN}"
  ${EndIf}
!macroend

; blazon.exe --menu ACTION çalıştırır, çıkış kodu $0'a yazılır (başlatılamazsa -1).
!macro BLAZON_MENU ACTION
  StrCpy $0 -1
  ClearErrors
  ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --menu ${ACTION}' $0
  ClearErrors
!macroend

!macro NSIS_HOOK_PREINSTALL
  ; Menü açıksa DLL yüklü olabilir: yüklü DLL'in üzerine yazılamaz ama adı değiştirilebilir.
  Delete "$INSTDIR\blazon_menu_old_*.tmp"
  ${If} ${FileExists} "$INSTDIR\blazon_shell_menu.dll"
    System::Call 'kernel32::GetTickCount()i.r0'
    Rename "$INSTDIR\blazon_shell_menu.dll" "$INSTDIR\blazon_menu_old_$0.tmp"
  ${EndIf}
  ClearErrors
  File "${BLAZON_MENU_DIR}\target\x86_64-pc-windows-msvc\release\blazon_shell_menu.dll"
  File "${BLAZON_MENU_DIR}\package\signed\blazon-menu.msix"
  File "${BLAZON_MENU_DIR}\blazon-menu.ico"
  Delete "$INSTDIR\blazon_menu_old_*.tmp"
  ClearErrors
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Klasöre sağ tık
  WriteRegStr HKCU "Software\Classes\Directory\shell\Blazon" "" "Blazon ile ikonu değiştir"
  WriteRegStr HKCU "Software\Classes\Directory\shell\Blazon" "Icon" "$INSTDIR\blazon-menu.ico"
  WriteRegStr HKCU "Software\Classes\Directory\shell\Blazon\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'

  ; Klasörün içindeki boş alana sağ tık
  WriteRegStr HKCU "Software\Classes\Directory\Background\shell\Blazon" "" "Blazon ile ikonu değiştir"
  WriteRegStr HKCU "Software\Classes\Directory\Background\shell\Blazon" "Icon" "$INSTDIR\blazon-menu.ico"
  WriteRegStr HKCU "Software\Classes\Directory\Background\shell\Blazon\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%V"'

  ; Windows 11 ana menüsü: yalnızca elle yapılan kurulumda ve seçenek kapalıyken sorulur.
  ${IfNot} ${Silent}
  ${AndIf} $PassiveMode <> 1
  ${AndIf} $UpdateMode <> 1
    !insertmacro BLAZON_MENU "status"
    ${If} $0 = 1
      !insertmacro BLAZON_TEXT $1 \
        "Blazon doğrudan sağ tık menüsünde görünsün mü?$\r$\n$\r$\nBunun için:$\r$\n• Yönetici onayı gerekir (bir kez).$\r$\n• Blazon'un sertifikası bu bilgisayarın $\"Güvenilen Kişiler$\" deposuna eklenir. Windows bu sertifikayla imzalanmış uygulama paketlerine güvenir; sertifikanın gizli anahtarı silinmiştir, yalnızca bu menü paketi için kullanılmıştır.$\r$\n• Blazon'u kaldırırken sertifika da kaldırılabilir.$\r$\n$\r$\nHayır dersen Blazon, $\"Daha fazla seçenek göster$\" menüsünde (Shift + sağ tık) görünür. Bu ayarı sonra Blazon'un Hakkında panelinden değiştirebilirsin." \
        "Show Blazon directly in the right-click menu?$\r$\n$\r$\nThis requires:$\r$\n• Administrator approval (once).$\r$\n• Adding Blazon's certificate to this computer's $\"Trusted People$\" store. Windows will trust app packages signed with it; its private key has been deleted and was only used for this menu package.$\r$\n• The certificate can be removed when you uninstall Blazon.$\r$\n$\r$\nIf you choose No, Blazon appears under $\"Show more options$\" (Shift + right-click). You can change this later in Blazon's About panel."
      ${If} ${Cmd} `MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2 "$1" IDYES`
        !insertmacro BLAZON_MENU "enable"
        ${If} $0 = 1223
          !insertmacro BLAZON_TEXT $1 \
            "Yönetici onayı verilmedi. Blazon, $\"Daha fazla seçenek göster$\" menüsünde görünecek." \
            "Administrator approval was not given. Blazon will appear under $\"Show more options$\"."
          MessageBox MB_OK|MB_ICONINFORMATION "$1"
        ${ElseIf} $0 <> 0
          IntFmt $0 "0x%08X" $0
          !insertmacro BLAZON_TEXT $1 \
            "Blazon kuruldu ama ana sağ tık menüsüne eklenemedi ($0). $\"Daha fazla seçenek göster$\" menüsünde görünecek." \
            "Blazon is installed, but could not be added to the main right-click menu ($0). It will appear under $\"Show more options$\"."
          MessageBox MB_OK|MB_ICONINFORMATION "$1"
        ${EndIf}
      ${EndIf}
    ${EndIf}
  ${EndIf}

  ; Ana menü açıksa yukarıda yeniden yazılan klasik kayıtlar gizli kalsın.
  !insertmacro BLAZON_MENU "sync"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Güncelleme sırasında çalışan kaldırma, menü seçeneğine dokunmaz.
  ${If} $UpdateMode <> 1
    !insertmacro BLAZON_MENU "disable-keep-cert"
    ${IfNot} ${Silent}
    ${AndIf} $PassiveMode <> 1
      !insertmacro BLAZON_MENU "cert-status"
      ${If} $0 = 0
        !insertmacro BLAZON_TEXT $1 \
          "Blazon'un sağ tık menüsü sertifikası da bu bilgisayardan kaldırılsın mı? Yönetici onayı gerekir." \
          "Also remove Blazon's right-click menu certificate from this computer? Administrator approval is required."
        ${If} ${Cmd} `MessageBox MB_YESNO|MB_ICONQUESTION "$1" IDYES`
          !insertmacro BLAZON_MENU "disable"
        ${EndIf}
      ${EndIf}
    ${EndIf}

    ; DLL'i tutan vekil süreç paketle birlikte kapanır; geç kapanırsa birkaç kez dene.
    ${For} $1 1 10
      ClearErrors
      Delete "$INSTDIR\blazon_shell_menu.dll"
      ${IfNot} ${Errors}
        ${Break}
      ${EndIf}
      Sleep 300
    ${Next}
    Delete "$INSTDIR\blazon-menu.msix"
    Delete "$INSTDIR\blazon-menu.ico"
    Delete "$INSTDIR\blazon_menu_old_*.tmp"
    ClearErrors
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey HKCU "Software\Classes\Directory\shell\Blazon"
  DeleteRegKey HKCU "Software\Classes\Directory\Background\shell\Blazon"
!macroend