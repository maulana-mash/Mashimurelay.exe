# Panduan Verifikasi & Audit Berkala: Fitur Teacher Lab Mode, Restricted Kit & QR Code Offline

Dokumen ini merupakan panduan audit berkala untuk memverifikasi bahwa implementasi **Mode Praktikum Guru (Teacher Lab Mode)**, **Pembatasan Perpustakaan Komponen (Restricted Kit)**, dan **Bagi via QR Code Offline (Teacher-to-Student)** pada ekosistem **Simurelay & Mashimurelay** (Web, Windows Desktop, dan Android Native) telah terpasang dengan benar, stabil, dan memenuhi standar pembelajaran interaktif.

---

## 1. Arsitektur & Spesifikasi Fitur

### A. Fitur 1: Kunci Posisi Komponen (Ctrl + L: Teacher & Canvas Lock)
- **Tujuan**: Memungkinkan guru merancang tata letak komponen di kanvas via Web/Windows, kemudian menguncinya dengan pintasan **`Ctrl + L`** (atau via menu klik kanan / tab Properties). Ketika berkas `.smrl` dibuka di aplikasi Android siswa, siswa **tidak dapat membuka kunci**, **tidak dapat memindahkan posisi**, dan **tidak dapat menghapus/memutar** komponen tersebut.
- **Bebas Konflik Shortcut**:
  - Menggunakan pintasan standar **`Ctrl + L`** tunggal di Web dan Windows Desktop.
  - Menghindari konflik dengan shortcut global sistem operasi Windows (`Ctrl + Shift + L` atau `Win + L`).
  - Mengunci komponen sekaligus mengaktifkan atribut `locked: true` dan `teacherLocked: true`.
- **Tidak Ada Opsi Buka Kunci di Android**:
  - Pada Simurelay Android, **tidak disediakan menu, tombol, atau opsi lock/unlock** untuk siswa.
  - Komponen terkunci yang diterima dari Web/Windows terkunci secara mutlak di Android.
- **Kemandirian Wiring (Wiring Integrity) 100%**:
  - Walaupun posisi komponen terkunci permanen, terminal pin pada komponen **100% tetap aktif**.
  - Siswa tetap dapat mengetuk (tap) pin terminal untuk memulai tarikan kabel dan menyambungkannya ke pin komponen lain.
  - Siswa dapat mengoperasikan saklar (switch) dan tombol tekan (push button) serta menjalankan simulasi secara penuh.
- **Skema Data (`.smrl`)**:
  ```json
  {
    "type": "ElectricalElement",
    "id": 1,
    "svgData": "breaker1p",
    "tag": "-F1",
    "x": 180,
    "y": 120,
    "locked": true,
    "teacherLocked": true
  }
  ```
- **Tampilan Visual**:
  - Pada Web/Windows: Komponen berstatus terkunci ditandai dengan badge gembok hijau zamrud (vektor padlock SVG minimalis) di pojok komponen.
  - Pada Android: Komponen ditandai dengan badge teks industrial CAD tegas (`KIT` atau `LOCK`) di kanvas CAD.

---

### B. Fitur 2: Pembatasan Perpustakaan Komponen (Restricted Assignment Kit)
- **Tujuan**: Guru dapat menentukan sekumpulan komponen tertentu yang diizinkan untuk digunakan oleh murid pada modul tugas praktikum. Murid tidak dibingungkan oleh ratusan komponen lain dan hanya dapat menambahkan komponen yang dibuka oleh guru.
- **Kebersihan Antarmuka Navbar (Clean Navbar)**:
  - Tombol Kit Guru **telah dihapus dari toolbar navbar** agar tampilan navigasi tetap bersih dan tidak menumpuk.
  - Kit Guru kini diakses eksklusif melalui panel drawer navigasi samping (`#drawerItemKit`) dengan ikon vektor crosshair standar Feather/Lucide, serta tombol gear pada banner kit aktif di panel komponen.
