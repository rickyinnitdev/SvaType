; NSIS's supported language set has no Khmer. Keep the standard English wizard
; chrome and present our instructions and custom messages in both languages.
!include "${__FILEDIR__}\welcome.nsh"

!macro NSIS_HOOK_POSTUNINSTALL
  ; Match Tauri's explicit, initially unchecked data-deletion checkbox.
  ; Updates and ordinary uninstalls always preserve all learner data.
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current
    RMDir /r "$LOCALAPPDATA\SvaType"
  ${EndIf}
  ; Shared ProgramData and portable folders are deliberately never removed by
  ; a per-user installer. Their owner manages them independently.
!macroend
