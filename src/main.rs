use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align, Color32, Frame, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, Ui, Vec2,
};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

// ============================================================================
// 1. STRUKTUR DATA (DATASET)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soal {
    pub id: u32,
    pub kategori: String, // "TWK", "TIU", atau "TKP"
    pub pertanyaan: String,
    pub opsi: Vec<String>,
    #[serde(default)]
    pub kunci_jawaban: Option<String>,
    #[serde(default)]
    pub bobot_tkp: Option<Vec<u8>>,
    #[serde(default)]
    pub pembahasan: Option<String>,
    #[serde(default)]
    pub gambar_soal: Option<String>,      // path relatif ke file gambar soal
    #[serde(default)]
    pub gambar_pembahasan: Option<String>, // path relatif ke file gambar pembahasan
}

impl Soal {
    pub fn is_twk(&self) -> bool { self.kategori.eq_ignore_ascii_case("twk") }
    pub fn is_tiu(&self) -> bool { self.kategori.eq_ignore_ascii_case("tiu") }
    pub fn is_tkp(&self) -> bool { self.kategori.eq_ignore_ascii_case("tkp") }
}

#[derive(Default, Clone)]
pub struct MasterPools {
    pub pool_twk: Vec<Rc<Soal>>,
    pub pool_tiu: Vec<Rc<Soal>>,
    pub pool_tkp: Vec<Rc<Soal>>,
}

impl MasterPools {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let file = File::open(path.as_ref())
            .map_err(|e| format!("Gagal membuka file {:?}: {}", path.as_ref(), e))?;
        let reader = BufReader::new(file);
        let list: Vec<Soal> = serde_json::from_reader(reader)
            .map_err(|e| format!("Gagal membaca/mem-parsing format JSON: {}", e))?;

        let mut pools = Self::default();
        for s in list {
            let rc = Rc::new(s);
            if rc.is_twk() { pools.pool_twk.push(rc); }
            else if rc.is_tiu() { pools.pool_tiu.push(rc); }
            else if rc.is_tkp() { pools.pool_tkp.push(rc); }
        }
        Ok(pools)
    }

    pub fn total_soal(&self) -> usize {
        self.pool_twk.len() + self.pool_tiu.len() + self.pool_tkp.len()
    }

    pub fn fallback_sample() -> Self {
        let mut pools = Self::default();

        for i in 1..=40 {
            pools.pool_twk.push(Rc::new(Soal {
                id: i,
                kategori: "TWK".to_string(),
                pertanyaan: format!(
                    "Soal TWK #{}: Nilai-nilai dasar Pancasila yang diwujudkan dalam kehidupan berbangsa dan bernegara adalah bentuk integritas dan komitmen kebangsaan. Hal ini mencerminkan pengamalan sila ke...",
                    i
                ),
                opsi: vec![
                    "Sila Pertama: Ketuhanan Yang Maha Esa".into(),
                    "Sila Kedua: Kemanusiaan yang Adil dan Beradab".into(),
                    "Sila Ketiga: Persatuan Indonesia".into(),
                    "Sila Keempat: Kerakyatan yang Dipimpin oleh Hikmat Kebijaksanaan".into(),
                    "Sila Kelima: Keadilan Sosial bagi Seluruh Rakyat Indonesia".into(),
                ],
                kunci_jawaban: Some("C".into()),
                bobot_tkp: None,
                pembahasan: Some("Komitmen kebangsaan, cinta tanah air, dan persatuan dalam keberagaman merupakan inti pengamalan Sila Ketiga: Persatuan Indonesia.".into()),
                gambar_soal: None,
                gambar_pembahasan: None,
            }));
        }

        for i in 41..=85 {
            pools.pool_tiu.push(Rc::new(Soal {
                id: i,
                kategori: "TIU".to_string(),
                pertanyaan: format!(
                    "Soal TIU #{}: Jika x = 24% dari 150 dan y = 150% dari 0,24, maka hubungan yang tepat antara nilai x dan y adalah...",
                    i
                ),
                opsi: vec![
                    "x = y".into(),
                    "x > y".into(),
                    "x < y".into(),
                    "x = 2y".into(),
                    "Hubungan x dan y tidak dapat ditentukan".into(),
                ],
                kunci_jawaban: Some("B".into()),
                bobot_tkp: None,
                pembahasan: Some("Hitung x: 0,24 * 150 = 36. Hitung y: 1,5 * 0,24 = 0,36. Maka jelas bahwa x > y (36 > 0,36).".into()),
                gambar_soal: None,
                gambar_pembahasan: None,
            }));
        }

        for i in 86..=140 {
            pools.pool_tkp.push(Rc::new(Soal {
                id: i,
                kategori: "TKP".to_string(),
                pertanyaan: format!(
                    "Soal TKP #{}: Saat sedang menyelesaikan laporan penting dengan tenggat waktu hari ini, seorang rekan kerja datang meminta bantuan untuk masalah sistem yang mendesak. Sikap Anda adalah...",
                    i
                ),
                opsi: vec![
                    "Menolaknya secara tegas karena pekerjaan saya sendiri sedang mendesak".into(),
                    "Membantunya sebentar lalu kembali melanjutkan pekerjaan saya".into(),
                    "Mendengarkan inti masalahnya, memberi petunjuk ringkas, dan melanjutkan tugas utama".into(),
                    "Meminta rekan lain yang sedang luang untuk membantunya agar tugas saya selesai tepat waktu".into(),
                    "Fokus penuh menyelesaikan tugas utama dahulu, lalu membantunya jika waktu masih memungkinkan".into(),
                ],
                kunci_jawaban: None,
                bobot_tkp: Some(vec![1, 3, 4, 5, 2]),
                pembahasan: Some("Aspek Manajemen Waktu dan Kolaborasi: Mendelegasikan bantuan kepada rekan lain yang luang (opsi D bernilai 5) menyelesaikan kendala rekan kerja sekaligus memastikan tugas utama selesai tepat waktu.".into()),
                gambar_soal: None,
                gambar_pembahasan: None,
            }));
        }

        pools
    }
}

// ============================================================================
// 2. LOGIKA PENILAIAN & KONSTANTA STANDAR BKN
// ============================================================================

pub const PG_TWK: u32 = 65;
pub const PG_TIU: u32 = 80;
pub const PG_TKP: u32 = 166;
pub const TOTAL_DURASI_DETIK: u64 = 100 * 60; // 100 Menit

pub const TARGET_TWK: usize = 30;
pub const TARGET_TIU: usize = 35;
pub const TARGET_TKP: usize = 45;
pub const TARGET_TOTAL: usize = TARGET_TWK + TARGET_TIU + TARGET_TKP;

#[derive(Debug, Clone)]
pub struct HasilUjian {
    pub skor_twk: u32,
    pub skor_tiu: u32,
    pub skor_tkp: u32,
    pub total_skor: u32,
    pub lulus_twk: bool,
    pub lulus_tiu: bool,
    pub lulus_tkp: bool,
    pub lulus_keseluruhan: bool,
    pub jumlah_terjawab: usize,
    pub total_soal: usize,
    pub waktu_pengerjaan: Duration,
    pub mode_tanpa_waktu: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum StatusJawabanReview {
    Benar,
    Salah,
    Kosong,
    TkpMaksimal,
    TkpSebagian,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FilterPembahasan {
    Semua,
    SalahAtauKurang,
    Twk,
    Tiu,
    Tkp,
}

// ============================================================================
// 3. STATE MANAGEMENT APLIKASI
// ============================================================================

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Layar {
    Menu,
    Ujian,
    Hasil,
    Pembahasan,
}

pub struct SimulasiCpnsApp {
    pools: MasterPools,
    info_dataset: String,
    layar: Layar,

    // Dataset selector state
    daftar_dataset: Vec<DatasetInfo>,  // daftar file dataset yang ditemukan
    dataset_terpilih_idx: usize,       // index yang sedang dipilih di dropdown

    // Exam runtime state
    soal_aktif: Vec<Rc<Soal>>,
    jawaban: HashMap<u32, usize>,
    ragu_ragu: HashMap<u32, bool>,
    index_soal: usize,
    waktu_mulai: Option<Instant>,
    durasi_ujian: Duration,
    dialog_konfirmasi: bool,
    mode_tanpa_waktu: bool, // NEW: flag mode tanpa batas waktu

    // Result state
    hasil: Option<HasilUjian>,

    // Review / Pembahasan state
    index_pembahasan: usize,
    filter_pembahasan: FilterPembahasan,

    // Cache texture gambar: path → TextureHandle
    texture_cache: HashMap<String, egui::TextureHandle>,
}

/// Metadata sebuah file dataset soal
#[derive(Debug, Clone)]
pub struct DatasetInfo {
    pub path: PathBuf,
    pub label: String,       // nama tampil di UI
    pub twk: usize,
    pub tiu: usize,
    pub tkp: usize,
    pub total: usize,
    pub error: Option<String>,
}

impl DatasetInfo {
    pub fn scan_directory(dir: &Path) -> Vec<Self> {
        let mut results = Vec::new();

        // Tambahkan "Gunakan Semua Dataset" sebagai opsi pertama
        let all_info = DatasetInfo {
            path: PathBuf::new(),
            label: "🗂  Semua Dataset (Gabungan)".to_string(),
            twk: 0,
            tiu: 0,
            tkp: 0,
            total: 0,
            error: None,
        };
        results.push(all_info);

        // Scan file JSON di direktori
        if let Ok(entries) = fs::read_dir(dir) {
            let mut json_paths: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
                .collect();
            json_paths.sort();

            for path in json_paths {
                let label = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string();

                let info = match MasterPools::from_file(&path) {
                    Ok(p) => DatasetInfo {
                        label: format!("📄  {}", label),
                        twk: p.pool_twk.len(),
                        tiu: p.pool_tiu.len(),
                        tkp: p.pool_tkp.len(),
                        total: p.total_soal(),
                        error: None,
                        path,
                    },
                    Err(e) => DatasetInfo {
                        label: format!("⚠  {}", label),
                        twk: 0,
                        tiu: 0,
                        tkp: 0,
                        total: 0,
                        error: Some(e),
                        path,
                    },
                };
                results.push(info);
            }
        }

        // Hitung total gabungan untuk opsi "Semua Dataset"
        let total_twk: usize = results.iter().skip(1).map(|d| d.twk).sum();
        let total_tiu: usize = results.iter().skip(1).map(|d| d.tiu).sum();
        let total_tkp: usize = results.iter().skip(1).map(|d| d.tkp).sum();
        if let Some(all) = results.first_mut() {
            all.twk = total_twk;
            all.tiu = total_tiu;
            all.tkp = total_tkp;
            all.total = total_twk + total_tiu + total_tkp;
        }

        results
    }
}

impl SimulasiCpnsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        // Coba direktori: current_dir, lalu direktori executable
        let scan_dir = if PathBuf::from("dataset_soal.json").exists() {
            PathBuf::from(".")
        } else {
            exe_dir
        };

        let daftar_dataset = DatasetInfo::scan_directory(&scan_dir);

        // Muat dataset default: index 1 = file pertama yang valid, atau 0 = semua gabungan
        let default_idx = if daftar_dataset.len() > 1 { 1 } else { 0 };
        let (pools, info) = Self::load_dataset_at(&daftar_dataset, default_idx);

        Self {
            pools,
            info_dataset: info,
            layar: Layar::Menu,
            daftar_dataset,
            dataset_terpilih_idx: default_idx,
            soal_aktif: Vec::new(),
            jawaban: HashMap::new(),
            ragu_ragu: HashMap::new(),
            index_soal: 0,
            waktu_mulai: None,
            durasi_ujian: Duration::from_secs(TOTAL_DURASI_DETIK),
            dialog_konfirmasi: false,
            mode_tanpa_waktu: false,
            hasil: None,
            index_pembahasan: 0,
            filter_pembahasan: FilterPembahasan::Semua,
            texture_cache: HashMap::new(),
        }
    }