- **Antarmuka Pemilih Komponen Guru (Kategori & Ikon Visual)**:
  - **Pembagian per Kategori**: Komponen dikelompokkan secara rapi ke dalam 9 kategori industri (*Sources & Power*, *Circuit Protections*, *Switches & Buttons*, *Contactors & Relays*, *Coils & Timers*, *Auxiliary Contacts*, *Motors & Drives*, *Loads & Signalling*, *Wiring & Junctions*).
  - **Pratinjau Gambar Ikon Komponen**: Setiap kartu menampilkan gambar ikon vektor/SVG asli di sebelah kiri dengan kotak kontras gelap, dan nama simpel library di sebelah kanan (seperti pada panel ensiklopedia).
  - **Kontrol Cepat per Kategori**: Tersedia tombol mini *"Pilih Semua"* dan *"Kosongkan"* di tiap header kategori serta penghitung jumlah komponen terpilih secara real-time.
- **Skema Data (`.smrl`)**:
  ```json
  "assignmentKit": {
    "enabled": true,
    "name": "Praktikum 1 - DOL Starter",
    "allowedComponents": [
      "sourceL1", "sourceL2", "sourceL3", "sourceN", "sourcePE",
      "breaker1p", "breaker3p", "overload3p", "contactor3p",
      "buttonNO", "buttonNC", "coil", "contactNO", "lamp", "motor3ph"
    ]
  }
  ```
- **Preset Bawaan Guru (Web)**:
  1. **DOL Starter**: Sumber daya 3 fasa, MCB 1P & 3P, TOR/Overload, Kontaktor, Push Button NO/NC, Koil, Kontak Bantu, Lampu, Motor 3 Fasa.
  2. **Forward-Reverse**: Komponen DOL Starter ditambah kontak interlock NC & kontaktor arah kedua.
  3. **Star-Delta**: Komponen DOL ditambah Koil On-Delay Timer, Kontak Timer, dan Motor Star-Delta 6-Terminal.
  4. **Timer & Flasher**: Sirkuit berbasis koil on-delay, off-delay, flasher, kontak bantu delay, serta lampu kedip.
  5. **Dasar Sirkuit**: Saklar manual, push button, koil, dan lampu indikator.

---

### C. Fitur 3: Ekspor & Impor Rangkaian via QR Code (100% Offline)
- **Tujuan**: Memungkinkan guru menampilkan QR Code rangkaian tugas di layar monitor / proyektor (via Web atau Windows), kemudian murid memindainya langsung menggunakan Simurelay Android secara **offline** tanpa memerlukan koneksi internet ataupun server cloud.
- **Alur Ekspor Guru (Web / Windows)**:
  1. Guru membuka menu **Save As / Ekspor Sirkuit** (`modalSaveAs`).
  2. Di dalam menu, terdapat opsi ke-4: **📲 Bagi via QR Code (Tugas Guru)**.
  3. Mengklik opsi tersebut membuka dialog modal **📲 Bagi Sirkuit via QR Code (Offline)** yang menampilkan:
     - Judul Rangkaian
     - Status Penguncian Guru (`🔒 X Komponen Terkunci`)
     - Status Kit Komponen (`🎯 Nama Kit` atau `Bebas`)
     - QR Code berresolusi tinggi (dengan zona tenang / quiet zone 4 modul standar industri)
     - Tombol **💾 Unduh Gambar PNG** (untuk disisipkan pada slide materi kuliah / PPT / lembar kerja cetak)
     - Tombol **📋 Salin Teks**
