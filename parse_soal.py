#!/usr/bin/env python3
"""
Parser soal dari hasil OCR PDF TO_dan_Pembahasan.pdf
Mengekstrak soal, opsi A-E, kunci jawaban, pembahasan, dan kategori.
"""

import os
import re
import json
import glob

OCR_DIR = "/home/fufufafa/Downloads/cpns/simulasi-cpns/ocr_pages"
OUTPUT_FILE = "/home/fufufafa/Downloads/cpns/simulasi-cpns/dataset_soal.json"

# ============================================================
# Helpers
# ============================================================

def clean(text):
    """Strip dan normalisasi whitespace."""
    return re.sub(r'\s+', ' ', text).strip()

def fix_opsi_prefix(line):
    """
    OCR kadang salah baca huruf opsi (B→8, D→>, D→D, dll).
    Kembalikan (huruf, teks) atau None jika bukan baris opsi.
    """
    # Pola normal: "A Teks...", "A. Teks...", "A) Teks..."
    m = re.match(r'^([A-Ea-e])[.\s)]\s*(.+)', line)
    if m:
        return m.group(1).upper(), m.group(2).strip()

    # OCR salah baca: "5 Teks..." (B→5), "> Teks..." (D→>), "EC Teks..." (E→E+noise)
    ocr_map = {
        '5': 'B', '8': 'B', '6': 'G',  # B  
        '>': 'D', 'd': 'D',             # D
        '1': 'I',                        # misc
    }
    first = line[0] if line else ''
    if first in ocr_map and len(line) > 1 and line[1] in (' ', '.'):
        return ocr_map[first], line[2:].strip()

    # Kadang OCR: "EC Teks..." → opsi E
    m2 = re.match(r'^[EBe][CcB]\s+(.+)', line)
    if m2:
        return 'E', m2.group(1).strip()

    return None

def extract_answer_letter(text):
    """Ekstrak huruf jawaban dari baris 'Jawaban: X' atau 'Jawaban kamu : X'."""
    # Cari "Jawaban: X" (bukan "Jawaban kamu")
    m = re.search(r'Jawaban\s*:\s*([A-Ea-e])\b(?!\s*kamu)', text, re.IGNORECASE)
    if m:
        return m.group(1).upper()
    # Fallback: cari di akhir teks
    m2 = re.search(r'\bJawaban\s*:\s*([A-Ea-e])\s*$', text, re.IGNORECASE | re.MULTILINE)
    if m2:
        return m2.group(1).upper()
    return None

def normalize_answer_in_text(text):
    """Ambil semua 'Jawaban: X' dari teks, pilih yang bukan 'Jawaban kamu'."""
    # Cari semua kemunculan "Jawaban: X" tapi BUKAN yang diikuti "kamu"
    matches = re.findall(r'Jawaban\s*:\s*([A-Ea-e])\b', text, re.IGNORECASE)
    # Filter out "Jawaban kamu : X" patterns
    kamu_matches = re.findall(r'Jawaban\s+kamu\s*:\s*([A-Ea-e])\b', text, re.IGNORECASE)
    kamu_set = set(m.upper() for m in kamu_matches)
    
    for m in matches:
        if m.upper() not in kamu_set or len(matches) > 1:
            return m.upper()
    
    if matches:
        return matches[-1].upper()
    return None


# ============================================================
# Main parser
# ============================================================

def parse_all_pages():
    txt_files = sorted(glob.glob(os.path.join(OCR_DIR, "page_*.txt")))
    print(f"Found {len(txt_files)} OCR text files")

    # Gabungkan semua teks dengan pemisah halaman
    all_text = ""
    for f in txt_files:
        with open(f, 'r', encoding='utf-8', errors='replace') as fp:
            all_text += fp.read() + "\n\f\n"

    return parse_text(all_text)


