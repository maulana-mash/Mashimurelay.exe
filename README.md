<div align="center">
  <img src="assets/logo_mashimurelay.png" alt="Mashimurelay Logo" width="140" />

  <h1>Mashimurelay Windows Desktop</h1>
  <p><strong>High-Performance Native Industrial Electromechanical Relay & Motor Control CAD Simulator</strong></p>

  <p>
    <a href="https://github.com/maulana-mash/Mashimurelay.exe/releases">
      <img src="https://img.shields.io/badge/Download-Windows_x64-0284c7?style=flat-square&logo=windows" alt="Download Windows" />
    </a>
    <img src="https://img.shields.io/badge/Framework-Rust_Tauri_v2-f97316?style=flat-square&logo=rust" alt="Rust Tauri v2" />
    <img src="https://img.shields.io/badge/Standard-IEC_60617-1e293b?style=flat-square" alt="IEC 60617 Standard" />
    <img src="https://img.shields.io/badge/Components-117_Industrial_Symbols-0f766e?style=flat-square" alt="117 Components" />
    <img src="https://img.shields.io/badge/RAM_Footprint-~35_MB-16a34a?style=flat-square" alt="Low RAM Usage" />
    <img src="https://img.shields.io/badge/Hotspot_Server-100%25_Offline_LAN-6366f1?style=flat-square" alt="Offline LAN Hotspot" />
    <img src="https://img.shields.io/badge/Distribution-Portable_%7C_NSIS_Setup-334155?style=flat-square" alt="Distribution Options" />
  </p>
</div>

---

## Ringkasan Proyek (Overview)

**Mashimurelay Windows** (`com.mashimurelay.cad`) adalah aplikasi Computer-Aided Design (CAD) dan simulator interaktif waktu nyata (*real-time*) untuk perancangan skematik rangkaian kontrol relay elektromekanikal serta sistem tenaga motor listrik industri berstandar **IEC 60617**.

Dibangun dengan arsitektur **Hybrid Native Desktop** berbasis **Rust (Tauri v2)** dan akselerasi grafis WebView2, Mashimurelay Windows menggabungkan keandalan komputasi tingkat rendah (*low-level system control*), konsumsi memori yang sangat efisien (~35 MB RAM berbanding 300+ MB pada runtime Electron), serta responsivitas rendering kanvas 60–144 FPS.

Aplikasi ini dirancang sebagai stasiun kerja (*workstation*) instruktur dan teknisi pada pendidikan vokasi (SMK Teknik Ketenagalistrikan / Otomasi Industri), politeknik elektro, dan balai latihan kerja industri. Mashimurelay Windows dilengkapi fitur revolusioner **Local Classroom Hotspot Server** yang memungkinkan guru mendistribusikan soal dan menerima pengumpulan tugas siswa secara nirkabel **tanpa memerlukan koneksi internet publik** sama sekali (100% Offline LAN).

---

## Arsitektur Teknis & Fitur Unggulan Desktop

### 1. Rust Tauri v2 Native Core & Ultra-Low Resource Usage
- **Memory Footprint Ringan**: Menggunakan WebView2 native bawaan sistem operasi Windows, menghasilkan konsumsi memori hanya ~35–50 MB RAM saat simulasi penuh berjalan.
- **Asosiasi Berkas Windows Otomatis**: Berkas proyek bersirkuit CAD standar (`.smrl` dan `.srelay`) langsung terdaftar di sistem Windows Explorer—cukup klik dua kali berkas untuk membukanya langsung di dalam kanvas.
- **Inter-Process Communication (IPC) Cepat**: Komunikasi dua arah antara antarmuka JavaScript dan thread sistem Rust dilakukan melalui mekanisme `invoke` asinkronus berkecepatan tinggi tanpa overhead soket web eksternal.

