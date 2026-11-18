# Mashimurelay — Windows Desktop Executables

Hasil kompilasi native Windows (.exe) menggunakan **Rust Tauri v2**:

1. **`Mashimurelay.exe` (10.75 MB)**
   - Standalone portable application.
   - Tidak memerlukan instalasi. Dapat langsung dijalankan dari folder mana saja atau dari USB flashdisk.
   - Seluruh aset HTML5 Canvas, JS engine, suara, dan simbol komponen terenkapsulasi penuh di dalam binary.

2. **`Mashimurelay_Setup.exe` (3.90 MB)**
   - Windows Setup Installer (NSIS).
   - Menambahkan shortcut aplikasi di Desktop dan Start Menu Windows.
   - Mendaftarkan asosiasi berkas `.smrl` & `.srelay` ke Windows Registry.
   - Mendukung uninstaller bersih melalui Windows Settings / Control Panel.
