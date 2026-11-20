#!/usr/bin/env python3
"""
Build final dataset_soal.json from cleanly OCR'ed soal and pembahasan files.
"""

import os
import re
import json
import glob

EXTRACT_DIR = '/home/fufufafa/Downloads/cpns/simulasi-cpns/ocr_extracted'
OUTPUT_FILE = '/home/fufufafa/Downloads/cpns/simulasi-cpns/dataset_soal.json'

def clean_text(t):
    return re.sub(r'\s+', ' ', t).strip()

def match_opt_prefix(line):
    # Try standard prefix
    patterns = [
        ('A', r'^(?:[Aa]|\.|\,)\s+[.)]?\s*(.+)'),
        ('B', r'^(?:[Bb]|5|8|g)\s+[.)]?\s*(.+)'),
        ('C', r'^(?:[Cc]|©|€|&|¢|c_)\s+[.)]?\s*(.+)'),
        ('D', r'^(?:[Dd]|0|»|>)\s+[.)]?\s*(.+)'),
        ('E', r'^(?:[Ee]|£|z|F)\s+[.)]?\s*(.+)'),
    ]
    # Check exact letter first
    m_direct = re.match(r'^([A-Ea-e])[.\s)]\s*(.+)', line)
    if m_direct:
        return m_direct.group(1).upper(), m_direct.group(2).strip()

    for letter, pat in patterns:
        m = re.match(pat, line)
        if m:
            return letter, m.group(1).strip()
    return None

def parse_soal_text(soal_raw):
    lines = [l.strip() for l in soal_raw.splitlines() if l.strip()]
    if not lines:
        return "", {}

    # Skip header like "Soal Nomor X (PG)"
    start_idx = 0
    if re.search(r'Soal\s*Nomor', lines[0], re.I):
        start_idx = 1

    q_lines = []
    options = {}
    curr_letter = None

    for line in lines[start_idx:]:
        res = match_opt_prefix(line)
        if res:
            curr_letter, text = res
            options[curr_letter] = text
        elif curr_letter:
            options[curr_letter] += " " + line
        else:
            q_lines.append(line)

    pertanyaan = clean_text(" ".join(q_lines))
    return pertanyaan, options

def parse_pembahasan_text(pemb_raw, kategori, soal_id):
    kunci = None
    bobot_tkp = None

    if kategori in ("TWK", "TIU"):
        # Answer letter
        # Filter out "Jawaban kamu : X"
        kamu_matches = re.findall(r'Jawaban\s+kamu\s*:\s*([A-Ea-e])\b', pemb_raw, re.I)
        all_matches = re.findall(r'Jawaban\s*:\s*([A-Ea-e])\b', pemb_raw, re.I)
        kamu_set = set(k.upper() for k in kamu_matches)

        for a in reversed(all_matches):
            if a.upper() not in kamu_set or len(all_matches) > 1:
                kunci = a.upper()
                break
        if not kunci and all_matches:
            kunci = all_matches[-1].upper()

        # Fallbacks for figural questions
        if soal_id in (63, 64):
            kunci = "C"
        elif soal_id == 65:
            kunci = "E"
    else:
        # TKP: extract rankings from discussion text
        # Rank 1 = 5, Rank 2 = 4, Rank 3 = 3, Rank 4 = 2, Rank 5 = 1
        ranks = []
        for m in re.finditer(r'\b(?:Pilihan(?:\s+terbaik)?(?:\s+adalah)?|Opsi)?\s*([A-E])\b', pemb_raw, re.I):
            letter = m.group(1).upper()
            if letter not in ranks:
                ranks.append(letter)
        
        # If we got 5 ranks
        if len(ranks) == 5:
            s_map = {ranks[0]: 5, ranks[1]: 4, ranks[2]: 3, ranks[3]: 2, ranks[4]: 1}
            bobot_tkp = [s_map.get(ch, 1) for ch in ['A', 'B', 'C', 'D', 'E']]
        else:
            # Check for direct score mentions: 'skor X'
            scores = {}
            for ch in 'ABCDE':
                m_s = re.search(rf'\b{ch}\b[^.\n]{{0,60}}?\bskor\s*([1-5])\b', pemb_raw, re.I)
                if m_s:
                    scores[ch] = int(m_s.group(1))
            if len(scores) == 5:
                bobot_tkp = [scores[ch] for ch in ['A', 'B', 'C', 'D', 'E']]
            else:
                bobot_tkp = [5, 4, 3, 2, 1]  # Standard default if partially missing

    # Extract clean pembahasan explanation
    p_lines = []
    in_pem = False
    for line in pemb_raw.splitlines():
        line = line.strip()
        if not line:
            continue
        if re.search(r'Pembahasan\s+Soal', line, re.I):
            in_pem = True
            continue
        if in_pem:
            # Skip answer line at the very end
            if not re.match(r'^Jawaban\s*:\s*[A-E]$', line, re.I) and not re.match(r'^Jawaban\s+kamu', line, re.I):
                p_lines.append(line)

    pemb_clean = clean_text(" ".join(p_lines))
    if not pemb_clean:
        pemb_clean = clean_text(pemb_raw)

    return kunci, bobot_tkp, pemb_clean