### 2. Local Classroom Hotspot Server (100% Offline LAN Microserver)
- **Multi-Threaded HTTP Engine Internal**: Diinisialisasi langsung oleh modul Rust (`std::net::TcpListener`) di port `8765`. Guru cukup mengaktifkan fitur Hotspot portabel di laptop Windows tanpa kabel router tambahan.
- **Distribusi Tugas Instan via QR Code**: Menghasilkan kode QR alamat server lokal secara otomatis di layar. Siswa memindai QR menggunakan aplikasi [Mashimurelay Android](https://github.com/maulana-mash/Mashimurelay.apk) untuk mengunduh berkas tugas CAD secara seketika via `GET /task.smrl`.
- **Penilaian & Pratinjau Kanvas Satu Klik**: Berkas sirkuit yang dikumpulkan siswa dapat langsung diuji dan disimulasikan di kanvas guru dengan menekan tombol *Buka di Kanvas*, tanpa perlu proses impor manual.

### 3. End-to-End Hybrid Classroom Mode: Praktikum vs. Ujian
Guru dapat mengatur mode pengumpulan tugas secara dinamis melalui antarmuka *segmented control* khusus:
- **Mode Praktikum (Bebas Revisi)**: Siswa bebas memperbaiki dan mengirim ulang rangkaian sirkuitnya. Setiap pengiriman ulang secara otomatis menimpa berkas lama di folder guru, mencatat penomoran revisi (*badge* `Revisi 2`, `Revisi 3`), dan menyusun daftar siswa terurut berdasarkan kedatangan pertama (`#1 Tercepat`).
- **Mode Ujian (Kunci 1x Kirim)**: Sistem menerapkan proteksi ketat di mana siswa hanya diizinkan mengumpulkan **satu kali**. Upaya pengiriman ulang oleh perangkat siswa yang sama akan langsung ditolak oleh server dengan respons **HTTP 403 Forbidden** (*Locked State*) disertai dialog peringatan gembok terkunci.

### 4. Pemilih Direktori Penyimpanan Tugas Lokal (Native Folder Picker)
- Mengintegrasikan dialog pemilih berkas native Windows (`rfd::FileDialogBuilder`).
- Guru dapat menentukan dan mengganti lokasi folder pengumpulan tugas siswa kapan saja, serta membuka folder arsip di Windows File Explorer dalam satu klik.

### 5. Discrete-Event Equipotential Circuit Solver
- **Multi-Rail Bitmask Propagation**: Melacak polaritas dan fasa daya `L1`, `L2`, `L3`, `N`, `PE`, `DC+`, dan `DC-` secara simultan melalui algoritma penelusuran graf konduktif *Breadth-First Search* (BFS).
- **Series & Parallel Load Resolution**: Menyelesaikan pembagian tegangan pada beban pasif 2-terminal yang disusun secara seri maupun paralel melalui penelusuran *Depth-First Search* (DFS), lengkap dengan pemodelan konduksi dioda semikonduktor (*forward-bias* vs *reverse-bias*).
- **Fixed-Point Equilibrium Convergence**: Mengevaluasi interaksi tertutup antara koil kontaktor, relay tunda waktu (*timer*), kontak bantu (*auxiliary contacts*), serta mekanisme *interlock* mekanis hingga mencapai titik stabil dalam siklus frame deterministik.
- **Fault & Overload Protection**: Mendeteksi hubung singkat (*short circuit*) antar-fasa atau fasa-ke-netral, kelebihan beban DC, karakteristik peleburan sekring (*fuse*), serta simulasi *trip* pada *Thermal Overload Relay*.

### 6. Hardware-Accelerated 2D Canvas & Web Audio API
- **Navigasi Kanvas Halus**: Akselerasi GPU rendering 2D untuk *pan*, *zoom*, dan *snap-to-grid* (grid 30px) pada refresh rate monitor tinggi (60–144 Hz).
- **Sintesis Audio Elektromekanik**: Menghasilkan efek suara fisik nyata (*mechanical clicks*, *contactor latching*, *motor hum*, *breaker trip*, dan *alarm bell*) menggunakan sintesis Web Audio API tanpa dependensi berkas MP3/WAV eksternal.

### 7. Technical PDF Report Generator (Etiket Gambar Teknik Standar)
- Mengekspor gambar skematik vektor definisi tinggi beserta tabel daftar komponen (*Bill of Materials* / BOM) ke dalam dokumen PDF 2 halaman lengkap dengan kepala gambar (*etiket gambar teknik kelistrikan*) standar industri.

---

## Pustaka Komponen Standar IEC 60617 (117 Simbol Industri)

Seluruh simbol komponen dirender dalam format vektor SVG presisi tinggi yang dikelompokkan ke dalam 9 kategori standar:

| Kategori | Cakupan Komponen Utama |
| :--- | :--- |
| **Sources** | Suplai Fasa Tunggal (`L1`), 3-Fasa (`L1, L2, L3`, `3P`, `3P+N`), Netral (`N`), Ground Proteksi (`PE`), Suplai DC (`+`, `-`, Battery), Power Supply AC/DC (1P & 3P dengan input kondisional), Generator 3-Fasa (Manual & Auto Control `C1-C2`). |
| **Protections** | MCB / Circuit Breaker (1P, 2P, 3P, 4P), Thermal-Magnetic Breaker, Motor Protection Circuit Breaker (MPCB), RCCB, RCBO, Fuse (1P, 2P, 3P + N), Fuse Disconnector, Isolator Switch, Thermal Overload Relay (TOR) 1P–3P dengan fitur *Test-Trip*. |
| **Switches** | Saklar Manual (`NO`, `NC`, `CON`), Saklar Selektor 3-Posisi (`I-0-II`, `I-II-III`), Pushbutton (`NO`, `NC`, `CON`, `NONC`, `Crossover`), Emergency Stop (`NO`, `NC`, `CON`, `NONC`), Limit Switch, Roller Switch, Foot Pedal, Key Switch, Pressure, Temperature, Float, Flow, & Proximity Sensor. |
| **Contactors** | Kontaktor Daya Utama `NO` dan `NC` (1P hingga 4P) yang tersinkronisasi otomatis dengan tag koil kendali (`-KM`). |
| **Coils & Timers** | Koil Kontaktor Standar, Latching Relay, Solenoid, On-Delay Timer (`KT`), Off-Delay Timer, On-Off Delay Timer, Flasher/Blinker Relay, dan Phase Failure Relay (`PFR`). |
| **Auxiliaries** | Kontak Bantu Instan (`NO`, `NC`, `CON`, `NONC`), Kontak Bantu Tunda Waktu (*On-Delay*, *Off-Delay*, *On-Off Delay*), serta Kontak Bantu *Thermal Overload* (`95-96` NC & `97-98` NO). |
| **Motors** | Motor Induksi 3-Fasa (`U1, V1, W1`), Motor 3-Fasa 6-Terminal *Star-Delta* (`U1..W2`), Motor Dahlander 2-Kecepatan, Motor 2-Belitan Terpisah, Motor Induksi 1-Fasa, dan Motor DC (dengan deteksi putaran *Clockwise* & *Counter-Clockwise*). |
| **Loads** | Lampu Indikator (5 pilihan warna spektrum), Lampu Kedip (*Blinking Lamp*), Buzzer, Siren, dan Industrial Bell dengan sintesis audio waktu nyata. |
| **Others** | Terminal Block, Junction Node, Cross-Reference Arrows (`arrowSource` & `arrowDestination` dengan penomoran tautan otomatis `-REF1`), Dioda Penyearah, dan Anotasi Teks CAD. |

---

## Ekosistem Tiga Platform Terintegrasi (Multi-Platform Ecosystem)

Format berkas proyek Mashimurelay (`.smrl` / `.srelay`) kompatibel secara penuh antar-platform:

1. **[Mashimurelay Windows Desktop](https://github.com/maulana-mash/Mashimurelay.exe)** (`Rust Tauri v2 Desktop`) — **Repositori ini.**
   - Stasiun kerja instruktur guru & teknisi dengan microserver Hotspot LAN offline, pengelolaan ujian kelas, dan asosiasi berkas desktop.
2. **[Mashimurelay Web](https://github.com/maulana-mash/Mashimurelay-Web)** (`PWA & Online Simulator`)
   - Berjalan langsung pada peramban web modern tanpa instalasi dengan dukungan *offline-first* Service Worker.
   - **Akses Langsung**: [https://mashimurelay.vercel.app/](https://mashimurelay.vercel.app/)
3. **[Mashimurelay Android](https://github.com/maulana-mash/Mashimurelay.apk)** (`100% Pure Kotlin Native`)
   - Aplikasi mobile native murni (`com.mashimurelay.app`) dengan akselerasi GPU Canvas untuk perangkat siswa di ruang kelas.

---

## Unduhan & Opsi Distribusi Windows

Anda dapat mengunduh biner rilis resmi melalui halaman **[Releases](https://github.com/maulana-mash/Mashimurelay.exe/releases)**:

| Berkas Biner | Ukuran | Deskripsi & Kegunaan |
| :--- | :---: | :--- |
| **`Mashimurelay.exe`** | ~17.9 MB | **Standalone Portable Executable**<br>Dapat langsung dijalankan dari folder mana saja atau melalui USB flashdisk tanpa hak administrator (*no installation required*). |
| **`Mashimurelay_Setup.exe`** | ~10.7 MB | **Standard Windows Installer (NSIS)**<br>Membuat pintasan (*shortcut*) di Start Menu dan Desktop Windows, mendaftarkan asosiasi berkas `.smrl` ke Windows Registry, serta menyediakan uninstaller bersih di Control Panel. |

---

## Struktur Direktori Repositori

```text
mashimurelay-windows/
├── assets/                          # Brand assets & vector logo untuk dokumentasi
│   └── logo_mashimurelay.png
├── dist/                            # Bundel web minified yang di-embed ke binary Rust
│   ├── index.html                   # Halaman CAD utama (komentar di-strip, secure bundle)
│   ├── components.json              # Topologi pin & geometri 117 simbol IEC 60617
│   ├── css/style.css                # Dark engineering styling theme
│   └── js/                          # Obfuscated & minified logic bundle (engine, canvas, app)
├── src-tauri/                       # Rust Native Desktop Backend (Tauri v2)
│   ├── Cargo.toml                   # Konfigurasi dependensi crate Rust
│   ├── tauri.conf.json              # Konfigurasi window, permission, & NSIS bundle
│   ├── build.rs                     # Build script tauri-build
│   ├── capabilities/                # Pengaturan capability security Tauri
│   ├── icons/                       # Ikon aplikasi (.ico, .png berbagai resolusi)
│   └── src/
│       ├── main.rs                  # Entrypoint biner desktop
│       └── lib.rs                   # Server Hotspot HTTP LAN, state manager, & native commands
├── Release_Output/                  # Output kompilasi executable lokal
├── package.json                     # Konfigurasi script Tauri CLI
├── .gitignore                       # Aturan pengabaian cache build & binary besar
└── README.md                        # Dokumentasi teknis repositori
```

---

## Panduan Kompilasi Mandiri (Building from Source)

Jika Anda ingin mengompilasi biner executable Windows sendiri dari kode sumber:

### Kebutuhan Lingkungan Pengembangan (Prerequisites)
1. **Sistem Operasi**: Windows 10 atau Windows 11 (64-bit).
2. **Rust Toolchain**: Versi 1.80 atau lebih baru ([rustup.rs](https://rustup.rs/)).
3. **Node.js**: Versi 18 LTS atau lebih baru ([nodejs.org](https://nodejs.org/)).
4. **C++ Build Tools**: Visual Studio C++ Build Tools (MSVC).

### 1. Kloning Repositori
```bash
git clone https://github.com/maulana-mash/Mashimurelay.exe.git
cd Mashimurelay.exe
```

### 2. Instalasi Dependensi Node
```bash
npm install
```

### 3. Menjalankan dalam Mode Pengembangan (Dev Mode)
```bash
npm run dev
# atau: npx @tauri-apps/cli dev
```

### 4. Mengompilasi Biner Rilis Produksi (Production Build)
```bash
npm run build
# atau: npx @tauri-apps/cli build
```

Biner keluaran akan berada pada lokasi berikut:
- **Executable Mandiri**: `src-tauri/target/release/mashimurelay.exe`
- **Installer NSIS**: `src-tauri/target/release/bundle/nsis/Mashimurelay_1.0.0_x64-setup.exe`

---

## Lisensi & Hak Cipta

Hak Cipta © 2026 **Mashimurelay Engineering Project (Mash Maulana)**. Seluruh hak cipta dilindungi undang-undang.  
Dirancang dan direkayasa untuk pendidikan vokasi ketenagalistrikan, laboratorium teknik elektro, serta pelatihan otomasi industri.

> **Keterangan Standar & Orisinalitas Aset:** Seluruh 117 simbol skematik kelistrikan mengacu pada standar internasional publik **IEC 60617 / DIN EN 60617** dan digenerasi mandiri sebagai vektor SVG *clean-room*. Mesin simulasi kelistrikan, deteksi gangguan per-jalur, *Teacher Lab Mode*, dan paket desktop Rust Tauri v2 merupakan implementasi rekayasa orisinal Mashimurelay.
