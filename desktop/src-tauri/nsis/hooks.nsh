!macro NSIS_HOOK_PREUNINSTALL
  ; Default button is "No" (MB_DEFBUTTON2) — losing the entire feed/post/
  ; folder database is unrecoverable, so an accidental Enter-press during
  ; uninstall must not wipe it.
  MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2 \
    "Also delete all Wyrm data (feeds, folders, downloaded articles)?$\r$\n$\r$\nThis cannot be undone." \
    IDYES wyrm_delete_data IDNO wyrm_keep_data
  wyrm_delete_data:
    RMDir /r "$APPDATA\rss.wyrm.desktop"
  wyrm_keep_data:
!macroend