    /// Load pools dari pilihan dataset berdasarkan index di daftar_dataset.
    /// Index 0 = gabungan semua dataset valid.
    fn load_dataset_at(daftar: &[DatasetInfo], idx: usize) -> (MasterPools, String) {
        let Some(info) = daftar.get(idx) else {
            let p = MasterPools::fallback_sample();
            return (p.clone(), "⚠ Tidak ada dataset ditemukan. Menggunakan data sampel.".to_string());
        };

        // Opsi "Semua Dataset Gabungan" (path kosong)
        if info.path == PathBuf::new() {
            let mut merged = MasterPools::default();
            for d in daftar.iter().skip(1) {
                if d.error.is_some() { continue; }
                if let Ok(p) = MasterPools::from_file(&d.path) {
                    merged.pool_twk.extend(p.pool_twk);
                    merged.pool_tiu.extend(p.pool_tiu);
                    merged.pool_tkp.extend(p.pool_tkp);
                }
            }
            if merged.total_soal() == 0 {
                let p = MasterPools::fallback_sample();
                return (p, "⚠ Semua dataset gagal dimuat. Menggunakan data sampel.".to_string());
            }
            let msg = format!(
                "✓ Semua Dataset Gabungan: TWK={} | TIU={} | TKP={} (Total={})",
                merged.pool_twk.len(), merged.pool_tiu.len(), merged.pool_tkp.len(), merged.total_soal()
            );
            return (merged, msg);
        }

        match MasterPools::from_file(&info.path) {
            Ok(p) => {
                let msg = format!(
                    "✓ Berhasil memuat '{}': TWK={} | TIU={} | TKP={} (Total={})",
                    info.path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
                    p.pool_twk.len(), p.pool_tiu.len(), p.pool_tkp.len(), p.total_soal()
                );
                (p, msg)
            }
            Err(e) => {
                let p = MasterPools::fallback_sample();
                let msg = format!(
                    "⚠ Gagal memuat '{}': {}. Menggunakan data sampel.",
                    info.path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
                    e
                );
                (p, msg)
            }
        }
    }

    /// Reload pools dari dataset yang saat ini dipilih
    pub fn reload_dataset(&mut self) {
        let (pools, info) = Self::load_dataset_at(&self.daftar_dataset, self.dataset_terpilih_idx);
        self.pools = pools;
        self.info_dataset = info;
    }

    pub fn mulai_tryout(&mut self, tanpa_waktu: bool) {
        let mut rng = rand::thread_rng();

        let ambil_twk: Vec<Rc<Soal>> = if self.pools.pool_twk.len() >= TARGET_TWK {
            self.pools.pool_twk.choose_multiple(&mut rng, TARGET_TWK).cloned().collect()
        } else {
            self.pools.pool_twk.clone()
        };

        let ambil_tiu: Vec<Rc<Soal>> = if self.pools.pool_tiu.len() >= TARGET_TIU {
            self.pools.pool_tiu.choose_multiple(&mut rng, TARGET_TIU).cloned().collect()
        } else {
            self.pools.pool_tiu.clone()
        };

        let ambil_tkp: Vec<Rc<Soal>> = if self.pools.pool_tkp.len() >= TARGET_TKP {
            self.pools.pool_tkp.choose_multiple(&mut rng, TARGET_TKP).cloned().collect()
        } else {
            self.pools.pool_tkp.clone()
        };

        let mut soal_gabungan = Vec::with_capacity(ambil_twk.len() + ambil_tiu.len() + ambil_tkp.len());
        soal_gabungan.extend(ambil_twk);
        soal_gabungan.extend(ambil_tiu);
        soal_gabungan.extend(ambil_tkp);
        soal_gabungan.shuffle(&mut rng);

        self.soal_aktif = soal_gabungan;
        self.jawaban.clear();
        self.ragu_ragu.clear();
        self.index_soal = 0;
        self.waktu_mulai = Some(Instant::now());
        self.dialog_konfirmasi = false;
        self.hasil = None;
        self.index_pembahasan = 0;
        self.filter_pembahasan = FilterPembahasan::Semua;
        self.mode_tanpa_waktu = tanpa_waktu;
        self.layar = Layar::Ujian;
    }

    pub fn selesaikan_ujian(&mut self) {
        let waktu_berjalan = match self.waktu_mulai {
            Some(t) => t.elapsed(),
            None => Duration::ZERO,
        };

        let mut skor_twk = 0u32;
        let mut skor_tiu = 0u32;
        let mut skor_tkp = 0u32;
        let mut terjawab = 0usize;
        let opsi_huruf = ["A", "B", "C", "D", "E"];

        for soal in &self.soal_aktif {
            if let Some(&pilihan_idx) = self.jawaban.get(&soal.id) {
                terjawab += 1;
                let huruf_dipilih = opsi_huruf.get(pilihan_idx).copied().unwrap_or("");

                if soal.is_twk() {
                    if let Some(ref kunci) = soal.kunci_jawaban {
                        if kunci.eq_ignore_ascii_case(huruf_dipilih) { skor_twk += 5; }
                    }
                } else if soal.is_tiu() {
                    if let Some(ref kunci) = soal.kunci_jawaban {
                        if kunci.eq_ignore_ascii_case(huruf_dipilih) { skor_tiu += 5; }
                    }
                } else if soal.is_tkp() {
                    if let Some(ref bobot) = soal.bobot_tkp {
                        if let Some(&poin) = bobot.get(pilihan_idx) {
                            skor_tkp += poin as u32;
                        }
                    }
                }
            }
        }

        let total_skor = skor_twk + skor_tiu + skor_tkp;
        let lulus_twk = skor_twk >= PG_TWK;
        let lulus_tiu = skor_tiu >= PG_TIU;
        let lulus_tkp = skor_tkp >= PG_TKP;
        let lulus_keseluruhan = lulus_twk && lulus_tiu && lulus_tkp;

        self.hasil = Some(HasilUjian {
            skor_twk,
            skor_tiu,
            skor_tkp,
            total_skor,
            lulus_twk,
            lulus_tiu,
            lulus_tkp,
            lulus_keseluruhan,
            jumlah_terjawab: terjawab,
            total_soal: self.soal_aktif.len(),
            waktu_pengerjaan: waktu_berjalan,
            mode_tanpa_waktu: self.mode_tanpa_waktu,
        });

        self.dialog_konfirmasi = false;
        self.index_pembahasan = 0;
        self.layar = Layar::Hasil;
    }

    pub fn cek_status_review(&self, soal: &Soal) -> StatusJawabanReview {
        let opsi_huruf = ["A", "B", "C", "D", "E"];
        if let Some(&pilihan_idx) = self.jawaban.get(&soal.id) {
            if soal.is_twk() || soal.is_tiu() {
                let huruf = opsi_huruf.get(pilihan_idx).copied().unwrap_or("");
                if let Some(ref kunci) = soal.kunci_jawaban {
                    if kunci.eq_ignore_ascii_case(huruf) {
                        return StatusJawabanReview::Benar;
                    }
                }
                StatusJawabanReview::Salah
            } else if soal.is_tkp() {
                if let Some(ref bobot) = soal.bobot_tkp {
                    if let Some(&poin) = bobot.get(pilihan_idx) {
                        return if poin == 5 {
                            StatusJawabanReview::TkpMaksimal
                        } else {
                            StatusJawabanReview::TkpSebagian
                        };
                    }
                }
                StatusJawabanReview::TkpSebagian
            } else {
                StatusJawabanReview::Salah
            }
        } else {
            StatusJawabanReview::Kosong
        }
    }