- **Protokol Kompresi Data Offline & Optimalisasi Modul Tebal (Ramah Kamera Buram)**:
  - Format URI: `smrl://v1/<base64url_compressed_payload>`
  - Kompresi: Menggunakan algoritma **Deflate (RFC 1950)** yang didukung native oleh browser (`CompressionStream('deflate')`) dan didekompresi seketika oleh Android menggunakan `java.util.zip.Inflater`.
  - **Penyusutan Data (Payload Minifier)**: Menghapus seluruh properti *default* (seperti `rotation: 0`, `fontSize: 24`, warna standar, dan array tag kosong) serta titik koordinat kabel berlebih sehingga ukuran berkas sangat kecil.
  - **Level Koreksi Error 'L' (Low Parity Overhead)**: Menggunakan Error Correction Level `L` (7%) sehingga matriks QR code turun drastis ke Version 3–5 dengan modul kotak hitam-putih yang berukuran **3× lebih tebal dan besar**, sangat mudah dibaca kamera HP buram, tergores, atau berpencahayaan rendah dari jarak jauh proyektor.
- **Alur & Akses Pindai Murid di Android Native (`simurelay-android`)**:
  - **Navbar Kanvas Bersih**: Sesuai arahan, navbar kanvas utama CAD dibiarkan bersih tanpa tombol scan agar ruang kerja lapang.
  - **Akses Pindai QR Terpusat di Workspace**:
    1. **Halaman Workspace (Utama)**:
       - **Ikon Toolbar Workspace**: Ikon kamera QR (`btnWorkspaceScanQr`) beraksen cyan di toolbar Workspace (posisi ringkas, rapi, dan ergonomis seperti sedia kala). Banner besar kartu scan telah dihapus agar antarmuka daftar proyek tetap clean dan konsisten.
       - **Menu Titik Tiga**: Opsi `Scan QR Tugas Guru` di menu `[ ⋮ MORE ]` Workspace.
    2. **Slide-in Drawer [ ☰ MENU ]**: Opsi `Scan QR Tugas Guru` beraksen cyan di menu samping.
    3. **Project Dashboard Dialog**: Tombol `📲 Scan QR Tugas Guru (Offline)` saat memilih sirkuit.
  - Kamera pemindai terbuka seketika secara **100% Portrait (`PortraitCaptureActivity`)** yang ergonomis untuk pengoperasian satu tangan.
  - Ketika QR Code terdeteksi, muncul dialog konfirmasi modern:
    - 🚀 **Buka Langsung di Proyek Baru**: Membuka lembar kerja baru dengan sirkuit dan batasan kit guru aktif seketika.
    - 💾 **Simpan ke Proyek Saya**: Menyimpan berkas ke direktori Workspace perangkat untuk dikerjakan nanti.
  - Komponen terkunci guru (`teacherLocked: true`) dan pembatasan perpustakaan komponen (`assignmentKit`) tetap aktif secara utuh di Android.

---

## 2. Checklist Pengujian & Audit Berkala

Lakukan pengujian berikut secara berkala setiap kali ada pembaruan versi:

### Checklist 1: Pengujian di Simurelay Web / Windows (Mode Guru)
- [ ] **Navbar Bersih (Clean Toolbar)**:
  - Pastikan tombol `🎯 Kit Guru` **tidak ada di toolbar navbar**.
  - Buka Drawer Menu (`☰`) -> menu `🎯 Kit Praktikum Guru` tersedia dengan ikon crosshair rapi.
- [ ] **Shortcut Kunci Komponen**:
  - Pilih satu atau beberapa komponen di kanvas.
  - Tekan `Ctrl + L` pada keyboard -> komponen berubah status menjadi terkunci dengan badge gembok hijau zamrud.
- [ ] **Pengaturan Kit Praktikum Guru (Kategori & Ikon)**:
  - Buka Drawer -> `🎯 Kit Praktikum Guru`.
  - Dialog modal terbuka dengan lebar lega, komponen tersusun per kategori lengkap dengan pratinjau ikon vektor di kiri dan nama library di kanan.
  - Uji tombol *"Pilih Semua"* dan *"Kosongkan"* pada salah satu kategori -> counter kategori dan counter total terupdate seketika.
  - Klik tombol `💾 Simpan & Terapkan Kit` -> banner `🎯 Kit Praktikum` muncul di panel komponen.
