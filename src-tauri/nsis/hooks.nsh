; Mashimurelay Official Windows Firewall Whitelist Hooks (Tauri v2 NSIS)
; Registers inbound TCP ports 8765-8785 strictly for Mashimurelay.exe during installation
; so teachers never face Windows Defender Firewall popups during offline LAN classes.

!macro NSIS_HOOK_POSTINSTALL
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="Mashimurelay Hotspot Kelas"'
  nsExec::ExecToLog 'netsh advfirewall firewall add rule name="Mashimurelay Hotspot Kelas" dir=in action=allow program="$INSTDIR\${MAINBINARYNAME}.exe" protocol=TCP localport=8765-8785 profile=private,public enable=yes description="Mengizinkan koneksi offline LAN Hotspot Kelas Mashimurelay (port 8765-8785)"'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="Mashimurelay Hotspot Kelas"'
!macroend
