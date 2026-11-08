use std::collections::HashMap;
use std::fs::File;
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
}

impl Soal {
    pub fn is_twk(&self) -> bool {
        self.kategori.eq_ignore_ascii_case("twk")
    }

    pub fn is_tiu(&self) -> bool {
        self.kategori.eq_ignore_ascii_case("tiu")
    }

    pub fn is_tkp(&self) -> bool {
        self.kategori.eq_ignore_ascii_case("tkp")
    }
}

// Master Pool untuk menyimpan soal terorganisir per kategori
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
            if rc.is_twk() {
                pools.pool_twk.push(rc);
            } else if rc.is_tiu() {
                pools.pool_tiu.push(rc);
            } else if rc.is_tkp() {
                pools.pool_tkp.push(rc);
            }
        }

        Ok(pools)
    }

    pub fn total_soal(&self) -> usize {
        self.pool_twk.len() + self.pool_tiu.len() + self.pool_tkp.len()
    }

    pub fn fallback_sample() -> Self {
        // Fallback dataset jika file dataset_soal.json belum dibuat
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
pub const TARGET_TOTAL: usize = TARGET_TWK + TARGET_TIU + TARGET_TKP; // 110 Soal

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
}

// ============================================================================
// 3. STATE MANAGEMENT APLIKASI
// ============================================================================

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Layar {
    Menu,
    Ujian,
    Hasil,
}

pub struct SimulasiCpnsApp {
    pools: MasterPools,
    info_dataset: String,
    layar: Layar,

    // Exam runtime state
    soal_aktif: Vec<Rc<Soal>>,
    jawaban: HashMap<u32, usize>, // id_soal -> index opsi (0..4)
    ragu_ragu: HashMap<u32, bool>, // id_soal -> bool
    index_soal: usize,
    waktu_mulai: Option<Instant>,
    durasi_ujian: Duration,
    dialog_konfirmasi: bool,

    // Result state
    hasil: Option<HasilUjian>,
}

