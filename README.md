# Simulasi CAT CPNS (Rust + eframe/egui)

Aplikasi desktop simulasi ujian CAT (Computer Assisted Test) CPNS (Calon Pegawai Negeri Sipil) berbasis Rust menggunakan antarmuka grafis `eframe`/`egui`.

## Fitur Utama

- **Kategori Soal Lengkap:**
  - **TWK** (Tes Wawasan Kebangsaan)
  - **TIU** (Tes Inteligensia Umum)
  - **TKP** (Tes Karakteristik Pribadi) dengan pembobotan skor 1–5
- **Tampilan GUI Modern & Ringan:** Dibuat menggunakan Rust dan `egui` yang responsif serta hemat resource.
- **Timer & Navigasi Soal:** Dilengkapi penghitung waktu ujian dan panel navigasi soal dengan penanda status (ragu-ragu, terjawab, belum dijawab).
- **Hasil & Evaluasi:** Perhitungan skor otomatis dengan passing grade standar CPNS dan ringkasan performa.
- **Dukungan Dataset Kustom:** Memuat soal dari file `dataset_soal.json` secara fleksibel.

## Prasyarat

Pastikan telah menginstal Rust dan Cargo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Untuk Linux (Ubuntu/Debian), dependensi sistem yang dibutuhkan untuk `egui`:
```bash
sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

## Menjalankan Aplikasi

```bash
cargo run --release
```

## Struktur Proyek

- `src/main.rs`: Logika aplikasi, state management, dan antarmuka egui.
- `dataset_soal.json`: Kumpulan data bank soal TWK, TIU, dan TKP.
- `Cargo.toml`: Konfigurasi dependensi project.
