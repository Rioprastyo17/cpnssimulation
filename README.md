# ==> Simulasi CAT CPNS (Rust + eframe/egui)

Aplikasi simulasi Computer Assisted Test (CAT) Seleksi Kompetensi Dasar (SKD) Calon Pegawai Negeri Sipil (CPNS) berbasis desktop yang dibangun menggunakan bahasa pemrograman **Rust** dan framework eframe/egui.

Aplikasi ini didesain ringan, cepat, tanpa lag, dan dapat berjalan secara offline (tanpa koneksi internet) pada sistem operasi **Windows**, **macOS**, dan **Linux**.

## ==> Demo Video

[![Lihat Demo](https://img.shields.io/badge/▶️-Lihat_Demo_Video-red)](https://github.com/Rioprastyo17/cpnssimulation/releases/download/v1.0.0/record.mp4)

## ==> Fitur Utama

- **Format Soal Resmi SKD CPNS:**
  - **TWK (Tes Wawasan Kebangsaan):** 30 Soal (Bobot: Benar = 5, Salah/Kosong = 0)
  - **TIU (Tes Inteligensia Umum):** 35 Soal (Bobot: Benar = 5, Salah/Kosong = 0)
  - **TKP (Tes Karakteristik Pribadi):** 45 Soal (Bobot: Skala nilai 1 – 5, Kosong = 0)
  - Total: **110 Soal** dengan batas waktu **100 Menit**.
- **Multi-Dataset & Bank Soal:**
  - Mendeteksi otomatis semua file bank soal (`dataset_soal*.json`) di direktori aplikasi.
  - Memilih paket soal latihan tertentu atau opsi **"Semua Dataset (Gabungan)"** untuk mengacak kumpulan bank soal yang lebih besar.
- **Dukungan Soal Gambar & Figural:**
  - Rendering gambar otomatis pada soal penalaran figural atau diagram geometri/matematika.
- **Navigasi & Lembar Ujian:**
  - Grid navigasi 110 nomor soal dengan indikator warna:
    - 🟩 Hijau: Sudah dijawab
    - 🟨 Kuning: Ditandai ragu-ragu
    - ⬜ Abu-abu: Belum dijawab
  - Tombol pintas: *Sebelumnya*, *Berikutnya*, dan checkbox *Ragu-ragu*.
- **Timer Real-time:**
  - Penghitung mundur waktu ujian interaktif. Ujian otomatis selesai jika waktu habis.
- **Hasil & Evaluasi Lengkap:**
  - Rekap perolehan skor per kategori (TWK, TIU, TKP) dan total nilai akhir.
  - Status kelulusan berdasarkan **Passing Grade** resmi.
  - Mode **Review & Pembahasan**: Tinjau kembali setiap nomor soal lengkap dengan kunci jawaban, pilihan peserta, dan penjelasan/pembahasan bergambar.

---

## ==> Prasyarat Sistem

Sebelum melakukan kompilasi atau menjalankan aplikasi dari sumber (source code), pastikan Anda telah memasang **Rust & Cargo** (disarankan versi stable terbaru).

### Pasang Rust
Kunjungi [rustup.rs](https://rustup.rs/) atau jalankan perintah berikut di terminal:

- **Linux / macOS:**
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source $HOME/.cargo/env
  ```
- **Windows:**
  Unduh dan jalankan installer resmi [rustup-init.exe](https://win.rustup.rs/).

---

## ==> Panduan Instalasi & Menjalankan

### 1. Linux (Ubuntu / Debian / Fedora / Arch)

#### Dependensi Sistem (GUI / eframe)
Aplikasi membutuhkan pustaka grafis X11 / Wayland dan fontconfig:

- **Ubuntu / Debian / Linux Mint:**
  ```bash
  sudo apt update
  sudo apt install -y build-essential pkg-config libssl-dev \
      libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
      libxkbcommon-dev libfontconfig1-dev
  ```

- **Fedora / RHEL:**
  ```bash
  sudo dnf install -y gcc gcc-c++ pkgconfig openssl-devel \
      libxcb-devel libxkbcommon-devel fontconfig-devel
  ```

- **Arch Linux / Manjaro:**
  ```bash
  sudo pacman -S --needed base-devel openssl libxkbcommon fontconfig
  ```

#### Menjalankan & Build di Linux
```bash
# Clone repository (jika belum)
git clone https://github.com/Rioprastyo17/cpnssimulation.git
cd cpnssimulation

# Jalankan dalam mode release (performa optimal)
cargo run --release

# Atau build binary mandiri
cargo build --release
# File binary akan berada di: target/release/simulasi-cpns
```

---

### 2. Microsoft Windows (Windows 10 / 11)

#### Dependensi Sistem
- Pastikan Anda telah menginstal **Visual Studio C++ Build Tools** (disarankan saat instalasi `rustup`). Jika belum, instal melalui Visual Studio Installer dengan mencentang *Desktop development with C++*.

#### Menjalankan & Build di Windows
Buka **PowerShell** atau **Command Prompt**:

```powershell
# Clone repository
git clone https://github.com/Rioprastyo17/cpnssimulation.git
cd cpnssimulation

# Jalankan aplikasi
cargo run --release

# Atau build file executable (.exe)
cargo build --release
# File binary akan berada di: target\release\simulasi-cpns.exe
```

> [!IMPORTANT]
> **Distribusi Mandiri di Windows:**
> Jika memindahkan `simulasi-cpns.exe` ke folder lain atau komputer lain, pastikan menyertakan file dataset JSON (`dataset_soal*.json`) dan folder `assets/` di direktori yang sama dengan file `.exe` agar aplikasi dapat mencari dan memuat bank soal serta gambar dengan benar.

---

### 3. Apple macOS (Intel & Apple Silicon M1/M2/M3)

#### Dependensi Sistem
Pastikan Xcode Command Line Tools sudah terpasang:
```bash
xcode-select --install
```

#### Menjalankan & Build di macOS
Buka **Terminal**:

```bash
# Clone repository
git clone https://github.com/Rioprastyo17/cpnssimulation.git
cd cpnssimulation

# Jalankan aplikasi
cargo run --release

# Atau build binary
cargo build --release
# File binary akan berada di: target/release/simulasi-cpns
```

---

## ==> Struktur Direktori Proyek

```text
simulasi-cpns/
├── Cargo.toml                     # Konfigurasi dependensi Rust
├── src/
│   └── main.rs                    # Logika utama aplikasi simulasi CPNS (egui)
├── assets/
│   └── images/                    # Gambar diagram/soal & pembahasan (figural)
├── dataset_soal.json              # Bank soal utama
├── dataset_soal1.json              # Bank soal paket 1
├── dataset_soal11.json             # Bank soal paket 11
├── dataset_soal22.json             # Bank soal paket 22
├── dataset_soal33.json             # Bank soal paket 33
├── dataset_soal_latihan*.json     # Koleksi paket latihan tambahan
├── record.mp4                     # Video demo aplikasi
└── README.md                      # Dokumentasi proyek
```

---

## ==> Menambahkan Dataset Soal Baru

Aplikasi secara otomatis mendeteksi file `.json` yang mengikuti format bank soal CPNS di folder aplikasi. Format JSON mengikuti struktur berikut:

```json
[
  {
    "id": 1,
    "kategori": "TWK",
    "pertanyaan": "Isi teks pertanyaan...",
    "gambar": null,
    "pilihan": [
      "A. Pilihan satu",
      "B. Pilihan dua",
      "C. Pilihan tiga",
      "D. Pilihan empat",
      "E. Pilihan lima"
    ],
    "kunci": "A",
    "pembahasan": "Penjelasan detail kunci jawaban...",
    "gambar_pembahasan": null,
    "skor_tkp": null
  },
  {
    "id": 66,
    "kategori": "TKP",
    "pertanyaan": "Isi studi kasus TKP...",
    "gambar": null,
    "pilihan": [
      "A. Respons A",
      "B. Respons B",
      "C. Respons C",
      "D. Respons D",
      "E. Respons E"
    ],
    "kunci": "C",
    "pembahasan": "Analisis nilai integritas...",
    "gambar_pembahasan": null,
    "skor_tkp": {
      "A": 3,
      "B": 4,
      "C": 5,
      "D": 2,
      "E": 1
    }
  }
]
```

Simpan file dengan awalan `dataset_soal` (misal: `dataset_soal_custom.json`), lalu buka kembali aplikasi atau klik pilih dataset pada menu utama.

---

## ==> Lisensi

Proyek ini didistribusikan untuk keperluan belajar dan persiapan ujian seleksi CASN / CPNS. Bebas digunakan dan dikembangkan lebih lanjut.