impl SimulasiCpnsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Coba load dataset_soal.json dari direktori saat ini
        let path = PathBuf::from("dataset_soal.json");
        let (pools, info) = match MasterPools::from_file(&path) {
            Ok(p) => {
                let info = format!(
                    "Berhasil memuat dataset '{}': TWK={} | TIU={} | TKP={} (Total={})",
                    path.display(),
                    p.pool_twk.len(),
                    p.pool_tiu.len(),
                    p.pool_tkp.len(),
                    p.total_soal()
                );
                (p, info)
            }
            Err(e) => {
                let p = MasterPools::fallback_sample();
                let info = format!(
                    "Perhatian: File 'dataset_soal.json' tidak ditemukan ({}). Menggunakan dataset sampel bawaan (TWK={} | TIU={} | TKP={}).",
                    e,
                    p.pool_twk.len(),
                    p.pool_tiu.len(),
                    p.pool_tkp.len()
                );
                (p, info)
            }
        };

        Self {
            pools,
            info_dataset: info,
            layar: Layar::Menu,
            soal_aktif: Vec::new(),
            jawaban: HashMap::new(),
            ragu_ragu: HashMap::new(),
            index_soal: 0,
            waktu_mulai: None,
            durasi_ujian: Duration::from_secs(TOTAL_DURASI_DETIK),
            dialog_konfirmasi: false,
            hasil: None,
        }
    }

    pub fn mulai_tryout(&mut self) {
        let mut rng = rand::thread_rng();

        // 1. Ambil tepat 30 TWK, 35 TIU, 45 TKP menggunakan choose_multiple (atau fallback jika kurang)
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

        // 2. Gabungkan 110 soal ke dalam satu Vector soal_aktif
        let mut soal_gabungan = Vec::with_capacity(ambil_twk.len() + ambil_tiu.len() + ambil_tkp.len());
        soal_gabungan.extend(ambil_twk);
        soal_gabungan.extend(ambil_tiu);
        soal_gabungan.extend(ambil_tkp);

        // 3. Acak (shuffle) urutannya secara total agar tidak berurutan per kategori
        soal_gabungan.shuffle(&mut rng);

        self.soal_aktif = soal_gabungan;
        self.jawaban.clear();
        self.ragu_ragu.clear();
        self.index_soal = 0;
        self.waktu_mulai = Some(Instant::now());
        self.dialog_konfirmasi = false;
        self.hasil = None;
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

        // Iterasi soal_aktif dan hitung skor
        for soal in &self.soal_aktif {
            if let Some(&pilihan_idx) = self.jawaban.get(&soal.id) {
                terjawab += 1;
                let huruf_dipilih = opsi_huruf.get(pilihan_idx).copied().unwrap_or("");

                if soal.is_twk() {
                    if let Some(ref kunci) = soal.kunci_jawaban {
                        if kunci.eq_ignore_ascii_case(huruf_dipilih) {
                            skor_twk += 5;
                        }
                    }
                } else if soal.is_tiu() {
                    if let Some(ref kunci) = soal.kunci_jawaban {
                        if kunci.eq_ignore_ascii_case(huruf_dipilih) {
                            skor_tiu += 5;
                        }
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
        });

        self.dialog_konfirmasi = false;
        self.layar = Layar::Hasil;
    }
}

// ============================================================================
// 4. DESAIN ANTARMUKA EFRAME / EGUI
// ============================================================================

impl eframe::App for SimulasiCpnsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Pengaturan countdown real-time tanpa memblokir thread
        if self.layar == Layar::Ujian {
            if let Some(start) = self.waktu_mulai {
                let elapsed = start.elapsed();
                if elapsed >= self.durasi_ujian {
                    self.selesaikan_ujian();
                } else {
                    // Refresh UI setiap 250ms secara halus untuk pembaruan countdown Wayland
                    ctx.request_repaint_after(Duration::from_millis(250));
                }
            }
        }

        egui::CentralPanel::default()
            .frame(Frame::default().fill(Color32::from_rgb(18, 20, 24)).inner_margin(Margin::same(16.0)))
            .show(ctx, |ui| {
                match self.layar {
                    Layar::Menu => self.render_menu_utama(ui),
                    Layar::Ujian => self.render_ujian(ui, ctx),
                    Layar::Hasil => self.render_hasil(ui),
                }
            });
    }
}

impl SimulasiCpnsApp {
    // ------------------------------------------------------------------------
    // HALAMAN 1: MENU UTAMA
    // ------------------------------------------------------------------------
    fn render_menu_utama(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);

            // Judul Aplikasi
            ui.label(
                RichText::new("SIMULASI UJIAN CAT CPNS")
                    .size(34.0)
                    .strong()
                    .color(Color32::from_rgb(230, 235, 245)),
            );
            ui.label(
                RichText::new("Berdasarkan Standar Resmi Badan Kepegawaian Negara (BKN)")
                    .size(15.0)
                    .color(Color32::from_rgb(150, 160, 175)),
            );

            ui.add_space(30.0);