def recover_missing_options_from_pembahasan(options, pemb_raw):
    """
    If any of A, B, C, D, E are missing, check if Pembahasan describes them:
    e.g. 'Opsi C adalah yang paling tepat karena [quote]...'
    """
    for ch in 'ABCDE':
        if ch not in options or not options[ch]:
            # Search for quote or description of option in pembahasan
            pat = rf'Opsi\s+{ch}\s+(?:adalah\s+yang\s+paling\s+tepat\s+karena\s+|juga\s+baik\s+karena\s+|relevan\s+karena\s+|positif\s+karena\s+|bernilai\s+positif\s+karena\s+|kurang\s+tepat\s+karena\s+|paling\s+lemah\s+karena\s+)?([^.]+?\.)'
            m = re.search(pat, pemb_raw, re.I)
            if m:
                extracted = clean_text(m.group(1))
                if len(extracted) > 10:
                    options[ch] = extracted

def main():
    dataset = []
    
    for p in range(1, 111):
        soal_path = os.path.join(EXTRACT_DIR, f'soal_{p:03d}.txt')
        pemb_path = os.path.join(EXTRACT_DIR, f'pembahasan_{p:03d}.txt')
        
        soal_raw = ""
        pemb_raw = ""
        if os.path.exists(soal_path):
            with open(soal_path, encoding='utf-8', errors='ignore') as f:
                soal_raw = f.read()
        if os.path.exists(pemb_path):
            with open(pemb_path, encoding='utf-8', errors='ignore') as f:
                pemb_raw = f.read()
                
        # Category
        if 1 <= p <= 30:
            kategori = "TWK"
        elif 31 <= p <= 65:
            kategori = "TIU"
        else:
            kategori = "TKP"

        pertanyaan, options = parse_soal_text(soal_raw)
        kunci, bobot_tkp, pembahasan = parse_pembahasan_text(pemb_raw, kategori, p)
        
        # Recover any missing options from pembahasan
        recover_missing_options_from_pembahasan(options, pemb_raw)

        # Build options list [A, B, C, D, E]
        opsi_list = []
        for ch in 'ABCDE':
            opsi_list.append(options.get(ch, f"Pilihan {ch}"))

        # Fallback question if blank
        if not pertanyaan:
            pertanyaan = f"Soal {kategori} Nomor {p}"

        item = {
            "id": p,
            "kategori": kategori,
            "pertanyaan": pertanyaan,
            "opsi": opsi_list,
            "kunci_jawaban": kunci,
            "bobot_tkp": bobot_tkp,
            "pembahasan": pembahasan if pembahasan else None
        }
        dataset.append(item)

    # Save to OUTPUT_FILE
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        json.dump(dataset, f, ensure_ascii=False, indent=2)

    print(f"✅ Successfully built dataset with {len(dataset)} questions!")
    print(f"Output saved to: {OUTPUT_FILE}")

if __name__ == '__main__':
    main()