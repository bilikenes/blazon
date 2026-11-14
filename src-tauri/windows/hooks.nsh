; Klasör sağ tık menüsüne "Blazon ile ikonu değiştir" ekler (HKCU, yönetici izni gerekmez).

!macro NSIS_HOOK_POSTINSTALL
  ; Klasöre sağ tık
  WriteRegStr HKCU "Software\Classes\Directory\shell\Blazon" "" "Blazon ile ikonu değiştir"
  WriteRegStr HKCU "Software\Classes\Directory\shell\Blazon" "Icon" "$INSTDIR\${MAINBINARYNAME}.exe"
  WriteRegStr HKCU "Software\Classes\Directory\shell\Blazon\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'

  ; Klasörün içindeki boş alana sağ tık
  WriteRegStr HKCU "Software\Classes\Directory\Background\shell\Blazon" "" "Blazon ile ikonu değiştir"
  WriteRegStr HKCU "Software\Classes\Directory\Background\shell\Blazon" "Icon" "$INSTDIR\${MAINBINARYNAME}.exe"
  WriteRegStr HKCU "Software\Classes\Directory\Background\shell\Blazon\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%V"'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey HKCU "Software\Classes\Directory\shell\Blazon"
  DeleteRegKey HKCU "Software\Classes\Directory\Background\shell\Blazon"
!macroend
