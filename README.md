<p align="center">
  <img src="https://img.shields.io/badge/status-dalam%20pengembangan-orange" alt="Status: dalam pengembangan">
  <img src="https://img.shields.io/badge/lisensi-GPL--3.0--or--later-blue" alt="Lisensi: GPL-3.0 atau lebih baru">
  <img src="https://img.shields.io/badge/rust-stable-dea584" alt="Rust stable">
</p>

# Zydra

Alat **pemulihan password berkas terenkripsi** dengan serangan wordlist dan brute force.
Ditulis dalam Rust, berjalan di Windows, macOS, dan Linux.

Ini adalah port ulang dari [Zydra](https://github.com/hamedA2/Zydra) yang aslinya berupa
skrip Python satu berkas. Yang diambil adalah perilakunya, bukan kodenya: seluruh
implementasi ditulis dari nol, dengan arsitektur yang memisahkan logika dari antarmuka.

> **Gunakan hanya untuk berkas milik sendiri** (misalnya lupa password), audit keamanan
> dengan izin tertulis, atau latihan CTF. Jangan memakai alat ini terhadap berkas milik
> orang lain tanpa izin.

---

## Keadaan sekarang

Bagian ini sengaja ditulis apa adanya, karena alat pemulihan password yang mengklaim
mendukung format yang belum bisa dipakai itu lebih buruk daripada tidak ada.

| Kemampuan | Keadaan |
|---|---|
| Serangan wordlist | Berfungsi |
| Serangan brute force (kelas karakter, panjang min/max) | Berfungsi |
| **ZIP** (ZipCrypto dan AES-256) | Berfungsi |
| **Berkas `shadow` Unix** (md5, sha256, sha512, bcrypt, des) | Berfungsi |
| Antarmuka baris perintah | Berfungsi |
| Dua bahasa (Inggris, Indonesia) | Berfungsi |
| **RAR** | **Belum ada.** Perintah akan melaporkan bahwa backend-nya tidak tersedia |
| **PDF** | **Belum ada.** Sama |
| Antarmuka grafis | Belum ada |
| Resume serangan panjang | Belum ada |
| Mask attack dan aturan mangling | Belum ada |
| Rilis siap unduh | Belum ada |

Pengujian: **191 tes otomatis**, dan `cargo clippy --all-targets -- -D warnings` bersih.

---

## Membangun

Persyaratan: Rust stable (edition 2021, `rust-version = "1.85"`).

```bash
git clone https://github.com/zabuzanotdev/zydra-rs
cd zydra-rs
cargo build --release
```

Binary-nya muncul di `target/release/zydra` (Windows: `zydra.exe`).

Menjalankan tes:

```bash
cargo test --workspace
```

---

## Cara pakai

### Wordlist

```bash
zydra -f vault.zip -d words.txt
```

### Brute force

```bash
zydra -f vault.zip -b letters,digits -m 1 -x 4
```

Kelas karakter yang tersedia: `lowercase`, `uppercase`, `digits`, `symbols`, `space`, dan
`letters` (gabungan huruf kecil dan besar). Beberapa kelas bisa digabung dengan koma.

### Berkas `shadow` Unix

```bash
sudo zydra -f /etc/shadow -d words.txt
```

Berjalan juga di Windows, tidak seperti versi Python yang bergantung pada modul `crypt`
yang hanya ada di Unix.

### Pilihan lain

| Pilihan | Keterangan |
|---|---|
| `-f`, `--file` | Berkas target |
| `-d`, `--wordlist` | Berkas wordlist, satu kandidat per baris |
| `-b`, `--charset` | Kelas karakter untuk brute force |
| `-m`, `--min-length` | Panjang minimum |
| `-x`, `--max-length` | Panjang maksimum |
| `--threads` | Jumlah worker, `0` berarti satu per CPU |
| `--lang` | Bahasa antarmuka: `en` atau `id` |
| `--json` | Satu objek JSON per baris, untuk skrip |
| `-q`, `--quiet` | Keluaran polos tanpa warna dan tanpa bilah progres |

### Kode keluar

Berbeda dari versi Python yang selalu mengembalikan `0`, Zydra membedakan hasilnya supaya
bisa dipakai di dalam skrip:

| Kode | Arti |
|---|---|
| `0` | Password ditemukan |
| `1` | Tidak ada kandidat yang cocok |
| `2` | Argumen tidak sah |
| `3` | Berkas target atau backend bermasalah |

Contoh keluaran:

```
Target      vault.zip
Method      wordlist
Candidates  5
Password found
Password  kr4k3n_1987
Elapsed  00:00
```

Dengan `--json`:

```json
{"kind":"started","total":5}
{"kind":"found","password":"kr4k3n_1987","elapsed_seconds":0}
```

---

## Arsitektur

Empat crate dalam satu workspace, dengan aturan dependensi satu arah:

```
zydra-gui ──┐
            ├──> zydra-i18n ──> zydra-core
zydra-cli ──┘
```

| Crate | Tanggung jawab |
|---|---|
| `zydra-core` | Logika pemulihan. Tidak tahu apa pun soal antarmuka, dan tidak memuat satu pun kalimat yang dilihat pengguna |
| `zydra-i18n` | Katalog teks (fluent). Satu-satunya tempat kalimat tinggal |
| `zydra-cli` | Baris perintah |
| `zydra-gui` | Antarmuka grafis (belum ada) |

Konsekuensi dari pemisahan ini:

- Menambah format baru berarti menambah satu berkas di `zydra-core/src/tester/`. Mesin
  paralel tidak perlu diubah.
- Menambah bahasa berarti menambah satu berkas `.ftl`. Kode tidak perlu diubah.
- Logika bisa diuji tanpa antarmuka, dan memang begitu: 123 dari 191 tes tidak menyentuh
  lapisan teks sama sekali.

### Mengapa arsip tidak diekstrak ke disk

Versi Python memanggil `extractall` untuk setiap kandidat password, membuat direktori
sementara berisi salinan arsip, lalu membersihkannya. Zydra Rust tidak menulis berkas sama
sekali: kandidat diuji dengan membaca entri di dalam memori.

Satu hal penting soal ZIP: ZipCrypto hanya menyediakan 1 dari 256 bit untuk memvalidasi
password, sehingga **password yang salah bisa diterima** saat header dibuka. Karena itu
verifikasi wajib membaca entri sampai habis dan bersandar pada CRC32. Memeriksa beberapa
kilobyte pertama tidak membuktikan apa pun. AES-256 tidak punya masalah ini karena punya
verifikasi HMAC.

---

## Kontribusi

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

Ketiganya harus bersih. Aturan yang tidak boleh dilanggar ada di `docs/STATUS.md`.

---

## Lisensi dan kredit

- Port Rust ini: **GPL-3.0-or-later**, Copyright (C) 2026 zabuzanotdev. Teks lengkap di
  [`LICENSE`](LICENSE).
- Proyek asli: [hamedA2/Zydra](https://github.com/hamedA2/Zydra) (Python, MIT,
  Copyright (c) 2019 hamedA2). Teks MIT dipertahankan di
  [`LICENSE-MIT-Zydra.txt`](LICENSE-MIT-Zydra.txt). MIT kompatibel dengan GPL-3.0.
- Binary hasil build memuat **libunrar** dari RARLAB, yang lisensinya mengizinkan
  ekstraksi tetapi melarang pembuatan kompresor RAR. Rinciannya di [`NOTICE`](NOTICE).

---

# Zydra (English)

A **password recovery tool** for encrypted files, using wordlist and brute-force attacks.
Written in Rust, running on Windows, macOS, and Linux.

A ground-up Rust rewrite of [Zydra](https://github.com/hamedA2/Zydra), originally a
single-file Python script. The behaviour was ported; the implementation was written from
scratch, with the logic separated from every user interface.

> **Use it only on files you own** — for example when you forgot your own password — on
> security audits you have written permission for, or in CTF practice. Do not point this
> at someone else's files without permission.

## Current state

Stated plainly, because a recovery tool that claims formats it cannot actually open is
worse than one that admits the gap.

| Capability | State |
|---|---|
| Wordlist attack | Works |
| Brute-force attack (character classes, min/max length) | Works |
| **ZIP** (ZipCrypto and AES-256) | Works |
| **Unix `shadow` files** (md5, sha256, sha512, bcrypt, des) | Works |
| Command-line interface | Works |
| Two languages (English, Indonesian) | Works |
| **RAR** | **Not implemented.** Reports that the backend is unavailable |
| **PDF** | **Not implemented.** Same |
| Graphical interface | Not started |
| Resuming long attacks | Not started |
| Mask attacks and mangling rules | Not started |
| Downloadable releases | Not started |

Testing: **191 automated tests**, with `cargo clippy --all-targets -- -D warnings` clean.

## Usage

```bash
# wordlist
zydra -f vault.zip -d words.txt

# brute force
zydra -f vault.zip -b letters,digits -m 1 -x 4

# machine readable
zydra -f vault.zip -d words.txt --json

# Indonesian interface
zydra -f vault.zip -d words.txt --lang id
```

Character classes: `lowercase`, `uppercase`, `digits`, `symbols`, `space`, and `letters`
(both cases). Combine them with commas.

Exit codes: `0` password found, `1` no candidate matched, `2` bad arguments, `3` target
file or backend problem. The original Python script always returned `0`, which made it
unusable inside scripts.

## Architecture

Four crates in one workspace, with a strictly one-way dependency rule:

```
zydra-gui ──┐
            ├──> zydra-i18n ──> zydra-core
zydra-cli ──┘
```

`zydra-core` holds the recovery logic and knows nothing about any interface — it contains
no sentence a user will ever read. `zydra-i18n` is the only place text lives. Adding a
format means adding one file under `zydra-core/src/tester/`; adding a language means
adding one `.ftl` file.

## Licence and credits

- Rust port: **GPL-3.0-or-later**, Copyright (C) 2026 zabuzanotdev. See [`LICENSE`](LICENSE).
- Original project: [hamedA2/Zydra](https://github.com/hamedA2/Zydra) (Python, MIT,
  Copyright (c) 2019 hamedA2). The MIT text is kept in
  [`LICENSE-MIT-Zydra.txt`](LICENSE-MIT-Zydra.txt); MIT is compatible with GPL-3.0.
- Built binaries include **libunrar** from RARLAB, whose licence permits extraction but
  forbids building a RAR compressor. Details in [`NOTICE`](NOTICE).