- [ ] **Ekspor QR Code di Menu Save As**:
  - Buka menu **Save As** (Ctrl+S / drawer Save As).
  - Pastikan opsi **📲 Bagi via QR Code (Tugas Guru)** muncul di daftar format ekspor.
  - Klik *"Tampilkan QR"* -> modal QR Code terbuka dengan badge info sirkuit, jumlah komponen terkunci, dan kit aktif.
  - Uji tombol *"Unduh Gambar PNG"* -> file gambar PNG QR tersimpan.

---

### Checklist 2: Pengujian di Simurelay Android (Mode Murid)
- [ ] **Tombol Scan QR di Workspace & Drawer**:
  - Masuk ke Workspace -> ikon kamera QR (`btnWorkspaceScanQr`) tersedia di sebelah tombol import file.
  - Buka Drawer Menu -> opsi **Scan QR Tugas Guru** tersedia dengan warna aksen cyan terang.
- [ ] **Pindai QR Code Offline**:
  - Arahkan kamera ke layar Web/Windows yang menampilkan QR Code tugas.
  - Dialog konfirmasi **📲 Tugas Sirkuit Terdeteksi** muncul otomatis menampilkan judul rangkaian, status kunci guru, dan kit praktikum.
- [ ] **Opsi Buka Langsung vs Simpan ke Proyek Saya**:
  - Uji opsi 1: *"🚀 Buka Langsung di Proyek Baru"* -> sirkuit langsung terbuka di kanvas CAD.
  - Uji opsi 2: *"💾 Simpan ke Proyek Saya"* -> berkas tersimpan di daftar workspace siswa.
- [ ] **Enforcement Kunci Guru di Android**:
  - Coba seret komponen terkunci guru -> komponen **tidak bergerak** dan toast peringatan gembok guru muncul.
  - Tidak ada opsi unlock untuk murid.
  - Coba hubungkan kabel antar pin -> wiring tetap berfungsi 100%.
  - Buka panel library -> hanya komponen yang diizinkan guru yang dapat ditambahkan.

---

## 3. Struktur Berkas yang Terlibat
- `simurelay-web/src_dev/js/canvas.js` (Logika seleksi, lock Ctrl+L, teacherLocked, dan metadata export/import)
- `simurelay-web/src_dev/js/app.js` (Event listener, dialog modal Kit Guru per kategori, QR generator deflate, dan filter sidebar)
- `simurelay-web/src_dev/js/qrcode.js` (Engine QR Code generator berbasis JavaScript)
- `simurelay-web/src_dev/index.html` (Markup UI modal Kit Guru lebar, menu Save As QR, dan modal QR Export)
- `simurelay-android/app/src/main/java/com/simurelay/cad/nativecad/NativeCircuitModel.kt` (Struktur data `teacherLocked` & `AssignmentKit`)
- `simurelay-android/app/src/main/java/com/simurelay/cad/nativecad/WorkspaceManager.kt` (Serialisasi JSON `.smrl`, import, and kit binding)
- `simurelay-android/app/src/main/java/com/simurelay/cad/nativecad/VisualComponentAdapter.kt` (Filter perpustakaan komponen Android)
- `simurelay-android/app/src/main/java/com/simurelay/cad/nativecad/NativeCanvasView.kt` (Enforcement drag-lock & render badge `🎯`)
- `simurelay-android/app/src/main/java/com/simurelay/cad/MainActivity.kt` (ZXing QR scanner offline, dekompresi Inflater, dialog konfirmasi 2-opsi)
- `simurelay-android/app/src/main/res/layout/dialog_modern_qr_import.xml` (Dialog konfirmasi tugas sirkuit QR Android)
- `simurelay-android/app/src/main/res/drawable/ic_qr_scan.xml` (Ikon vektor Material pemindai QR)