    /// Load gambar dari file ke texture egui, dengan cache agar tidak reload berulang.
    /// Mengembalikan None jika file tidak ditemukan atau gagal dibaca.
    fn load_texture(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> Option<egui::TextureHandle> {
        // Kembalikan dari cache jika sudah ada
        if let Some(tex) = self.texture_cache.get(path) {
            return Some(tex.clone());
        }

        // Load dari disk
        let img = image::open(path).ok()?;
        let img_rgba = img.to_rgba8();
        let (w, h) = img_rgba.dimensions();
        let pixels: Vec<egui::Color32> = img_rgba
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        let color_image = egui::ColorImage {
            size: [w as usize, h as usize],
            pixels,
        };

        let texture = ctx.load_texture(
            path,
            color_image,
            egui::TextureOptions::LINEAR,
        );

        self.texture_cache.insert(path.to_string(), texture.clone());
        Some(texture)
    }

    /// Render gambar soal/pembahasan dengan lebar maksimum dan aspect ratio terjaga.
    fn render_image(
        ui: &mut Ui,
        texture: &egui::TextureHandle,
        max_width: f32,
    ) {
        let orig = texture.size_vec2();
        let scale = (max_width / orig.x).min(1.0); // tidak diperbesar, hanya diperkecil
        let display_size = orig * scale;

        let (rect, _) = ui.allocate_exact_size(display_size, egui::Sense::hover());
        ui.painter().image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}

// ============================================================================
// 4. HELPERS UI
// ============================================================================

/// Warna aksen utama aplikasi
mod palette {
    use eframe::egui::Color32;
    pub const BG_APP: Color32 = Color32::from_rgb(15, 17, 22);
    pub const BG_CARD: Color32 = Color32::from_rgb(23, 27, 35);
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(28, 33, 43);
    pub const BG_HEADER: Color32 = Color32::from_rgb(20, 24, 32);
    pub const BORDER: Color32 = Color32::from_rgb(38, 44, 58);
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(32, 37, 48);
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(236, 240, 248);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 158, 175);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(90, 100, 118);
    pub const BLUE: Color32 = Color32::from_rgb(59, 130, 246);
    pub const BLUE_DARK: Color32 = Color32::from_rgb(37, 99, 235);
    pub const GREEN: Color32 = Color32::from_rgb(34, 197, 94);
    pub const GREEN_DARK: Color32 = Color32::from_rgb(22, 101, 52);
    pub const RED: Color32 = Color32::from_rgb(239, 68, 68);
    pub const RED_DARK: Color32 = Color32::from_rgb(153, 27, 27);
    pub const AMBER: Color32 = Color32::from_rgb(245, 158, 11);

    pub const TWK_COLOR: Color32 = Color32::from_rgb(56, 139, 202);
    pub const TIU_COLOR: Color32 = Color32::from_rgb(220, 90, 20);
    pub const TKP_COLOR: Color32 = Color32::from_rgb(34, 180, 100);
}

fn card_frame(fill: Color32, border: Color32, rounding: f32) -> Frame {
    Frame::none()
        .fill(fill)
        .rounding(Rounding::same(rounding))
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(0.0))
}

fn pill_frame(fill: Color32) -> Frame {
    Frame::none()
        .fill(fill)
        .rounding(Rounding::same(5.0))
        .inner_margin(Margin::symmetric(10.0, 4.0))
}

// ============================================================================
// 5. IMPLEMENTASI RENDERING HALAMAN
// ============================================================================

impl eframe::App for SimulasiCpnsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Countdown timer
        if self.layar == Layar::Ujian && !self.mode_tanpa_waktu {
            if let Some(start) = self.waktu_mulai {
                let elapsed = start.elapsed();
                if elapsed >= self.durasi_ujian {
                    self.selesaikan_ujian();
                } else {
                    ctx.request_repaint_after(Duration::from_millis(500));
                }
            }
        }

        egui::CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(palette::BG_APP)
                    .inner_margin(Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                match self.layar {
                    Layar::Menu => self.render_menu_utama(ui),
                    Layar::Ujian => self.render_ujian(ui, ctx),
                    Layar::Hasil => self.render_hasil(ui),
                    Layar::Pembahasan => self.render_pembahasan(ui, ctx),
                }
            });
    }
}

