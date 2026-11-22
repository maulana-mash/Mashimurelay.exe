# Mashimurelay — Windows Desktop Executables

Hasil kompilasi native Windows (.exe) menggunakan **Rust Tauri v2**:

1. **`Mashimurelay.exe` / `Mashimurelay_Portable.exe`**
   - Standalone portable application (tanpa instalasi, bisa langsung dijalankan dari folder mana saja atau USB flashdisk).
   - Seluruh aset CAD, JS engine, audio, dan 117 simbol IEC 60617 terenkapsulasi penuh di dalam binary.
   - Dilengkapi deteksi status Windows Firewall otomatis + tombol 1-klik **"Pasang Whitelist / Buka Blokir Firewall"** di dalam modal Mode Hotspot Guru.

2. **`Mashimurelay_Setup.exe` / `Mashimurelay_1.0.0_Setup.exe`**
   - Windows Setup Installer (NSIS `perMachine`).
   - Menambahkan shortcut aplikasi di Desktop dan Start Menu Windows.
   - Mendaftarkan asosiasi berkas `.smrl` & `.srelay` ke Windows Registry.
   - **Otomatis mendaftarkan Whitelist Windows Firewall** (`TCP 8765-8785`, profil Private & Public) saat instalasi sehingga Mode Hotspot Guru langsung siap pakai tanpa popup Firewall saat mengajar (dan otomatis dibersihkan saat uninstall).