def parse_text(raw_text):
    """
    Parse teks OCR gabungan menjadi list soal.
    Strategi: scan per-soal menggunakan pola "Soal Nomor X"
    """
    # Split menjadi blok per soal
    # Pattern: "Soal Nomor N (PG)" atau "Soal NomorN (PG)"
    soal_pattern = re.compile(
        r'Soal\s+Nomor\s*(\d+)\s*\(PG\)',
        re.IGNORECASE
    )
    
    # Cari semua posisi soal
    soal_matches = list(soal_pattern.finditer(raw_text))
    print(f"Found {len(soal_matches)} soal markers")

    dataset = []
    seen_numbers = set()

    for i, match in enumerate(soal_matches):
        nomor = int(match.group(1))
        
        # Ambil teks blok soal ini (sampai soal berikutnya atau akhir)
        start = match.start()
        end = soal_matches[i+1].start() if i+1 < len(soal_matches) else len(raw_text)
        block = raw_text[start:end]

        soal_obj = parse_soal_block(nomor, block)
        
        if soal_obj is None:
            print(f"  [SKIP] Soal #{nomor}: gagal parse")
            continue

        # Dedup: skip jika nomor sudah ada (soal muncul 2x di PDF)
        soal_key = nomor
        if soal_key in seen_numbers:
            print(f"  [DUP]  Soal #{nomor}: sudah ada, skip")
            continue
        seen_numbers.add(soal_key)

        dataset.append(soal_obj)

    return dataset


def parse_soal_block(nomor, block):
    """Parse satu blok soal menjadi dict."""
    lines = [l.strip() for l in block.splitlines()]
    lines = [l for l in lines if l]  # hilangkan baris kosong

    # Cari batas: soal | pembahasan
    pembahasan_start = None
    for idx, line in enumerate(lines):
        if re.search(r'Pembahasan\s+Nomor\s*\d+', line, re.IGNORECASE):
            pembahasan_start = idx
            break

    soal_lines = lines[:pembahasan_start] if pembahasan_start else lines
    pembahasan_lines = lines[pembahasan_start:] if pembahasan_start else []

    # --- Parse bagian soal ---
    # Baris pertama adalah header "Soal Nomor X (PG)" → skip
    teks_pertanyaan_parts = []
    opsi = {}  # {'A': 'teks', 'B': 'teks', ...}
    state = "pertanyaan"  # atau "opsi"
    current_opsi_huruf = None

    for line in soal_lines[1:]:  # skip header
        # Cek apakah ini baris opsi
        result = fix_opsi_prefix(line)
        if result:
            huruf, teks = result
            if huruf in 'ABCDE':
                state = "opsi"
                current_opsi_huruf = huruf
                opsi[huruf] = teks
                continue

        if state == "pertanyaan":
            # Tambahkan ke pertanyaan
            teks_pertanyaan_parts.append(line)
        elif state == "opsi" and current_opsi_huruf:
            # Lanjutan teks opsi sebelumnya (multi-line)
            # Hanya jika bukan seperti header/noise
            if not re.match(r'^(Soal|Pembahasan|Jawaban)', line, re.IGNORECASE):
                opsi[current_opsi_huruf] += " " + line

    pertanyaan = clean(" ".join(teks_pertanyaan_parts))

    # --- Parse bagian pembahasan ---
    kunci = None
    teks_pembahasan_parts = []
    in_pembahasan_soal = False

    for line in pembahasan_lines:
        # Cari jawaban akhir
        ans = extract_answer_letter(line)
        if ans and not re.search(r'kamu', line, re.IGNORECASE):
            kunci = ans

        # Kumpulkan teks pembahasan
        if re.search(r'Pembahasan\s+Soal', line, re.IGNORECASE):
            in_pembahasan_soal = True
            continue
        if in_pembahasan_soal:
            if not re.search(r'Jawaban\s*:\s*[A-E]', line, re.IGNORECASE) or len(line) > 20:
                if not re.match(r'^(Soal\s+Nomor|Jawaban\s+kamu)', line, re.IGNORECASE):
                    teks_pembahasan_parts.append(line)

    teks_pembahasan = clean(" ".join(teks_pembahasan_parts))

    # Validasi minimal: pertanyaan ada, minimal 3 opsi
    if not pertanyaan or len(opsi) < 3:
        print(f"  [WARN] Soal #{nomor}: pertanyaan={'ok' if pertanyaan else 'MISSING'}, opsi={sorted(opsi.keys())}")
        # Coba tetap simpan jika ada pertanyaan dan minimal 3 opsi
        if not pertanyaan:
            return None

    # Susun opsi sebagai list [A, B, C, D, E]
    opsi_list = []
    for huruf in 'ABCDE':
        if huruf in opsi:
            opsi_list.append(opsi[huruf])
        else:
            opsi_list.append("")  # placeholder jika opsi hilang

    # Hapus trailing empty opsi
    while opsi_list and not opsi_list[-1]:
        opsi_list.pop()

    # Tentukan kategori berdasarkan nomor soal standar CPNS:
    # TWK: 1-30, TIU: 31-55, TKP: 56-100
    # Tapi ini TO, kita deteksi dari konten atau gunakan default TWK
    # (nanti bisa disesuaikan berdasarkan header bagian)
    kategori = detect_kategori(nomor, pertanyaan, teks_pembahasan)

    obj = {
        "id": nomor,
        "kategori": kategori,
        "pertanyaan": pertanyaan,
        "opsi": opsi_list,
        "kunci_jawaban": kunci,
        "pembahasan": teks_pembahasan if teks_pembahasan else None,
    }

    return obj