impl SimulasiCpnsApp {
    // -------------------------------------------------------------------------
    // HALAMAN 1: MENU UTAMA
    // -------------------------------------------------------------------------
    fn render_menu_utama(&mut self, ui: &mut Ui) {
        // Wrapper scroll untuk jendela kecil
        ScrollArea::vertical().id_salt("menu_scroll").show(ui, |ui| {
            ui.add_space(40.0);

            ui.vertical_centered(|ui| {
                // ---- Logo / Badge ----
                pill_frame(Color32::from_rgb(30, 58, 110))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("🏛  BADAN KEPEGAWAIAN NEGARA")
                                .size(11.5)
                                .strong()
                                .color(Color32::from_rgb(147, 197, 253)),
                        );
                    });

                ui.add_space(14.0);

                // ---- Judul ----
                ui.label(
                    RichText::new("Simulasi Ujian CAT CPNS")
                        .size(38.0)
                        .strong()
                        .color(palette::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("Latihan Seleksi Kompetensi Dasar (SKD) — Standar Resmi BKN")
                        .size(15.0)
                        .color(palette::TEXT_SECONDARY),
                );

                ui.add_space(32.0);

                // ---- Kartu Ketentuan Ujian ----
                card_frame(palette::BG_CARD, palette::BORDER, 12.0)
                    .inner_margin(Margin::same(28.0))
                    .show(ui, |ui| {
                        ui.set_max_width(760.0);

                        ui.label(
                            RichText::new("Format & Ketentuan Seleksi Kompetensi Dasar")
                                .size(17.0)
                                .strong()
                                .color(palette::TEXT_PRIMARY),
                        );
                        ui.add_space(16.0);

                        // Tiga stat box
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(14.0, 14.0);
                            Self::render_stat_box(
                                ui, "TWK", "30 Soal",
                                "Passing Grade: 65 / 150",
                                "Tes Wawasan Kebangsaan",
                                palette::TWK_COLOR,
                            );
                            Self::render_stat_box(
                                ui, "TIU", "35 Soal",
                                "Passing Grade: 80 / 175",
                                "Tes Intelegensi Umum",
                                palette::TIU_COLOR,
                            );
                            Self::render_stat_box(
                                ui, "TKP", "45 Soal",
                                "Passing Grade: 166 / 225",
                                "Tes Karakteristik Pribadi",
                                palette::TKP_COLOR,
                            );
                        });

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(14.0);

                        // Daftar aturan
                        let rules = [
                            ("📋", "Total Soal", "110 butir (30 TWK + 35 TIU + 45 TKP) diacak secara acak"),
                            ("⏱", "Durasi Ujian", "100 menit (ada pilihan mode tanpa batas waktu)"),
                            ("🏆", "Sistem Skoring", "TWK/TIU: +5 benar, 0 salah/kosong • TKP: 1—5 poin bertingkat"),
                            ("📖", "Pembahasan", "Pembahasan lengkap tersedia setelah ujian selesai"),
                            ("✅", "Syarat Lulus", "Wajib melampaui Passing Grade di KETIGA kategori sekaligus"),
                        ];

                        for (icon, judul, deskripsi) in &rules {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(*icon).size(15.0));
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(format!("{}: ", judul))
                                        .size(13.5)
                                        .strong()
                                        .color(palette::TEXT_PRIMARY),
                                );
                                ui.label(
                                    RichText::new(*deskripsi)
                                        .size(13.5)
                                        .color(palette::TEXT_SECONDARY),
                                );
                            });
                            ui.add_space(5.0);
                        }
                    });

                ui.add_space(14.0);

                // ---- Pemilih Dataset Soal ----
                card_frame(palette::BG_CARD, palette::BORDER, 12.0)
                    .inner_margin(Margin::same(20.0))
                    .show(ui, |ui| {
                        ui.set_max_width(760.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("🗂  Pilih Dataset Soal")
                                    .size(15.0)
                                    .strong()
                                    .color(palette::TEXT_PRIMARY),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let n_dataset = self.daftar_dataset.len().saturating_sub(1);
                                ui.label(
                                    RichText::new(format!("{} file dataset tersedia", n_dataset))
                                        .size(11.5)
                                        .color(palette::TEXT_MUTED),
                                );
                            });
                        });

                        ui.add_space(10.0);

                        if self.daftar_dataset.is_empty() {
                            ui.label(
                                RichText::new("⚠ Tidak ada file dataset JSON ditemukan di direktori ini.")
                                    .size(13.0)
                                    .color(Color32::from_rgb(200, 150, 60)),
                            );
                        } else {
                            // Dropdown ComboBox
                            let selected_label = self
                                .daftar_dataset
                                .get(self.dataset_terpilih_idx)
                                .map(|d| d.label.clone())
                                .unwrap_or_else(|| "— pilih dataset —".to_string());

                            let mut changed = false;
                            egui::ComboBox::from_id_salt("dataset_picker")
                                .selected_text(
                                    RichText::new(&selected_label)
                                        .size(13.5)
                                        .color(palette::TEXT_PRIMARY),
                                )
                                .width(620.0)
                                .show_ui(ui, |ui| {
                                    for (i, ds) in self.daftar_dataset.iter().enumerate() {
                                        let is_selected = i == self.dataset_terpilih_idx;
                                        let txt = if ds.error.is_some() {
                                            RichText::new(&ds.label)
                                                .size(13.0)
                                                .color(Color32::from_rgb(200, 100, 60))
                                        } else if i == 0 {
                                            RichText::new(&ds.label)
                                                .size(13.0)
                                                .strong()
                                                .color(Color32::from_rgb(147, 197, 253))
                                        } else {
                                            RichText::new(&ds.label)
                                                .size(13.0)
                                                .color(palette::TEXT_PRIMARY)
                                        };
                                        if ui.selectable_label(is_selected, txt).clicked() {
                                            if !is_selected {
                                                self.dataset_terpilih_idx = i;
                                                changed = true;
                                            }
                                        }
                                    }
                                });

                            if changed {
                                self.reload_dataset();
                            }

                            // Info detail dataset terpilih
                            ui.add_space(8.0);
                            if let Some(ds) = self.daftar_dataset.get(self.dataset_terpilih_idx) {
                                if let Some(ref err) = ds.error {
                                    ui.label(
                                        RichText::new(format!("⚠ Error: {}", err))
                                            .size(12.0)
                                            .color(Color32::from_rgb(200, 100, 60)),
                                    );
                                } else {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 16.0;
                                        pill_frame(Color32::from_rgba_premultiplied(14, 35, 55, 255)).show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("TWK  {}", ds.twk))
                                                    .size(12.5)
                                                    .strong()
                                                    .color(palette::TWK_COLOR),
                                            );
                                        });
                                        pill_frame(Color32::from_rgba_premultiplied(55, 22, 5, 255)).show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("TIU  {}", ds.tiu))
                                                    .size(12.5)
                                                    .strong()
                                                    .color(palette::TIU_COLOR),
                                            );
                                        });
                                        pill_frame(Color32::from_rgba_premultiplied(5, 45, 20, 255)).show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("TKP  {}", ds.tkp))
                                                    .size(12.5)
                                                    .strong()
                                                    .color(palette::TKP_COLOR),
                                            );
                                        });
                                        ui.label(
                                            RichText::new(format!("│  Total {} soal", ds.total))
                                                .size(12.5)
                                                .color(palette::TEXT_SECONDARY),
                                        );
                                    });
                                }
                            }
                        }
                    });

                ui.add_space(14.0);

                // Status dataset
                let info_color = if self.info_dataset.starts_with('✓') {
                    Color32::from_rgb(74, 155, 95)
                } else {
                    Color32::from_rgb(200, 150, 60)
                };
                ui.label(
                    RichText::new(&self.info_dataset)
                        .size(12.0)
                        .italics()
                        .color(info_color),
                );

                ui.add_space(28.0);

                // ---- Tombol Mulai ----
                ui.label(
                    RichText::new("Pilih Mode Ujian")
                        .size(14.0)
                        .strong()
                        .color(palette::TEXT_SECONDARY),
                );
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.add_space(0.0); // centering trick via vertical_centered outer

                    // Tombol Dengan Waktu
                    let btn_waktu = egui::Button::new(
                        RichText::new("⏱  Mulai Dengan Waktu")
                            .size(16.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(palette::BLUE_DARK)
                    .rounding(Rounding::same(8.0));

                    if ui.add_sized([220.0, 50.0], btn_waktu).clicked() {
                        self.mulai_tryout(false);
                    }

                    ui.add_space(16.0);

                    // Tombol Tanpa Waktu
                    let btn_tanpa = egui::Button::new(
                        RichText::new("∞  Mulai Tanpa Batas Waktu")
                            .size(16.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(88, 60, 180))
                    .rounding(Rounding::same(8.0));

                    if ui.add_sized([250.0, 50.0], btn_tanpa).clicked() {
                        self.mulai_tryout(true);
                    }
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(0.0);
                    ui.label(
                        RichText::new("⏱ Dengan Waktu: Timer 100 menit berjalan otomatis")
                            .size(11.5)
                            .color(palette::TEXT_MUTED),
                    );
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new("∞ Tanpa Waktu: Kerjakan soal santai tanpa tekanan timer")
                            .size(11.5)
                            .color(palette::TEXT_MUTED),
                    );
                });

                ui.add_space(40.0);
            });
        });
    }

    fn render_stat_box(
        ui: &mut Ui,
        kode: &str,
        jumlah: &str,
        pg: &str,
        nama_panjang: &str,
        warna: Color32,
    ) {
        Frame::none()
            .fill(Color32::from_rgba_premultiplied(
                warna.r() / 8,
                warna.g() / 8,
                warna.b() / 8,
                255,
            ))
            .rounding(Rounding::same(10.0))
            .stroke(Stroke::new(1.5, warna))
            .inner_margin(Margin::same(16.0))
            .show(ui, |ui| {
                ui.set_width(216.0);

                // Kode badge
                pill_frame(warna).show(ui, |ui| {
                    ui.label(RichText::new(kode).size(13.0).strong().color(Color32::WHITE));
                });
                ui.add_space(8.0);

                ui.label(RichText::new(nama_panjang).size(12.0).color(Color32::from_rgb(180, 190, 205)));
                ui.add_space(4.0);
                ui.label(RichText::new(jumlah).size(22.0).strong().color(Color32::WHITE));
                ui.add_space(2.0);
                ui.label(RichText::new(pg).size(12.0).color(Color32::from_rgb(160, 170, 185)));
            });
    }

    // -------------------------------------------------------------------------
    // HALAMAN 2: UJIAN TRYOUT
    // -------------------------------------------------------------------------
    fn render_ujian(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        if self.soal_aktif.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Tidak ada soal aktif.").color(palette::TEXT_SECONDARY));
            });
            return;
        }

        let total_soal = self.soal_aktif.len();
        let soal_saat_ini = Rc::clone(&self.soal_aktif[self.index_soal]);

        // Hitung sisa waktu
        let sisa_detik = if self.mode_tanpa_waktu {
            None
        } else {
            match self.waktu_mulai {
                Some(start) => {
                    let elapsed = start.elapsed().as_secs();
                    Some(if elapsed < TOTAL_DURASI_DETIK { TOTAL_DURASI_DETIK - elapsed } else { 0 })
                }
                None => Some(TOTAL_DURASI_DETIK),
            }
        };

        // Warna & nama kategori
        let (badge_color, kategori_nama) = if soal_saat_ini.is_twk() {
            (palette::TWK_COLOR, "TES WAWASAN KEBANGSAAN")
        } else if soal_saat_ini.is_tiu() {
            (palette::TIU_COLOR, "TES INTELEGENSI UMUM")
        } else {
            (palette::TKP_COLOR, "TES KARAKTERISTIK PRIBADI")
        };

        // Progress soal terjawab
        let terjawab = self.jawaban.len();
        let pct = terjawab as f32 / total_soal as f32;

        // ============ HEADER ============
        Frame::none()
            .fill(palette::BG_HEADER)
            .stroke(Stroke::new(1.0, palette::BORDER))
            .inner_margin(Margin::symmetric(20.0, 12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Badge kategori
                    pill_frame(badge_color).show(ui, |ui| {
                        ui.label(
                            RichText::new(kategori_nama)
                                .size(12.0)
                                .strong()
                                .color(Color32::WHITE),
                        );
                    });

                    ui.add_space(10.0);

                    ui.label(
                        RichText::new(format!("Soal {} / {}", self.index_soal + 1, total_soal))
                            .size(14.0)
                            .strong()
                            .color(palette::TEXT_PRIMARY),
                    );

                    ui.add_space(10.0);

                    // Mini progress bar teks
                    ui.label(
                        RichText::new(format!("({} terjawab)", terjawab))
                            .size(12.5)
                            .color(palette::TEXT_MUTED),
                    );

                    // Timer di kanan
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(sisa) = sisa_detik {
                            let menit = sisa / 60;
                            let detik = sisa % 60;
                            let timer_color = if sisa < 300 {
                                palette::RED
                            } else if sisa < 600 {
                                palette::AMBER
                            } else {
                                palette::GREEN
                            };
                            ui.label(
                                RichText::new(format!("⏱  {:02}:{:02}", menit, detik))
                                    .size(16.0)
                                    .strong()
                                    .color(timer_color),
                            );
                        } else {
                            ui.label(
                                RichText::new("∞  Tanpa Batas Waktu")
                                    .size(14.0)
                                    .strong()
                                    .color(Color32::from_rgb(160, 130, 230)),
                            );
                        }
                    });
                });

                // Progress bar soal
                ui.add_space(8.0);
                let bar_rect = {
                    let avail = ui.available_width();
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(avail, 4.0), egui::Sense::hover());
                    rect
                };
                let painter = ui.painter();
                painter.rect_filled(bar_rect, Rounding::same(2.0), palette::BORDER);
                let fill_w = (bar_rect.width() * pct).max(0.0);
                if fill_w > 0.0 {
                    let fill_rect = egui::Rect::from_min_size(
                        bar_rect.min,
                        Vec2::new(fill_w, bar_rect.height()),
                    );
                    painter.rect_filled(fill_rect, Rounding::same(2.0), palette::BLUE);
                }
            });

        ui.add_space(10.0);

        // ============ BODY (Soal Kiri + Navigasi Kanan) ============
        // Pakai item_spacing.x eksplisit agar gap kolom tidak ikut melebar
        let gap_kolom = 10.0;
        let lebar_tersedia = ui.available_width();
        let tinggi_tersedia = (ui.available_height() - 70.0).max(200.0);
        // Hitung sidebar lebar dulu, lalu sisa untuk soal
        let lebar_sidebar = (lebar_tersedia * 0.27).clamp(220.0, 280.0);
        let lebar_soal = (lebar_tersedia - lebar_sidebar - gap_kolom).max(280.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0; // kelola gap manual

            // ---- Kolom Kiri: Pertanyaan & Opsi ----
            ui.allocate_ui_with_layout(
                Vec2::new(lebar_soal, tinggi_tersedia),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ScrollArea::vertical().id_salt("area_soal").show(ui, |ui| {
                        card_frame(palette::BG_CARD, palette::BORDER, 10.0)
                            .inner_margin(Margin::same(22.0))
                            .show(ui, |ui| {
                                // Nomor soal inline
                                ui.horizontal(|ui| {
                                    pill_frame(palette::BG_HEADER).show(ui, |ui| {
                                        ui.label(
                                            RichText::new(format!("No. {}", self.index_soal + 1))
                                                .size(12.5)
                                                .strong()
                                                .color(palette::TEXT_SECONDARY),
                                        );
                                    });
                                });
                                ui.add_space(12.0);

                                // Teks pertanyaan
                                ui.label(
                                    RichText::new(&soal_saat_ini.pertanyaan)
                                        .size(15.5)
                                        .color(palette::TEXT_PRIMARY),
                                );

                                // Gambar soal (jika ada)
                                if let Some(ref path_gambar) = soal_saat_ini.gambar_soal.clone() {
                                    if let Some(tex) = self.load_texture(ctx, path_gambar) {
                                        ui.add_space(14.0);
                                        Frame::none()
                                            .fill(Color32::from_rgb(10, 12, 18))
                                            .rounding(Rounding::same(8.0))
                                            .stroke(Stroke::new(1.0, palette::BORDER))
                                            .inner_margin(Margin::same(10.0))
                                            .show(ui, |ui| {
                                                let max_w = (ui.available_width() - 20.0).max(100.0);
                                                Self::render_image(ui, &tex, max_w);
                                            });
                                    } else {
                                        ui.add_space(8.0);
                                        ui.label(
                                            RichText::new(format!("⚠ Gambar tidak ditemukan: {}", path_gambar))
                                                .size(12.0)
                                                .color(palette::AMBER),
                                        );
                                    }
                                }

                                ui.add_space(18.0);
                                ui.separator();
                                ui.add_space(14.0);

                                // Opsi jawaban
                                let opsi_huruf = ["A", "B", "C", "D", "E"];
                                let mut terpilih = self.jawaban.get(&soal_saat_ini.id).copied();

                                for (idx, opsi_teks) in soal_saat_ini.opsi.iter().enumerate() {
                                    let huruf = opsi_huruf.get(idx).copied().unwrap_or("?");
                                    let is_selected = terpilih == Some(idx);

                                    let (opt_fill, opt_stroke) = if is_selected {
                                        (
                                            Color32::from_rgb(28, 46, 80),
                                            Stroke::new(2.0, palette::BLUE),
                                        )
                                    } else {
                                        (
                                            palette::BG_CARD_HOVER,
                                            Stroke::new(1.0, palette::BORDER_SUBTLE),
                                        )
                                    };

                                    let resp = Frame::none()
                                        .fill(opt_fill)
                                        .rounding(Rounding::same(8.0))
                                        .stroke(opt_stroke)
                                        .inner_margin(Margin::symmetric(14.0, 11.0))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Radio button
                                                if ui.radio(is_selected, "").clicked() {
                                                    terpilih = Some(idx);
                                                }

                                                // Huruf badge
                                                let huruf_color = if is_selected {
                                                    palette::BLUE
                                                } else {
                                                    palette::TEXT_MUTED
                                                };
                                                ui.label(
                                                    RichText::new(format!("{}.", huruf))
                                                        .size(14.0)
                                                        .strong()
                                                        .color(huruf_color),
                                                );

                                                // Teks opsi
                                                let label = ui.add(
                                                    egui::Label::new(
                                                        RichText::new(opsi_teks)
                                                            .size(14.0)
                                                            .color(if is_selected {
                                                                palette::TEXT_PRIMARY
                                                            } else {
                                                                palette::TEXT_SECONDARY
                                                            }),
                                                    )
                                                    .wrap(),
                                                );
                                                if label.clicked() {
                                                    terpilih = Some(idx);
                                                }
                                            });
                                        });

                                    if resp.response.clicked() {
                                        terpilih = Some(idx);
                                    }

                                    ui.add_space(8.0);
                                }

                                if let Some(pilihan) = terpilih {
                                    self.jawaban.insert(soal_saat_ini.id, pilihan);
                                }
                            });
                    });
                },
            );

            // ---- Kolom Kanan: Navigasi Grid ----
            ui.add_space(gap_kolom);
            ui.allocate_ui_with_layout(
                Vec2::new(lebar_sidebar, tinggi_tersedia),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ScrollArea::vertical()
                        .id_salt("area_palette")
                        .show(ui, |ui| {
                        card_frame(palette::BG_CARD, palette::BORDER, 10.0)
                            .inner_margin(Margin::symmetric(12.0, 14.0))
                            .show(ui, |ui| {
                                ui.set_max_width(lebar_sidebar - 24.0);
                                ui.label(
                                    RichText::new("Navigasi Soal")
                                        .size(14.0)
                                        .strong()
                                        .color(palette::TEXT_PRIMARY),
                                );
                                ui.add_space(10.0);

                                // Legend
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 6.0);
                                    Self::render_legend(ui, palette::GREEN_DARK, "Terjawab");
                                    Self::render_legend(ui, Color32::from_rgb(180, 110, 20), "Ragu-ragu");
                                    Self::render_legend(ui, Color32::from_rgb(40, 46, 60), "Kosong");
                                });

                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(10.0);

                                // Grid responsif
                                // Gunakan lebar sidebar minus padding card (2 * 12px) dan scrollbar (~12px)
                                let spacing = 5.0;
                                let avail_w = (lebar_sidebar - 24.0 - 12.0).max(0.0);
                                let min_btn = 32.0;
                                let cols = (((avail_w + spacing) / (min_btn + spacing)).floor() as usize).max(4).min(7);
                                let btn_w = ((avail_w - spacing * (cols - 1) as f32) / cols as f32).floor().max(28.0);

                                egui::Grid::new("grid_nomor_soal")
                                    .spacing(Vec2::new(spacing, spacing))
                                    .show(ui, |ui| {
                                        for idx in 0..total_soal {
                                            let s = &self.soal_aktif[idx];
                                            let is_current = idx == self.index_soal;
                                            let is_answered = self.jawaban.contains_key(&s.id);
                                            let is_ragu = *self.ragu_ragu.get(&s.id).unwrap_or(&false);

                                            let btn_fill = if is_ragu {
                                                Color32::from_rgb(180, 110, 20)
                                            } else if is_answered {
                                                Color32::from_rgb(30, 110, 60)
                                            } else {
                                                Color32::from_rgb(32, 38, 50)
                                            };

                                            let stroke = if is_current {
                                                Stroke::new(2.5, Color32::WHITE)
                                            } else {
                                                Stroke::NONE
                                            };

                                            let nomor_btn = egui::Button::new(
                                                RichText::new(format!("{}", idx + 1))
                                                    .size(12.0)
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            )
                                            .fill(btn_fill)
                                            .stroke(stroke)
                                            .rounding(Rounding::same(5.0));

                                            if ui.add_sized([btn_w, 32.0], nomor_btn).clicked() {
                                                self.index_soal = idx;
                                            }

                                            if (idx + 1) % cols == 0 {
                                                ui.end_row();
                                            }
                                        }
                                        if total_soal % cols != 0 {
                                            ui.end_row();
                                        }
                                    });

                                ui.add_space(12.0);
                                ui.separator();
                                ui.add_space(10.0);

                                // Ringkasan jumlah
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("✓ {}", terjawab)).size(13.0).strong().color(palette::GREEN));
                                    ui.add_space(8.0);
                                    ui.label(RichText::new(format!("○ {}", total_soal - terjawab)).size(13.0).color(palette::TEXT_MUTED));
                                });
                            });
                    });
                },
            );
        });

        ui.add_space(10.0);

        // ============ FOOTER CONTROLS ============
        Frame::none()
            .fill(palette::BG_HEADER)
            .stroke(Stroke::new(1.0, palette::BORDER))
            .inner_margin(Margin::symmetric(20.0, 10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Tombol Sebelumnya
                    let prev_en = self.index_soal > 0;
                    if ui
                        .add_enabled(
                            prev_en,
                            egui::Button::new(RichText::new("«  Sebelumnya").size(13.5))
                                .min_size(Vec2::new(120.0, 36.0)),
                        )
                        .clicked()
                    {
                        self.index_soal -= 1;
                    }

                    ui.add_space(10.0);

                    // Tombol Selanjutnya
                    let next_en = self.index_soal + 1 < total_soal;
                    if ui
                        .add_enabled(
                            next_en,
                            egui::Button::new(RichText::new("Selanjutnya  »").size(13.5))
                                .min_size(Vec2::new(120.0, 36.0)),
                        )
                        .clicked()
                    {
                        self.index_soal += 1;
                    }

                    ui.add_space(16.0);

                    // Checkbox ragu-ragu
                    let mut ragu = *self.ragu_ragu.get(&soal_saat_ini.id).unwrap_or(&false);
                    if ui
                        .checkbox(
                            &mut ragu,
                            RichText::new("⚑  Tandai Ragu-Ragu")
                                .size(13.5)
                                .color(palette::AMBER),
                        )
                        .changed()
                    {
                        self.ragu_ragu.insert(soal_saat_ini.id, ragu);
                    }

                    // Tombol Akhiri (kanan)
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let btn_selesai = egui::Button::new(
                            RichText::new("  Akhiri Ujian  ")
                                .size(13.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(palette::RED_DARK)
                        .rounding(Rounding::same(6.0));

                        if ui.add_sized([130.0, 36.0], btn_selesai).clicked() {
                            self.dialog_konfirmasi = true;
                        }
                    });
                });
            });

        // ============ MODAL KONFIRMASI ============
        if self.dialog_konfirmasi {
            let belum = total_soal.saturating_sub(self.jawaban.len());

            egui::Window::new("Konfirmasi Akhiri Ujian")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(
                    Frame::window(&ctx.style())
                        .fill(Color32::from_rgb(22, 26, 34))
                        .stroke(Stroke::new(1.5, palette::RED))
                        .inner_margin(Margin::same(24.0)),
                )
                .show(ctx, |ui| {
                    ui.set_width(400.0);
                    ui.label(
                        RichText::new("Apakah Anda yakin ingin mengakhiri ujian?")
                            .size(16.0)
                            .strong()
                            .color(palette::TEXT_PRIMARY),
                    );
                    ui.add_space(12.0);

                    if belum > 0 {
                        ui.label(
                            RichText::new(format!(
                                "⚠  Masih terdapat {} soal yang BELUM dijawab!",
                                belum
                            ))
                            .size(13.5)
                            .strong()
                            .color(palette::AMBER),
                        );
                    } else {
                        ui.label(
                            RichText::new("✓ Semua soal telah dijawab.")
                                .size(13.5)
                                .color(palette::GREEN),
                        );
                    }

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(
                                [178.0, 38.0],
                                egui::Button::new(RichText::new("Lanjutkan Ujian").size(14.0)),
                            )
                            .clicked()
                        {
                            self.dialog_konfirmasi = false;
                        }

                        ui.add_space(8.0);

                        let ya_btn = egui::Button::new(
                            RichText::new("Ya, Selesaikan").strong().color(Color32::WHITE).size(14.0),
                        )
                        .fill(palette::RED_DARK)
                        .rounding(Rounding::same(6.0));

                        if ui.add_sized([178.0, 38.0], ya_btn).clicked() {
                            self.selesaikan_ujian();
                        }
                    });
                });
        }
    }

    fn render_legend(ui: &mut Ui, warna: Color32, label: &str) {
        ui.horizontal(|ui| {
            Frame::none()
                .fill(warna)
                .rounding(Rounding::same(3.0))
                .inner_margin(Margin::symmetric(6.0, 5.0))
                .show(ui, |_| {});
            ui.label(RichText::new(label).size(11.0).color(palette::TEXT_MUTED));
        });
    }

    // -------------------------------------------------------------------------
    // HALAMAN 3: HASIL TRYOUT
    // -------------------------------------------------------------------------
    fn render_hasil(&mut self, ui: &mut Ui) {
        let hasil = match &self.hasil {
            Some(h) => h.clone(),
            None => return,
        };

        ScrollArea::vertical().id_salt("hasil_scroll").show(ui, |ui| {
            ui.add_space(28.0);
            ui.vertical_centered(|ui| {
                // ---- Judul ----
                ui.label(
                    RichText::new("LAPORAN HASIL SIMULASI SKD CPNS")
                        .size(26.0)
                        .strong()
                        .color(palette::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("Berdasarkan Peraturan Menteri PANRB & Standar Kelulusan BKN")
                        .size(13.5)
                        .color(palette::TEXT_SECONDARY),
                );

                if hasil.mode_tanpa_waktu {
                    ui.add_space(6.0);
                    pill_frame(Color32::from_rgb(50, 30, 100)).show(ui, |ui| {
                        ui.label(
                            RichText::new("∞ Mode: Tanpa Batas Waktu")
                                .size(12.0)
                                .color(Color32::from_rgb(180, 150, 255)),
                        );
                    });
                }

                ui.add_space(20.0);

                // ---- Banner Kelulusan ----
                let (bg_banner, ikon, teks_status, sub_status) = if hasil.lulus_keseluruhan {
                    (
                        Color32::from_rgb(14, 60, 35),
                        "🎉",
                        "SELAMAT! ANDA DINYATAKAN LULUS",
                        "Nilai Anda di ketiga subtes melampaui Nilai Ambang Batas Resmi BKN.",
                    )
                } else {
                    (
                        Color32::from_rgb(80, 15, 15),
                        "📋",
                        "BELUM MEMENUHI PASSING GRADE",
                        "Ada satu atau lebih subtes yang masih di bawah Nilai Ambang Batas BKN.",
                    )
                };

                Frame::none()
                    .fill(bg_banner)
                    .rounding(Rounding::same(12.0))
                    .stroke(Stroke::new(
                        1.5,
                        if hasil.lulus_keseluruhan { palette::GREEN } else { palette::RED },
                    ))
                    .inner_margin(Margin::symmetric(28.0, 20.0))
                    .show(ui, |ui| {
                        ui.set_max_width(760.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(ikon).size(28.0));
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(teks_status)
                                        .size(18.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                );
                                ui.label(
                                    RichText::new(sub_status)
                                        .size(13.0)
                                        .color(Color32::from_rgb(220, 225, 235)),
                                );
                            });
                        });
                    });

                ui.add_space(22.0);

                // ---- Tiga Kartu Skor ----
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(16.0, 16.0);
                    Self::render_hasil_kartu(ui, "TWK", "Wawasan Kebangsaan", hasil.skor_twk, PG_TWK, 150, hasil.lulus_twk, palette::TWK_COLOR);
                    Self::render_hasil_kartu(ui, "TIU", "Intelegensi Umum", hasil.skor_tiu, PG_TIU, 175, hasil.lulus_tiu, palette::TIU_COLOR);
                    Self::render_hasil_kartu(ui, "TKP", "Karakteristik Pribadi", hasil.skor_tkp, PG_TKP, 225, hasil.lulus_tkp, palette::TKP_COLOR);
                });

                ui.add_space(22.0);

                // ---- Statistik Ringkasan ----
                card_frame(palette::BG_CARD, palette::BORDER, 10.0)
                    .inner_margin(Margin::same(22.0))
                    .show(ui, |ui| {
                        ui.set_max_width(760.0);

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(50.0, 14.0);

                            // Total skor
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Total Nilai SKD").size(12.5).color(palette::TEXT_SECONDARY));
                                ui.label(
                                    RichText::new(format!("{} / 550", hasil.total_skor))
                                        .size(28.0)
                                        .strong()
                                        .color(palette::TEXT_PRIMARY),
                                );
                            });

                            // Soal dijawab
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Soal Dijawab").size(12.5).color(palette::TEXT_SECONDARY));
                                ui.label(
                                    RichText::new(format!("{} / {}", hasil.jumlah_terjawab, hasil.total_soal))
                                        .size(22.0)
                                        .strong()
                                        .color(palette::TEXT_PRIMARY),
                                );
                            });

                            // Waktu
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Waktu Digunakan").size(12.5).color(palette::TEXT_SECONDARY));
                                if hasil.mode_tanpa_waktu {
                                    let m = hasil.waktu_pengerjaan.as_secs() / 60;
                                    let s = hasil.waktu_pengerjaan.as_secs() % 60;
                                    ui.label(
                                        RichText::new(format!("{}m {}s", m, s))
                                            .size(22.0)
                                            .strong()
                                            .color(Color32::from_rgb(160, 130, 230)),
                                    );
                                } else {
                                    let m = hasil.waktu_pengerjaan.as_secs() / 60;
                                    let s = hasil.waktu_pengerjaan.as_secs() % 60;
                                    ui.label(
                                        RichText::new(format!("{}m {}s", m, s))
                                            .size(22.0)
                                            .strong()
                                            .color(palette::TEXT_PRIMARY),
                                    );
                                }
                            });

                            // Akurasi
                            ui.vertical(|ui| {
                                let benar: usize = self.soal_aktif.iter().filter(|s| {
                                    matches!(self.cek_status_review(s), StatusJawabanReview::Benar | StatusJawabanReview::TkpMaksimal)
                                }).count();
                                let akurasi = if hasil.jumlah_terjawab > 0 {
                                    benar * 100 / hasil.jumlah_terjawab
                                } else { 0 };
                                ui.label(RichText::new("Akurasi").size(12.5).color(palette::TEXT_SECONDARY));
                                ui.label(
                                    RichText::new(format!("{}%", akurasi))
                                        .size(22.0)
                                        .strong()
                                        .color(if akurasi >= 70 { palette::GREEN } else { palette::AMBER }),
                                );
                            });
                        });
                    });

                ui.add_space(28.0);

                // ---- Tombol Navigasi ----
                ui.horizontal(|ui| {
                    let btn_pembahasan = egui::Button::new(
                        RichText::new("🔍  Lihat Pembahasan Lengkap")
                            .size(15.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(5, 130, 90))
                    .rounding(Rounding::same(8.0));

                    if ui.add_sized([280.0, 46.0], btn_pembahasan).clicked() {
                        self.index_pembahasan = 0;
                        self.filter_pembahasan = FilterPembahasan::Semua;
                        self.layar = Layar::Pembahasan;
                    }

                    ui.add_space(14.0);

                    let btn_kembali = egui::Button::new(
                        RichText::new("↩  Kembali ke Menu")
                            .size(15.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(palette::BLUE_DARK)
                    .rounding(Rounding::same(8.0));

                    if ui.add_sized([200.0, 46.0], btn_kembali).clicked() {
                        self.layar = Layar::Menu;
                    }
                });

                ui.add_space(28.0);
            });
        });
    }

    fn render_hasil_kartu(
        ui: &mut Ui,
        kode: &str,
        nama: &str,
        skor: u32,
        pg: u32,
        maks: u32,
        lulus: bool,
        warna: Color32,
    ) {
        let bg = Color32::from_rgba_premultiplied(
            warna.r() / 8, warna.g() / 8, warna.b() / 8, 255,
        );
        Frame::none()
            .fill(bg)
            .rounding(Rounding::same(12.0))
            .stroke(Stroke::new(2.0, warna))
            .inner_margin(Margin::same(20.0))
            .show(ui, |ui| {
                ui.set_width(230.0);

                // Header kode + nama
                pill_frame(warna).show(ui, |ui| {
                    ui.label(RichText::new(kode).size(13.0).strong().color(Color32::WHITE));
                });
                ui.add_space(4.0);
                ui.label(RichText::new(nama).size(12.0).color(Color32::from_rgb(180, 190, 205)));
                ui.add_space(10.0);

                // Skor besar
                ui.label(
                    RichText::new(format!("{}", skor))
                        .size(40.0)
                        .strong()
                        .color(Color32::WHITE),
                );

                // Progress bar mini
                let pct = (skor as f32 / maks as f32).clamp(0.0, 1.0);
                let bar_w = 190.0_f32;
                let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, 6.0), egui::Sense::hover());
                ui.painter().rect_filled(bar_rect, Rounding::same(3.0), palette::BORDER);
                let fill_w = (bar_w * pct).max(0.0);
                if fill_w > 0.0 {
                    let fill_rect = egui::Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, 6.0));
                    ui.painter().rect_filled(
                        fill_rect,
                        Rounding::same(3.0),
                        if lulus { palette::GREEN } else { palette::RED },
                    );
                }

                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("Maks: {}  |  PG: {}", maks, pg))
                        .size(12.0)
                        .color(Color32::from_rgb(150, 160, 175)),
                );

                ui.add_space(10.0);

                // Badge lulus/tidak
                if lulus {
                    pill_frame(Color32::from_rgb(14, 70, 40)).show(ui, |ui| {
                        ui.label(RichText::new("✓  LULUS").size(13.0).strong().color(palette::GREEN));
                    });
                } else {
                    pill_frame(Color32::from_rgb(80, 14, 14)).show(ui, |ui| {
                        ui.label(RichText::new("✗  TIDAK LULUS").size(13.0).strong().color(palette::RED));
                    });
                }
            });
    }

    // -------------------------------------------------------------------------
    // HALAMAN 4: PEMBAHASAN SOAL
    // -------------------------------------------------------------------------
    fn render_pembahasan(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        if self.soal_aktif.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Tidak ada soal untuk dibahas.");
            });
            if ui.button("Kembali ke Menu").clicked() {
                self.layar = Layar::Menu;
            }
            return;
        }

        let total_soal = self.soal_aktif.len();
        if self.index_pembahasan >= total_soal {
            self.index_pembahasan = 0;
        }

        let soal = Rc::clone(&self.soal_aktif[self.index_pembahasan]);
        let status_jawaban = self.cek_status_review(&soal);

        let (badge_color, kategori_nama) = if soal.is_twk() {
            (palette::TWK_COLOR, "TWK — Wawasan Kebangsaan")
        } else if soal.is_tiu() {
            (palette::TIU_COLOR, "TIU — Intelegensi Umum")
        } else {
            (palette::TKP_COLOR, "TKP — Karakteristik Pribadi")
        };

        // ============ HEADER ============
        Frame::none()
            .fill(palette::BG_HEADER)
            .stroke(Stroke::new(1.0, palette::BORDER))
            .inner_margin(Margin::symmetric(20.0, 12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    pill_frame(badge_color).show(ui, |ui| {
                        ui.label(
                            RichText::new(kategori_nama)
                                .size(12.0)
                                .strong()
                                .color(Color32::WHITE),
                        );
                    });

                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(format!("Pembahasan {} / {}", self.index_pembahasan + 1, total_soal))
                            .size(14.0)
                            .strong()
                            .color(palette::TEXT_PRIMARY),
                    );

                    ui.add_space(10.0);

                    // Pill status
                    match status_jawaban {
                        StatusJawabanReview::Benar => {
                            pill_frame(Color32::from_rgb(14, 70, 40)).show(ui, |ui| {
                                ui.label(RichText::new("✓ BENAR (+5)").size(12.0).strong().color(palette::GREEN));
                            });
                        }
                        StatusJawabanReview::Salah => {
                            pill_frame(Color32::from_rgb(80, 14, 14)).show(ui, |ui| {
                                ui.label(RichText::new("✗ SALAH (0)").size(12.0).strong().color(palette::RED));
                            });
                        }
                        StatusJawabanReview::Kosong => {
                            pill_frame(Color32::from_rgb(40, 46, 60)).show(ui, |ui| {
                                ui.label(RichText::new("— KOSONG (0)").size(12.0).color(palette::TEXT_MUTED));
                            });
                        }
                        StatusJawabanReview::TkpMaksimal => {
                            pill_frame(Color32::from_rgb(14, 70, 40)).show(ui, |ui| {
                                ui.label(RichText::new("★ MAKS (5/5)").size(12.0).strong().color(palette::GREEN));
                            });
                        }
                        StatusJawabanReview::TkpSebagian => {
                            let poin = self.jawaban.get(&soal.id).and_then(|&idx| {
                                soal.bobot_tkp.as_ref().and_then(|b| b.get(idx).copied())
                            }).unwrap_or(0);
                            pill_frame(Color32::from_rgb(80, 45, 10)).show(ui, |ui| {
                                ui.label(RichText::new(format!("★ {} / 5 Poin", poin)).size(12.0).strong().color(palette::AMBER));
                            });
                        }
                    }

                    // Tombol kembali ke hasil
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(RichText::new("↩ Kembali ke Hasil").size(13.0))
                                    .min_size(Vec2::new(140.0, 32.0)),
                            )
                            .clicked()
                        {
                            self.layar = Layar::Hasil;
                        }
                    });
                });
            });

        ui.add_space(8.0);

        // ============ FILTER BAR ============
        Frame::none()
            .fill(palette::BG_HEADER)
            .stroke(Stroke::new(1.0, palette::BORDER))
            .inner_margin(Margin::symmetric(20.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Filter:").size(12.5).color(palette::TEXT_MUTED));
                    ui.add_space(6.0);

                    let filters: &[(&str, FilterPembahasan)] = &[
                        ("Semua", FilterPembahasan::Semua),
                        ("Salah/Kurang", FilterPembahasan::SalahAtauKurang),
                        ("TWK", FilterPembahasan::Twk),
                        ("TIU", FilterPembahasan::Tiu),
                        ("TKP", FilterPembahasan::Tkp),
                    ];

                    for (label, filter) in filters {
                        let aktif = self.filter_pembahasan == *filter;
                        let bg = if aktif { palette::BLUE_DARK } else { palette::BG_CARD_HOVER };
                        let txt = if aktif { Color32::WHITE } else { palette::TEXT_SECONDARY };

                        if ui
                            .add(
                                egui::Button::new(RichText::new(*label).size(12.5).color(txt))
                                    .fill(bg)
                                    .rounding(Rounding::same(5.0)),
                            )
                            .clicked()
                        {
                            self.filter_pembahasan = *filter;
                        }
                    }
                });
            });

        ui.add_space(8.0);

        // ============ BODY (Detail + Grid) ============
        let gap_kolom = 10.0;
        let lebar_tersedia = ui.available_width() - gap_kolom;
        let tinggi_tersedia = (ui.available_height() - 70.0).max(200.0);
        let lebar_sidebar = (lebar_tersedia * 0.27).clamp(220.0, 290.0);
        let lebar_detail = (lebar_tersedia - lebar_sidebar).max(300.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap_kolom;

            // ---- Detail Kiri ----
            ui.allocate_ui_with_layout(
                Vec2::new(lebar_detail, tinggi_tersedia),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ScrollArea::vertical().id_salt("scroll_pembahasan_detail").show(ui, |ui| {
                        card_frame(palette::BG_CARD, palette::BORDER, 10.0)
                            .inner_margin(Margin::same(22.0))
                            .show(ui, |ui| {
                                // Pertanyaan
                                ui.label(
                                    RichText::new(&soal.pertanyaan)
                                        .size(15.0)
                                        .color(palette::TEXT_PRIMARY),
                                );

                                // Gambar soal (jika ada)
                                if let Some(ref path_gambar) = soal.gambar_soal.clone() {
                                    if let Some(tex) = self.load_texture(ctx, path_gambar) {
                                        ui.add_space(14.0);
                                        Frame::none()
                                            .fill(Color32::from_rgb(10, 12, 18))
                                            .rounding(Rounding::same(8.0))
                                            .stroke(Stroke::new(1.0, palette::BORDER))
                                            .inner_margin(Margin::same(10.0))
                                            .show(ui, |ui| {
                                                let max_w = (ui.available_width() - 20.0).max(100.0);
                                                Self::render_image(ui, &tex, max_w);
                                            });
                                    } else {
                                        ui.add_space(8.0);
                                        ui.label(
                                            RichText::new(format!("⚠ Gambar tidak ditemukan: {}", path_gambar))
                                                .size(12.0)
                                                .color(palette::AMBER),
                                        );
                                    }
                                }

                                ui.add_space(16.0);
                                ui.separator();
                                ui.add_space(14.0);

                                // Opsi dengan indikator
                                let opsi_huruf = ["A", "B", "C", "D", "E"];
                                let jawaban_user = self.jawaban.get(&soal.id).copied();

                                for (idx, opsi_teks) in soal.opsi.iter().enumerate() {
                                    let huruf = opsi_huruf.get(idx).copied().unwrap_or("?");
                                    let is_user = jawaban_user == Some(idx);

                                    let is_kunci = if soal.is_twk() || soal.is_tiu() {
                                        soal.kunci_jawaban.as_deref().unwrap_or("").eq_ignore_ascii_case(huruf)
                                    } else if soal.is_tkp() {
                                        soal.bobot_tkp.as_ref().and_then(|b| b.get(idx).copied()).unwrap_or(0) == 5
                                    } else {
                                        false
                                    };

                                    let (opt_bg, opt_stroke) = if is_kunci {
                                        (Color32::from_rgb(10, 38, 22), Stroke::new(1.5, palette::GREEN))
                                    } else if is_user {
                                        (Color32::from_rgb(45, 10, 15), Stroke::new(1.5, palette::RED))
                                    } else {
                                        (palette::BG_CARD_HOVER, Stroke::new(1.0, palette::BORDER_SUBTLE))
                                    };

                                    Frame::none()
                                        .fill(opt_bg)
                                        .rounding(Rounding::same(8.0))
                                        .stroke(opt_stroke)
                                        .inner_margin(Margin::symmetric(14.0, 10.0))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Ikon status
                                                if is_kunci {
                                                    ui.label(RichText::new("✓").size(14.0).strong().color(palette::GREEN));
                                                } else if is_user {
                                                    ui.label(RichText::new("✗").size(14.0).strong().color(palette::RED));
                                                } else {
                                                    ui.label(RichText::new("·").size(14.0).color(palette::TEXT_MUTED));
                                                }

                                                // Huruf
                                                let huruf_col = if is_kunci { palette::GREEN }
                                                    else if is_user { palette::RED }
                                                    else { palette::TEXT_MUTED };
                                                ui.label(RichText::new(format!("{}.", huruf)).size(13.5).strong().color(huruf_col));

                                                // Teks + tag
                                                let mut teks = opsi_teks.clone();
                                                if is_kunci && is_user {
                                                    teks.push_str("  ✓ KUNCI & PILIHAN ANDA");
                                                } else if is_kunci {
                                                    teks.push_str("  ✓ KUNCI JAWABAN");
                                                } else if is_user {
                                                    teks.push_str("  ✗ PILIHAN ANDA");
                                                }
                                                if soal.is_tkp() {
                                                    if let Some(ref bobot) = soal.bobot_tkp {
                                                        if let Some(&p) = bobot.get(idx) {
                                                            teks.push_str(&format!("  ({} Poin)", p));
                                                        }
                                                    }
                                                }

                                                let txt_col = if is_kunci { Color32::from_rgb(134, 239, 172) }
                                                    else if is_user { Color32::from_rgb(252, 165, 165) }
                                                    else { palette::TEXT_SECONDARY };

                                                ui.add(egui::Label::new(RichText::new(teks).size(13.5).color(txt_col)).wrap());
                                            });
                                        });

                                    ui.add_space(7.0);
                                }
                            });

                        ui.add_space(14.0);

                        // Kotak Pembahasan
                        card_frame(Color32::from_rgb(16, 22, 36), Color32::from_rgb(50, 100, 180), 10.0)
                            .inner_margin(Margin::same(20.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("💡  PEMBAHASAN & ANALISIS JAWABAN")
                                        .size(14.5)
                                        .strong()
                                        .color(Color32::from_rgb(96, 165, 250)),
                                );
                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(12.0);

                                let teks_p = match &soal.pembahasan {
                                    Some(p) if !p.trim().is_empty() => p.as_str(),
                                    _ => "Pembahasan untuk soal ini belum tersedia.",
                                };
                                ui.label(
                                    RichText::new(teks_p)
                                        .size(14.0)
                                        .line_height(Some(22.0))
                                        .color(palette::TEXT_PRIMARY),
                                );

                                // Gambar pembahasan (jika ada)
                                if let Some(ref path_gambar) = soal.gambar_pembahasan.clone() {
                                    if let Some(tex) = self.load_texture(ctx, path_gambar) {
                                        ui.add_space(14.0);
                                        ui.label(
                                            RichText::new("📷 Visualisasi Pembahasan:")
                                                .size(12.5)
                                                .strong()
                                                .color(Color32::from_rgb(96, 165, 250)),
                                        );
                                        ui.add_space(8.0);
                                        Frame::none()
                                            .fill(Color32::from_rgb(10, 12, 18))
                                            .rounding(Rounding::same(8.0))
                                            .stroke(Stroke::new(1.0, Color32::from_rgb(50, 100, 180)))
                                            .inner_margin(Margin::same(10.0))
                                            .show(ui, |ui| {
                                                let max_w = (ui.available_width() - 20.0).max(100.0);
                                                Self::render_image(ui, &tex, max_w);
                                            });
                                    }
                                }

                                ui.add_space(14.0);

                                let tip = if soal.is_twk() {
                                    "📌 Tips TWK: Pelajari UUD 1945, Pancasila, sejarah proklamasi, dan regulasi ASN."
                                } else if soal.is_tiu() {
                                    "📌 Tips TIU: Latih eliminasi cepat, pola deret, silogisme, dan perhitungan mental."
                                } else {
                                    "📌 Tips TKP: Utamakan tindakan berorientasi pelayanan, integritas, dan kolaborasi."
                                };

                                ui.label(
                                    RichText::new(tip)
                                        .size(12.0)
                                        .italics()
                                        .color(Color32::from_rgb(130, 170, 240)),
                                );
                            });
                    });
                },
            );

            // ---- Grid Kanan ----
            let sisa_lebar = ui.available_width();
            ui.allocate_ui_with_layout(
                Vec2::new(sisa_lebar, tinggi_tersedia),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ScrollArea::vertical()
                        .id_salt("scroll_pembahasan_palette")
                        .show(ui, |ui| {
                        card_frame(palette::BG_CARD, palette::BORDER, 10.0)
                            .inner_margin(Margin::symmetric(12.0, 14.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("Navigasi Pembahasan")
                                        .size(14.0)
                                        .strong()
                                        .color(palette::TEXT_PRIMARY),
                                );
                                ui.add_space(10.0);

                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 6.0);
                                    Self::render_legend(ui, palette::GREEN_DARK, "Benar/5Poin");
                                    Self::render_legend(ui, Color32::from_rgb(140, 60, 10), "TKP 1-4");
                                    Self::render_legend(ui, palette::RED_DARK, "Salah");
                                    Self::render_legend(ui, Color32::from_rgb(40, 46, 60), "Kosong");
                                });

                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(10.0);

                                let spacing = 5.0;
                                let avail_w = (ui.available_width() - 1.0).max(0.0);
                                let min_btn = 32.0;
                                let cols = (((avail_w + spacing) / (min_btn + spacing)).floor() as usize).max(4).min(7);
                                let btn_w = ((avail_w - spacing * (cols - 1) as f32) / cols as f32).floor().max(28.0);

                                egui::Grid::new("grid_nomor_pembahasan")
                                    .spacing(Vec2::new(spacing, spacing))
                                    .show(ui, |ui| {
                                        for idx in 0..total_soal {
                                            let s = &self.soal_aktif[idx];
                                            let is_current = idx == self.index_pembahasan;
                                            let st = self.cek_status_review(s);

                                            let match_filter = match self.filter_pembahasan {
                                                FilterPembahasan::Semua => true,
                                                FilterPembahasan::SalahAtauKurang => {
                                                    st == StatusJawabanReview::Salah
                                                        || st == StatusJawabanReview::Kosong
                                                        || st == StatusJawabanReview::TkpSebagian
                                                }
                                                FilterPembahasan::Twk => s.is_twk(),
                                                FilterPembahasan::Tiu => s.is_tiu(),
                                                FilterPembahasan::Tkp => s.is_tkp(),
                                            };

                                            let btn_fill = if !match_filter {
                                                Color32::from_rgb(22, 26, 34)
                                            } else {
                                                match st {
                                                    StatusJawabanReview::Benar | StatusJawabanReview::TkpMaksimal => palette::GREEN_DARK,
                                                    StatusJawabanReview::TkpSebagian => Color32::from_rgb(140, 60, 10),
                                                    StatusJawabanReview::Salah => palette::RED_DARK,
                                                    StatusJawabanReview::Kosong => Color32::from_rgb(40, 46, 60),
                                                }
                                            };

                                            let stroke = if is_current {
                                                Stroke::new(2.5, Color32::WHITE)
                                            } else {
                                                Stroke::NONE
                                            };

                                            let txt_col = if !match_filter { palette::TEXT_MUTED } else { Color32::WHITE };

                                            let nomor_btn = egui::Button::new(
                                                RichText::new(format!("{}", idx + 1))
                                                    .size(12.0)
                                                    .strong()
                                                    .color(txt_col),
                                            )
                                            .fill(btn_fill)
                                            .stroke(stroke)
                                            .rounding(Rounding::same(5.0));

                                            if ui.add_sized([btn_w, 32.0], nomor_btn).clicked() {
                                                self.index_pembahasan = idx;
                                            }

                                            if (idx + 1) % cols == 0 {
                                                ui.end_row();
                                            }
                                        }
                                        if total_soal % cols != 0 {
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
                },
            );
        });

        ui.add_space(10.0);

        // ============ FOOTER ============
        Frame::none()
            .fill(palette::BG_HEADER)
            .stroke(Stroke::new(1.0, palette::BORDER))
            .inner_margin(Margin::symmetric(20.0, 10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let prev_en = self.index_pembahasan > 0;
                    if ui
                        .add_enabled(
                            prev_en,
                            egui::Button::new(RichText::new("«  Sebelumnya").size(13.5))
                                .min_size(Vec2::new(130.0, 36.0)),
                        )
                        .clicked()
                    {
                        self.index_pembahasan -= 1;
                    }

                    ui.add_space(8.0);

                    let next_en = self.index_pembahasan + 1 < total_soal;
                    if ui
                        .add_enabled(
                            next_en,
                            egui::Button::new(RichText::new("Selanjutnya  »").size(13.5))
                                .min_size(Vec2::new(130.0, 36.0)),
                        )
                        .clicked()
                    {
                        self.index_pembahasan += 1;
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Menu Utama").size(13.5).color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(45, 55, 72))
                                .min_size(Vec2::new(110.0, 36.0)),
                            )
                            .clicked()
                        {
                            self.layar = Layar::Menu;
                        }

                        ui.add_space(10.0);

                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Ringkasan Nilai").size(13.5).strong().color(Color32::WHITE),
                                )
                                .fill(palette::BLUE_DARK)
                                .min_size(Vec2::new(140.0, 36.0)),
                            )
                            .clicked()
                        {
                            self.layar = Layar::Hasil;
                        }
                    });
                });
            });
    }
}

// ============================================================================
// 6. ENTRY POINT
// ============================================================================

fn main() -> eframe::Result<()> {
    if std::env::var("WAYLAND_DISPLAY").is_ok() && std::env::var("WINIT_UNIX_BACKEND").is_err() {
        std::env::set_var("WINIT_UNIX_BACKEND", "wayland");
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Simulasi Ujian CPNS — CAT BKN Standard")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 560.0])
            .with_active(true)
            .with_resizable(true)
            .with_transparent(false),
        ..Default::default()
    };

    eframe::run_native(
        "Simulasi Ujian CPNS",
        native_options,
        Box::new(|cc| Ok(Box::new(SimulasiCpnsApp::new(cc)))),
    )
}