            // Kotak Informasi Aturan Ujian
            Frame::none()
                .fill(Color32::from_rgb(26, 30, 38))
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, Color32::from_rgb(45, 52, 65)))
                .inner_margin(Margin::same(24.0))
                .show(ui, |ui| {
                    ui.set_max_width(700.0);

                    ui.heading(
                        RichText::new("Ketentuan & Format Seleksi Kompetensi Dasar (SKD)")
                            .size(18.0)
                            .color(Color32::WHITE),
                    );
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        Self::render_stat_box(ui, "TWK", "30 Soal", "Ambang Batas: 65", Color32::from_rgb(41, 128, 185));
                        Self::render_stat_box(ui, "TIU", "35 Soal", "Ambang Batas: 80", Color32::from_rgb(211, 84, 0));
                        Self::render_stat_box(ui, "TKP", "45 Soal", "Ambang Batas: 166", Color32::from_rgb(39, 174, 96));
                    });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(12.0);

                    ui.label(RichText::new("• Total Soal: 110 Butir (Diacak secara acak total)").color(Color32::from_rgb(210, 215, 225)));
                    ui.label(RichText::new("• Durasi Ujian: 100 Menit (Timer hitung mundur non-stop)").color(Color32::from_rgb(210, 215, 225)));
                    ui.label(RichText::new("• Sistem Skoring: TWK/TIU (+5 benar, 0 salah/kosong), TKP (1 s/d 5 poin bertingkat)").color(Color32::from_rgb(210, 215, 225)));
                    ui.label(RichText::new("• Syarat Kelulusan: Wajib melampaui Passing Grade di KETIGA kategori sekaligus").color(Color32::from_rgb(241, 196, 15)));
                });

            ui.add_space(20.0);

            // Status Dataset
            ui.label(
                RichText::new(&self.info_dataset)
                    .size(12.5)
                    .italics()
                    .color(Color32::from_rgb(140, 150, 165)),
            );

            ui.add_space(30.0);

            // Tombol Mulai Ujian
            let btn = egui::Button::new(
                RichText::new("  MULAI TRYOUT SEKARANG  ")
                    .size(18.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(37, 99, 235))
            .rounding(Rounding::same(6.0));

            if ui.add_sized([260.0, 48.0], btn).clicked() {
                self.mulai_tryout();
            }

            ui.add_space(20.0);
        });
    }

    fn render_stat_box(ui: &mut Ui, judul: &str, jumlah: &str, pg: &str, warna: Color32) {
        Frame::none()
            .fill(Color32::from_rgb(34, 39, 49))
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::new(1.0, warna))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_width(190.0);
                ui.label(RichText::new(judul).strong().size(16.0).color(warna));
                ui.label(RichText::new(jumlah).size(15.0).strong().color(Color32::WHITE));
                ui.label(RichText::new(pg).size(11.5).color(Color32::from_rgb(180, 185, 195)));
            });
    }

    // ------------------------------------------------------------------------
    // HALAMAN 2: UJIAN TRYOUT
    // ------------------------------------------------------------------------
    fn render_ujian(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        if self.soal_aktif.is_empty() {
            ui.label("Tidak ada soal aktif.");
            return;
        }

        let total_soal = self.soal_aktif.len();
        let soal_saat_ini = Rc::clone(&self.soal_aktif[self.index_soal]);

        // Hitung sisa waktu
        let sisa_detik = match self.waktu_mulai {
            Some(start) => {
                let elapsed = start.elapsed().as_secs();
                if elapsed < TOTAL_DURASI_DETIK {
                    TOTAL_DURASI_DETIK - elapsed
                } else {
                    0
                }
            }
            None => TOTAL_DURASI_DETIK,
        };

        let menit = sisa_detik / 60;
        let detik = sisa_detik % 60;

        // Warna badge kategori
        let (badge_color, kategori_nama) = if soal_saat_ini.is_twk() {
            (Color32::from_rgb(41, 128, 185), "TES WAWASAN KEBANGSAAN (TWK)")
        } else if soal_saat_ini.is_tiu() {
            (Color32::from_rgb(211, 84, 0), "TES INTELEGENSI UMUM (TIU)")
        } else {
            (Color32::from_rgb(39, 174, 96), "TES KARAKTERISTIK PRIBADI (TKP)")
        };

        // ---------------- HEADER BAR ----------------
        Frame::none()
            .fill(Color32::from_rgb(26, 30, 38))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(16.0, 12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Badge Kategori
                    Frame::none()
                        .fill(badge_color)
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(kategori_nama).size(13.0).strong().color(Color32::WHITE));
                        });

                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(format!("Soal {} dari {}", self.index_soal + 1, total_soal))
                            .size(15.0)
                            .strong()
                            .color(Color32::from_rgb(220, 225, 235)),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let timer_color = if sisa_detik < 300 {
                            Color32::from_rgb(231, 76, 60) // Merah jika < 5 menit
                        } else {
                            Color32::from_rgb(46, 204, 113) // Hijau normal
                        };

                        ui.label(
                            RichText::new(format!("⏱ Sisa Waktu: {:02}:{:02}", menit, detik))
                                .size(17.0)
                                .strong()
                                .color(timer_color),
                        );
                    });
                });
            });

        ui.add_space(10.0);

        // ---------------- BODY UTAMA (Soal + Grid Nomor) ----------------
        let lebar_tersedia = ui.available_width();
        let tinggi_tersedia = (ui.available_height() - 60.0).max(100.0);

        ui.horizontal(|ui| {
            // Kolom Kiri: Pertanyaan dan Opsi Jawaban (72% lebar)
            ui.allocate_ui_with_layout(
                Vec2::new(lebar_tersedia * 0.72, tinggi_tersedia),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ScrollArea::vertical()
                        .id_salt("area_soal")
                        .show(ui, |ui| {
                            Frame::none()
                                .fill(Color32::from_rgb(24, 28, 35))
                                .rounding(Rounding::same(8.0))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(40, 46, 58)))
                                .inner_margin(Margin::same(20.0))
                                .show(ui, |ui| {
                                    // Teks Pertanyaan (Word wrap penuh)
                                    ui.label(
                                        RichText::new(&soal_saat_ini.pertanyaan)
                                            .size(16.0)
                                            .color(Color32::from_rgb(240, 243, 248)),
                                    );

                                    ui.add_space(20.0);
                                    ui.separator();
                                    ui.add_space(16.0);

                                    // Opsi Jawaban Vertikal RadioButton
                                    let opsi_huruf = ["A", "B", "C", "D", "E"];
                                    let mut terpilih = self.jawaban.get(&soal_saat_ini.id).copied();

                                    for (idx, opsi_teks) in soal_saat_ini.opsi.iter().enumerate() {
                                        let huruf = opsi_huruf.get(idx).copied().unwrap_or("?");
                                        let is_selected = terpilih == Some(idx);

                                        let opt_frame_color = if is_selected {
                                            Color32::from_rgb(33, 44, 66)
                                        } else {
                                            Color32::from_rgb(29, 34, 43)
                                        };

                                        let opt_stroke = if is_selected {
                                            Stroke::new(1.5, Color32::from_rgb(59, 130, 246))
                                        } else {
                                            Stroke::new(1.0, Color32::from_rgb(45, 52, 65))
                                        };

                                        Frame::none()
                                            .fill(opt_frame_color)
                                            .rounding(Rounding::same(6.0))
                                            .stroke(opt_stroke)
                                            .inner_margin(Margin::symmetric(14.0, 10.0))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    if ui.radio(is_selected, "").clicked() {
                                                        terpilih = Some(idx);
                                                    }

                                                    let label_text = format!("{}. {}", huruf, opsi_teks);
                                                    let label = ui.add(
                                                        egui::Label::new(
                                                            RichText::new(label_text)
                                                                .size(14.5)
                                                                .color(if is_selected {
                                                                    Color32::WHITE
                                                                } else {
                                                                    Color32::from_rgb(205, 212, 222)
                                                                }),
                                                        )
                                                        .wrap(),
                                                    );

                                                    if label.clicked() {
                                                        terpilih = Some(idx);
                                                    }
                                                });
                                            });

                                        ui.add_space(8.0);
                                    }

                                    if let Some(pilihan) = terpilih {
                                        self.jawaban.insert(soal_saat_ini.id, pilihan);
                                    }
                                });
                        });
                },
            );

            ui.add_space(8.0);

            // Kolom Kanan: Navigasi Nomor Soal (Palette / Grid)
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), tinggi_tersedia),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ScrollArea::vertical()
                        .id_salt("area_palette")
                        .show(ui, |ui| {
                            Frame::none()
                                .fill(Color32::from_rgb(24, 28, 35))
                                .rounding(Rounding::same(8.0))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(40, 46, 58)))
                                .inner_margin(Margin::same(12.0))
                                .show(ui, |ui| {
                                    ui.heading(
                                        RichText::new("Navigasi Soal")
                                            .size(15.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                    ui.add_space(8.0);

                                    // Legend indikator
                                    ui.horizontal_wrapped(|ui| {
                                        Self::render_legend(ui, Color32::from_rgb(39, 174, 96), "Terjawab");
                                        Self::render_legend(ui, Color32::from_rgb(243, 156, 18), "Ragu");
                                        Self::render_legend(ui, Color32::from_rgb(60, 68, 82), "Kosong");
                                    });

                                    ui.add_space(10.0);
                                    ui.separator();
                                    ui.add_space(10.0);

                                    // Grid nomor soal
                                    let jumlah_kolom = 5;
                                    egui::Grid::new("grid_nomor_soal")
                                        .spacing(Vec2::new(6.0, 6.0))
                                        .show(ui, |ui| {
                                            for idx in 0..total_soal {
                                                let s = &self.soal_aktif[idx];
                                                let is_current = idx == self.index_soal;
                                                let is_answered = self.jawaban.contains_key(&s.id);
                                                let is_ragu = *self.ragu_ragu.get(&s.id).unwrap_or(&false);

                                                let btn_fill = if is_ragu {
                                                    Color32::from_rgb(243, 156, 18)
                                                } else if is_answered {
                                                    Color32::from_rgb(39, 174, 96)
                                                } else {
                                                    Color32::from_rgb(48, 54, 66)
                                                };

                                                let stroke = if is_current {
                                                    Stroke::new(2.0, Color32::WHITE)
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
                                                .rounding(Rounding::same(4.0));

                                                if ui.add_sized([34.0, 30.0], nomor_btn).clicked() {
                                                    self.index_soal = idx;
                                                }

                                                if (idx + 1) % jumlah_kolom == 0 {
                                                    ui.end_row();
                                                }
                                            }
                                        });
                                });
                        });
                },
            );
        });

        ui.add_space(12.0);

        // ---------------- FOOTER CONTROLS BAR ----------------
        Frame::none()
            .fill(Color32::from_rgb(26, 30, 38))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(16.0, 10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Tombol Sebelumnya
                    let prev_enabled = self.index_soal > 0;
                    if ui
                        .add_enabled(
                            prev_enabled,
                            egui::Button::new(RichText::new("« Sebelumnya").size(14.0)).min_size(Vec2::new(120.0, 36.0)),
                        )
                        .clicked()
                    {
                        self.index_soal -= 1;
                    }

                    // Checkbox Ragu-Ragu
                    let mut ragu = *self.ragu_ragu.get(&soal_saat_ini.id).unwrap_or(&false);
                    if ui
                        .checkbox(
                            &mut ragu,
                            RichText::new("Tandai Ragu-Ragu").size(14.0).color(Color32::from_rgb(241, 196, 15)),
                        )
                        .changed()
                    {
                        self.ragu_ragu.insert(soal_saat_ini.id, ragu);
                    }

                    // Tombol Selanjutnya
                    let next_enabled = self.index_soal + 1 < total_soal;
                    if ui
                        .add_enabled(
                            next_enabled,
                            egui::Button::new(RichText::new("Selanjutnya »").size(14.0)).min_size(Vec2::new(120.0, 36.0)),
                        )
                        .clicked()
                    {
                        self.index_soal += 1;
                    }

                    // Tombol Selesai (Kanan)
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let btn_selesai = egui::Button::new(
                            RichText::new("  Akhiri Ujian  ")
                                .size(14.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(220, 38, 38))
                        .rounding(Rounding::same(4.0));

                        if ui.add_sized([130.0, 36.0], btn_selesai).clicked() {
                            self.dialog_konfirmasi = true;
                        }
                    });
                });
            });

        // ---------------- MODAL KONFIRMASI AKHIRI UJIAN ----------------
        if self.dialog_konfirmasi {
            let belum_dijawab = total_soal.saturating_sub(self.jawaban.len());

            egui::Window::new("Konfirmasi Akhiri Ujian")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(
                    Frame::window(&ctx.style())
                        .fill(Color32::from_rgb(28, 32, 40))
                        .stroke(Stroke::new(1.5, Color32::from_rgb(220, 38, 38)))
                        .inner_margin(Margin::same(20.0)),
                )
                .show(ctx, |ui| {
                    ui.set_width(380.0);
                    ui.heading(
                        RichText::new("Apakah Anda yakin ingin mengakhiri ujian?")
                            .size(16.0)
                            .color(Color32::WHITE),
                    );

                    ui.add_space(10.0);
                    if belum_dijawab > 0 {
                        ui.label(
                            RichText::new(format!(
                                "⚠️ Peringatan: Masih terdapat {} soal yang BELUM dijawab!",
                                belum_dijawab
                            ))
                            .color(Color32::from_rgb(243, 156, 18))
                            .size(14.0)
                            .strong(),
                        );
                    } else {
                        ui.label(
                            RichText::new("Hebat! Anda telah menjawab seluruh 110 butir soal.")
                                .color(Color32::from_rgb(46, 204, 113))
                                .size(14.0),
                        );
                    }

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([170.0, 36.0], egui::Button::new("Lanjutkan Ujian"))
                            .clicked()
                        {
                            self.dialog_konfirmasi = false;
                        }

                        let ya_btn = egui::Button::new(
                            RichText::new("Ya, Selesaikan")
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(220, 38, 38));

                        if ui.add_sized([170.0, 36.0], ya_btn).clicked() {
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
                .inner_margin(Margin::same(5.0))
                .show(ui, |_| {});
            ui.label(RichText::new(label).size(11.0).color(Color32::from_rgb(170, 175, 185)));
        });
    }

    // ------------------------------------------------------------------------
    // HALAMAN 3: HASIL TRYOUT & KELULUSAN BKN
    // ------------------------------------------------------------------------
    fn render_hasil(&mut self, ui: &mut Ui) {
        let hasil = match &self.hasil {
            Some(h) => h.clone(),
            None => return,
        };

        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            ui.label(
                RichText::new("LAPORAN HASIL SIMULASI SKD CPNS")
                    .size(28.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.label(
                RichText::new("Berdasarkan Peraturan Menteri PANRB & Standar Kelulusan BKN")
                    .size(14.0)
                    .color(Color32::from_rgb(150, 160, 175)),
            );

            ui.add_space(24.0);

            // Banner Kelulusan Utama
            let (bg_banner, teks_status, sub_status) = if hasil.lulus_keseluruhan {
                (
                    Color32::from_rgb(22, 101, 52),
                    "SELAMAT! ANDA DINYATAKAN LULUS (MEMENUHI PASSING GRADE)",
                    "Nilai Anda di ketiga subtes (TWK, TIU, TKP) berhasil melampaui Nilai Ambang Batas Resmi BKN.",
                )
            } else {
                (
                    Color32::from_rgb(153, 27, 27),
                    "MOHON MAAF, ANDA TIDAK MEMENUHI PASSING GRADE",
                    "Ada satu atau lebih subtes yang nilainya masih di bawah Nilai Ambang Batas BKN.",
                )
            };

            Frame::none()
                .fill(bg_banner)
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::symmetric(24.0, 18.0))
                .show(ui, |ui| {
                    ui.set_max_width(750.0);
                    ui.label(RichText::new(teks_status).size(20.0).strong().color(Color32::WHITE));
                    ui.add_space(4.0);
                    ui.label(RichText::new(sub_status).size(13.5).color(Color32::from_rgb(240, 240, 240)));
                });

            ui.add_space(24.0);

            // Tiga Kartu Skor Subtes
            ui.horizontal(|ui| {
                Self::render_hasil_kartu(
                    ui,
                    "TWK (Wawasan Kebangsaan)",
                    hasil.skor_twk,
                    PG_TWK,
                    150,
                    hasil.lulus_twk,
                    Color32::from_rgb(41, 128, 185),
                );

                Self::render_hasil_kartu(
                    ui,
                    "TIU (Intelegensi Umum)",
                    hasil.skor_tiu,
                    PG_TIU,
                    175,
                    hasil.lulus_tiu,
                    Color32::from_rgb(211, 84, 0),
                );

                Self::render_hasil_kartu(
                    ui,
                    "TKP (Karakteristik Pribadi)",
                    hasil.skor_tkp,
                    PG_TKP,
                    225,
                    hasil.lulus_tkp,
                    Color32::from_rgb(39, 174, 96),
                );
            });

            ui.add_space(24.0);

            // Statistik Tambahan
            Frame::none()
                .fill(Color32::from_rgb(26, 30, 38))
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, Color32::from_rgb(45, 52, 65)))
                .inner_margin(Margin::same(18.0))
                .show(ui, |ui| {
                    ui.set_max_width(750.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Total Nilai SKD:").size(14.0).color(Color32::from_rgb(160, 170, 185)));
                            ui.label(
                                RichText::new(format!("{} / 550", hasil.total_skor))
                                    .size(24.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                        });

                        ui.add_space(50.0);

                        ui.vertical(|ui| {
                            ui.label(RichText::new("Soal Dijawab:").size(14.0).color(Color32::from_rgb(160, 170, 185)));
                            ui.label(
                                RichText::new(format!("{} / {}", hasil.jumlah_terjawab, hasil.total_soal))
                                    .size(20.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                        });

                        ui.add_space(50.0);

                        ui.vertical(|ui| {
                            ui.label(RichText::new("Waktu Digunakan:").size(14.0).color(Color32::from_rgb(160, 170, 185)));
                            let menit_pakai = hasil.waktu_pengerjaan.as_secs() / 60;
                            let detik_pakai = hasil.waktu_pengerjaan.as_secs() % 60;
                            ui.label(
                                RichText::new(format!("{} Menit {} Detik", menit_pakai, detik_pakai))
                                    .size(20.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                        });
                    });
                });

            ui.add_space(30.0);

            // Tombol Kembali ke Menu
            let btn_kembali = egui::Button::new(
                RichText::new("  Kembali ke Menu Utama  ")
                    .size(16.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(37, 99, 235))
            .rounding(Rounding::same(6.0));

            if ui.add_sized([240.0, 44.0], btn_kembali).clicked() {
                self.layar = Layar::Menu;
            }

            ui.add_space(20.0);
        });
    }

    fn render_hasil_kartu(
        ui: &mut Ui,
        subtes: &str,
        skor: u32,
        pg: u32,
        maks: u32,
        lulus: bool,
        subtes_color: Color32,
    ) {
        Frame::none()
            .fill(Color32::from_rgb(28, 33, 42))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.5, subtes_color))
            .inner_margin(Margin::same(16.0))
            .show(ui, |ui| {
                ui.set_width(230.0);
                ui.label(RichText::new(subtes).size(13.5).strong().color(subtes_color));
                ui.add_space(8.0);

                ui.label(
                    RichText::new(format!("{}", skor))
                        .size(36.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    RichText::new(format!("Maksimal: {}", maks))
                        .size(11.5)
                        .color(Color32::from_rgb(140, 150, 160)),
                );

                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("Passing Grade: {}", pg))
                        .size(12.5)
                        .color(Color32::from_rgb(190, 195, 205)),
                );

                ui.add_space(8.0);
                if lulus {
                    Frame::none()
                        .fill(Color32::from_rgb(22, 101, 52))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("✓ LULUS").strong().color(Color32::WHITE));
                        });
                } else {
                    Frame::none()
                        .fill(Color32::from_rgb(153, 27, 27))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("✗ TIDAK LULUS").strong().color(Color32::WHITE));
                        });
                }
            });
    }
}

// ============================================================================
// 5. ENTRY POINT & NATIVE WAYLAND CONFIGURATION
// ============================================================================

fn main() -> eframe::Result<()> {
    // Pastikan environment Wayland di Linux diprioritaskan
    if std::env::var("WAYLAND_DISPLAY").is_ok() && std::env::var("WINIT_UNIX_BACKEND").is_err() {
        std::env::set_var("WINIT_UNIX_BACKEND", "wayland");
    }

    // Konfigurasi NativeOptions yang ramah untuk Linux Wayland & Tiling Window Manager (Hyprland / Sway)
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Simulasi Ujian CPNS - CAT BKN Standard")
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([760.0, 520.0])
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