def detect_kategori(nomor, pertanyaan, pembahasan):
    """
    Deteksi kategori soal berdasarkan konten dan nomor.
    Standar CAT CPNS: TWK 1-30, TIU 31-55, TKP 56-100
    Jika ada kata kunci TKP/TIU di pembahasan, gunakan itu.
    """
    combined = (pertanyaan + " " + (pembahasan or "")).lower()
    
    # Kata kunci TIU
    tiu_keywords = [
        'analogi', 'silogisme', 'deret', 'angka', 'matematika', 'hitungan',
        'perbandingan', 'pecahan', 'aljabar', 'geometri', 'logika',
        'antonim', 'sinonim', 'x =', 'y =', 'berapa', 'hitung',
        'rata-rata', 'persentase', '%', 'km', 'meter', 'liter',
        'bangun', 'sudut', 'segitiga', 'persegi'
    ]
    
    # Kata kunci TKP
    tkp_keywords = [
        'karakteristik pribadi', 'motivasi kerja', 'pelayanan',
        'integritas', 'profesionalisme', 'jejaring kerja',
        'sosial budaya', 'teknologi informasi', 'anti korupsi',
        'bawahan', 'atasan', 'rekan kerja', 'instansi', 'ASN',
        'PNS', 'pegawai', 'jabatan', 'tugas pokok', 'dinas',
    ]
    
    tiu_score = sum(1 for kw in tiu_keywords if kw in combined)
    tkp_score = sum(1 for kw in tkp_keywords if kw in combined)
    
    if tiu_score >= 2 and tiu_score > tkp_score:
        return "TIU"
    if tkp_score >= 2:
        return "TKP"
    
    # Fallback ke nomor urut (standar CPNS)
    if 1 <= nomor <= 30:
        return "TWK"
    elif 31 <= nomor <= 55:
        return "TIU"
    else:
        return "TKP"


# ============================================================
# Run
# ============================================================

if __name__ == "__main__":
    print("=== Parsing OCR pages ===")
    dataset = parse_all_pages()
    
    print(f"\n=== Hasil ===")
    print(f"Total soal berhasil diparsing: {len(dataset)}")
    
    # Statistik
    from collections import Counter
    cats = Counter(s['kategori'] for s in dataset)
    print(f"Kategori: {dict(cats)}")
    
    has_kunci = sum(1 for s in dataset if s['kunci_jawaban'])
    has_pembahasan = sum(1 for s in dataset if s['pembahasan'])
    has_full_opsi = sum(1 for s in dataset if len(s['opsi']) >= 5)
    print(f"Punya kunci jawaban: {has_kunci}/{len(dataset)}")
    print(f"Punya pembahasan: {has_pembahasan}/{len(dataset)}")
    print(f"Opsi lengkap (≥5): {has_full_opsi}/{len(dataset)}")
    
    # Preview 2 soal pertama
    print("\n=== Preview soal 1 ===")
    if dataset:
        s = dataset[0]
        print(f"  Nomor: {s['id']}, Kategori: {s['kategori']}")
        print(f"  Pertanyaan: {s['pertanyaan'][:100]}...")
        print(f"  Opsi: {s['opsi']}")
        print(f"  Kunci: {s['kunci_jawaban']}")
    
    # Simpan ke JSON
    # Sort by id
    dataset.sort(key=lambda x: x['id'])
    
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        json.dump(dataset, f, ensure_ascii=False, indent=2)
    
    print(f"\n✅ Saved to: {OUTPUT_FILE}")
    print(f"   Total: {len(dataset)} soal")
