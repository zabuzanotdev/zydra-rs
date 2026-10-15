# Status & Handoff

Terakhir diperbarui: 2026-10-15

Repositori: https://github.com/zabuzanotdev/zydra-rs (privat, akun gh 1 `zabuzanotdev`)
Lokal: `D:\zydra-rs`
Proyek asal: `D:\zydra\Zydra.py` (Python, 686 baris) — belum disentuh, masih utuh

Berkas ini menjelaskan keadaan proyek, apa yang sudah terverifikasi, dan apa yang belum.
Baca ini dulu sebelum melanjutkan, bersama
`docs/superpowers/specs/2026-10-15-zydra-rust-design.md` (spec, 17 bagian, disetujui).

---

## 1. Keadaan sekarang

Yang **sudah selesai dan terverifikasi**:

| Lapisan | Berkas | Status |
|---|---|---|
| Spec desain | `docs/superpowers/specs/2026-10-15-zydra-rust-design.md` | Disetujui, dikoreksi 11 kali |
| Plan A1 | `docs/superpowers/plans/2026-10-15-zydra-core-engine.md` | 9 tugas, selesai semua |
| `zydra-core` | `crates/zydra-core/` | Selesai untuk ZIP dan shadow |
| `zydra-i18n` | `crates/zydra-i18n/` | Selesai, locale en + id |
| `zydra-cli` | `crates/zydra-cli/` | Selesai, menggantikan `Zydra.py` untuk ZIP dan shadow |
| `zydra-gui` | `crates/zydra-gui/` | Selesai untuk tata letak Sidebar + Workspace yang responsif |
| Backend ZIP | `src/tester/zip.rs` | Selesai, teruji end-to-end |
| Backend shadow | `src/tester/shadow.rs` | Selesai, teruji |
| Backend RAR | — | **Belum ada.** Factory mengembalikan `BackendUnavailable` |
| Backend PDF | — | **Belum ada.** Factory mengembalikan `BackendUnavailable` |

Total **248 tes** lintas workspace, fmt bersih, clippy bersih.

Isi `zydra-core` yang sudah ada:

```
src/
├── lib.rs            deklarasi modul
├── constants.rs      semua angka ajaib
├── error.rs          CoreError (kesalahan prasarana)
├── attack.rs         AttackPlan, AttackLimits, PreparedAttack
├── log.rs            LogLevel, LogEntry, LogBuffer (cincin berbatas)
├── model/
│   ├── charset.rs    CharsetSet (bitflag), alphabet, total_combinations, parse
│   ├── target.rs     TargetKind, CrackTarget, deteksi jenis berkas
│   ├── failure.rs    CrackFailure, Backend
│   └── progress.rs   TryingData, CrackProgress
├── candidate/
│   ├── mod.rs        trait CandidateSource
│   ├── wordlist.rs   WordlistSource
│   └── bruteforce.rs BruteForceSource
├── tester/
│   ├── mod.rs        trait PasswordTester, TesterFactory, TesterInfo, MatchOutcome
│   ├── factory.rs    DefaultTesterFactory
│   ├── zip.rs        ZipTester (ZipCrypto + AES-256)
│   └── shadow.rs     ShadowTester (md5/sha256/sha512/bcrypt/des)
└── engine/
    ├── cancel.rs     CancelToken
    ├── stats.rs      SpeedMeter (rata-rata bergerak)
    └── crack.rs      EngineConfig, EngineEvent, run()
```

Yang **belum ada sama sekali**: `tester/rar.rs`, `tester/pdf.rs`, `model/mask.rs`,
`model/rules.rs`, `candidate/mask.rs`, `candidate/rules.rs`, `session`, `config`,
crate `zydra-gui`, CI, README, `LICENSE`, `NOTICE`.

### Kemampuan yang sudah bisa dibuktikan sekarang

`cargo test --workspace` menjalankan 191 tes:

- 119 tes unit `zydra-core`
- 4 tes end-to-end `zydra-core` yang membuat arsip ZIP AES-256 sendiri lalu
  memecahkannya lewat jalur lengkap `AttackPlan` → `DefaultTesterFactory` → `ZipTester`
  → `engine::run`
- 23 tes `zydra-i18n`
- 28 tes unit `zydra-cli`
- 17 tes integrasi `zydra-cli` yang menjalankan **binary yang sesungguhnya** lewat
  `CARGO_BIN_EXE_zydra`: kedua mode serangan, kedua bahasa, `--json`, `--quiet`, dan setiap
  kode keluar
- 52 tes `zydra-gui`: logika responsif tepat di nilai ambang, aturan "kapan tombol mulai
  boleh ditekan", dan pengendali yang menjalankan serangan sungguhan sampai selesai

Perintah yang sudah setara dengan `Zydra.py`:

```powershell
# wordlist
.\target\debug\zydra.exe -f vault.zip -d words.txt

# brute force
.\target\debug\zydra.exe -f vault.zip -b letters,digits -m 1 -x 4

# shadow Unix
.\target\debug\zydra.exe -f shadow -d words.txt

# bahasa Indonesia, keluaran terstruktur
.\target\debug\zydra.exe -f vault.zip -d words.txt --lang id --json
```

Kode keluar: `0` ditemukan, `1` tidak ditemukan, `2` argumen salah, `3` berkas atau backend
bermasalah. Versi Python selalu memakai `exit(0)` sehingga tidak bisa dipakai di skrip.

## 2. Cara menjalankan

```powershell
cd D:\zydra-rs
cargo test --workspace          # 74 tes, harus lolos
cargo fmt --check               # harus tanpa keluaran
cargo clippy --all-targets -- -D warnings   # harus bersih
```

## 3. Jebakan lingkungan di mesin ini (WAJIB dibaca)

1. **JANGAN membuat `rust-toolchain.toml`.** `rustup` di mesin ini diinstal dengan host
   triple `x86_64-pc-windows-gnu`, jadi `channel = "stable"` menyalin ke toolchain GNU
   yang butuh `dlltool.exe` dan **tidak ada** di sini — build langsung gagal di
   `windows-sys`. Menulis `channel = "stable-x86_64-pc-windows-msvc"` memang jalan di
   mesin ini tetapi merusak macOS dan Linux. Solusinya: tidak ada berkas itu, dan
   toolchain di-pin di CI per OS. Toolchain yang benar di mesin ini adalah
   `stable-x86_64-pc-windows-msvc` (default rustup).

2. **Shell adalah PowerShell 5.1, bukan bash.** Tidak ada heredoc (`<<'EOF'`), `head`,
   `tail`, `grep`. Pakai `Select-Object -First/-Last` dan `Select-String`. Untuk commit
   message multi-baris: tulis ke berkas, lalu `git commit -F <berkas>`.

3. **`Exited with code 1` sering bukan kegagalan.** PowerShell membungkus stderr program
   native (cargo, git) sebagai `NativeCommandError`. Yang menentukan adalah baris
   `test result:` dan `Finished`. Jangan "memperbaiki" yang tidak rusak.

4. **`Select-String` menahan seluruh keluaran sampai perintah selesai.** Kalau sebuah
   perintah mungkin menggantung, jalankan dengan `Tee-Object` ke berkas dan baca
   berkasnya, jangan `Select-String`.

5. **Jangan menjalankan `cargo` bersamaan dengan subagent yang juga menjalankan `cargo`.**
   Keduanya berebut kunci direktori build dan perintah Anda akan menunggu tanpa keluaran
   sama sekali (bukan gagal, hanya diam) sampai timeout. Kalau sebuah perintah cargo
   "menggantung" tanpa keluaran, periksa dulu
   `Get-Process -Name cargo,rustc` sebelum menyimpulkan ada deadlock di kode.

6. **`rust-version` yang terlalu rendah diam-diam menurunkan versi dependensi.**
   `rust-version = "1.75"` membuat `cargo add zip` memilih zip 4.6.1 alih-alih 9.0.3,
   sehingga API yang sudah diverifikasi dari dokumentasi versi terbaru tidak berlaku lagi.
   Nilai sekarang `1.85`, disamakan di `Cargo.toml` dan `clippy.toml`. Kalau menambah
   dependensi baru dan versinya terasa aneh, periksa dua hal ini lebih dulu.

7. **Mesin ini tidak punya tool arsip/PDF pihak ketiga.** Tidak ada `7z`, `7za`, `7zr`,
   `rar`, `unrar`, maupun `qpdf`. Konsekuensinya langsung: fixture RAR dan PDF terenkripsi
   tidak bisa dibuat di sini. Lihat bagian 7.

### Jebakan pustaka yang sudah terbayar

- **clap**: arg bantuan dan versi bawaan (`-h`, `-V`) baru ada **setelah**
  `Command::build()` dipanggil. Memanggil `mut_arg("help", ..)` sebelum itu panik dengan
  "Argument `help` is undefined". `build()` menerima `&mut self` dan mengembalikan `()`.
- **clap**: bahasa harus diketahui sebelum perintah dibangun, karena teks bantuan disusun
  saat itu. Karena itu `--lang` disisir dari argumen mentah lebih dulu.
- **clap**: kata bawaan "Usage:", "Options:", "Print help", "Print version" semuanya
  terlihat pengguna. Harus diambil alih lewat `help_template` dan `mut_arg`.
- **owo-colors**: `if_supports_color` dan `Stream` butuh fitur `supports-colors`. Dan
  merangkai dua gaya (`text.bold().green()`) tidak bisa dikompilasi karena meminjam nilai
  antara yang dibuat di dalam closure. Satu gaya per panggilan.
- **fluent-bundle**: `set_use_isolating(false)` wajib dipanggil. Tanpa itu nilai sisipan
  dibungkus tanda isolasi Unicode yang tidak terlihat di terminal tetapi ikut tersalin.
- **`zip`**: `FileOptions::default()` tidak bisa menyimpulkan tipe ekstensinya; pakai alias
  `SimpleFileOptions`.

## 4. Pelajaran dari Plan A1

**Kesalahan di dokumen rencana lebih mahal daripada kesalahan di kode.** Dua bug nyata
di Plan A1 keduanya berasal dari rencana, bukan dari implementasi:

1. **`seek` meleset satu langkah.** Rencana menempatkan odometer *pada* kandidat ke-`index`,
   padahal `next_candidate` selalu memanggil `advance()` lebih dulu, sehingga hasilnya
   kandidat `index + 1`. Perbaikannya: posisikan pada pendahulu (`target - 1`), dan jepit
   `target` ke `total` supaya melompat melewati akhir menghasilkan `None`.

2. **Livelock karena kehabisan tidak lengket.** `BruteForceSource::advance()` me-reset
   odometer ke nol sebelum mengembalikan `false`, sehingga panggilan `next_candidate`
   berikutnya memulai ulang dari awal. Dengan dua worker, satu berhenti di ujung sementara
   yang lain menelusuri ulang seluruh ruang — berputar tanpa henti, dan brute force yang
   passwordnya tidak ada tidak pernah melaporkan "tidak ditemukan". Perbaikannya: flag
   `done` yang membuat kehabisan permanen, dan kontrak itu sekarang ditulis eksplisit di
   trait `CandidateSource`.

**Konsekuensi untuk penulis rencana berikutnya:**

- Verifikasi API dari dokumentasi resmi sebelum menulis kode di rencana. Rencana A1 sudah
  memuat kesalahan desain `zip_verify_bytes` yang hanya ketahuan setelah membaca docs.rs.
- Untuk sumber kandidat, tes kehabisan harus memanggil `next_candidate` **berulang kali**
  setelah `None`, bukan sekali. Tes A1 semula hanya sekali, sehingga bug tidak tertangkap.
- Setelah memperbaiki bug di kode, **perbaiki juga dokumen rencananya**, atau rencana itu
  akan mewariskan bug yang sama ke eksekusi berikutnya.

## 5. Cara kerja yang terbukti bermasalah

Pendekatan subagent tanpa isolasi **tidak aman di repo ini**:

- Subagent pertama tetap hidup setelah melapor selesai, lalu menulis ulang
  `rust-toolchain.toml` dan mengembalikan `lib.rs` ke keadaan setengah jadi tepat setelah
  commit dibuat. Harus dihentikan lewat melanjutkan sesinya dengan instruksi berhenti.
- Subagent ketiga selesai tanpa laporan apa pun dan meninggalkan proses `cargo` yang
  memegang kunci build.

**Kalau memakai subagent lagi: beri tiap subagent worktree git sendiri.** Instruksi wajib
di prompt-nya: semua perintah di foreground (jangan background), jangan jalankan `rustup`,
jangan buat `rust-toolchain.toml`, commit setelah setiap tugas, jangan tulis ulang riwayat
git. Dan verifikasi sendiri hasilnya — jangan percaya laporan.

## 6. API yang sudah diverifikasi (hemat riset di sesi berikutnya)

Diambil dari docs.rs, bukan dari ingatan. Versi terbaru per 2026-10-15.

**`zip` 9.0.3** — fitur `aes-crypto` untuk AES.

```rust
let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;
let mut entry = archive.by_index_decrypt(0, password.as_bytes())?;
std::io::copy(&mut entry, &mut std::io::sink())?;   // baca sampai habis
```

Peringatan resmi dari dokumentasi crate: ZipCrypto hanya memvalidasi 1 dari 256 bit,
sehingga **password salah bisa diterima** saat membuka header. Karena itu verifikasi wajib
membaca entri sampai habis dan bersandar pada CRC32. AES-256 tidak punya masalah ini
karena punya HMAC. `ZipArchive::get_aes_verification_key_and_salt` bisa dipakai untuk
mengisi `TesterInfo::encryption`.

**`unrar` 0.5.8** — libunrar dibundel, dikompilasi dari sumber (butuh compiler C++).

```rust
use unrar::Archive;
let archive = Archive::with_password(&path, password.as_bytes());
let open = archive.open_for_processing()?;          // CursorBeforeHeader
let open = open.read_header()?.expect("arsip kosong"); // CursorBeforeFile
let (data, _rest) = open.read()?;                    // baca payload ke memori
```

Crate ini **hanya bisa ekstrak dan daftar, tidak bisa membuat** arsip. Fixture RAR harus
dibuat dengan tool lain (7-Zip atau `rar` CLI). Kalau tidak tersedia, tandai tesnya
`#[ignore]` dengan alasan jujur dan nyatakan di README bahwa RAR belum tercakup tes
otomatis — jangan mengklaim "teruji".

**`pwhash` 1.0.0**

```rust
pwhash::unix::verify(password, stored_hash)   // mengenali sendiri format modular
```

Mendukung `$1$` md5crypt, `$5$` sha256crypt, `$6$` sha512crypt, `$2y$` bcrypt, descrypt,
bsdi_crypt. Berjalan di Windows juga — ini perbaikan nyata atas versi Python yang memakai
modul `crypt` Unix-only.

**`lopdf`**

```rust
let doc = lopdf::Document::load_with_password(path, password)?;
if !doc.was_encrypted() || doc.encryption_state.is_some() { /* cocok */ }
```

Ada juga `doc.is_encrypted()`, `doc.decrypt(password)`, `doc.authenticate_owner_password`,
dan `LoadOptions { password: Option<String>, .. }`. Semantik tepatnya (apakah password
salah menghasilkan `Err` atau `Ok` dengan `encryption_state == None`) harus dipastikan
lewat fixture tes, bukan diasumsikan.

## 7. Yang belum dikerjakan

Urutan yang direncanakan. Setiap rencana menghasilkan software yang bisa dipakai sendiri.

**Plan A2 — menyelesaikan `zydra-core`.**

Sudah selesai (dikerjakan langsung, bukan lewat subagent, karena sesi subagent terbukti
bermasalah di repo ini):

1. `attack` — `AttackPlan`, `AttackLimits`, `PreparedAttack::prepare`. Satu tempat yang
   menerjemahkan pilihan pengguna menjadi sumber kandidat, supaya CLI dan GUI tidak
   menulis aturan validasi dua kali.
2. `log` — `LogLevel`, `LogEntry` (kunci i18n + argumen), `LogBuffer` cincin berbatas.
3. `tester::zip` — ZipCrypto dan AES-256, membaca entri sampai habis.
4. `tester::shadow` — lewat `pwhash::unix::verify`.
5. `tester::factory` — Zip dan Shadow tersambung; Rar dan Pdf mengembalikan
   `BackendUnavailable` dengan sebab `backend-not-built`.
6. Tes end-to-end `tests/zip_end_to_end.rs` — membuat arsip terenkripsi sendiri lalu
   memecahkannya lewat jalur lengkap.

Sisa Plan A2:

7. `tester::rar` — API sudah diverifikasi (lihat bagian 6). **Terhalang**: mesin ini tidak
   punya 7-Zip, `rar`, maupun `unrar`, jadi fixture RAR tidak bisa dibuat di sini. Pilihan:
   unduh satu fixture RAR terenkripsi kecil dari sumber publik lalu commit, atau tandai
   tesnya `#[ignore]` dengan alasan jujur dan nyatakan di README bahwa RAR belum tercakup
   tes otomatis.
8. `tester::pdf` — API sudah diverifikasi. **Terhalang hal yang sama**: tidak ada `qpdf`
   dan `lopdf` tidak bisa menulis PDF terenkripsi, jadi fixture PDF terenkripsi kecil harus
   dibuat sekali di tempat lain lalu di-commit. Semantik `load_with_password` untuk password
   salah harus dipastikan lewat fixture itu, jangan diasumsikan.

**Plan A3 — frontend terminal.** Selesai. `zydra-i18n` (fluent, locale `en` + `id`,
disematkan dengan `include_str!`) dan `zydra-cli` (clap, `indicatif`, `owo-colors`, kode
keluar 0/1/2/3, `--json`, `--quiet`).

Sisa yang belum dikerjakan, berurutan:

1. **README, `LICENSE` GPL-3.0, `NOTICE`.** Belum ada sama sekali. `NOTICE` wajib memuat
   atribusi MIT `hamedA2/Zydra` dan catatan lisensi UnRAR sebelum rilis pertama.
2. `tester::rar` — API sudah diverifikasi (bagian 6). **Terhalang**: mesin ini tidak punya
   7-Zip, `rar`, maupun `unrar`, jadi fixture RAR tidak bisa dibuat di sini.
3. `tester::pdf` — API sudah diverifikasi. Terhalang hal yang sama.

**Plan B — GUI.** Selesai untuk irisan pertama: `zydra-gui` dengan egui, tema Cyber
Industrial, tata letak Sidebar + Workspace yang responsif, pengendali yang menghubungkan
antarmuka ke mesin, konsol aktivitas, spanduk hasil dengan tombol salin, dan penggantian
bahasa dari dalam jendela.

Yang belum ada di GUI:

- Panel pengaturan lanjutan (jumlah worker, batas kombinasi, byte verifikasi ZIP).
- Panel untuk memilih dan memuat sesi yang tersimpan (menunggu `session` di Plan C).
- Ekspor log dari antarmuka (menunggu ekspor log di Plan C).

**Plan C — fitur lanjutan + rilis.** `session` (resume), `config`, mask, rules, panel
settings, ekspor log, CI 3 OS, dan rilis otomatis saat tag.

### Jebakan yang sudah terbayar di GUI

- **`rust-version` kembali memilih versi lama.** `cargo add eframe` mengambil 0.32.3
  padahal yang tersedia 0.36.2. Tetap dipakai 0.32.3 supaya tidak mengejar MSRV lagi.
  API egui 0.32 sudah diverifikasi langsung dari sumber crate-nya.
- **`Stroke::new(1.0, ..)` memicu peringatan `float_literal_f32_fallback`** yang akan
  menjadi kesalahan di rilis Rust berikutnya. Di tema dipakai konstanta `HAIRLINE`;
  di widget dipakai sufiks `1.0_f32`.
- **`cargo add <crate>` untuk tes tidak cukup.** Tes integrasi tidak bisa memakai
  dependensi biasa, jadi `zip`, `tempfile`, dan `crossbeam-channel` juga harus ada di
  `[dev-dependencies]` pada crate yang mengujinya.

Catatan: `TesterInfo.encryption` dan `CrackFailure` sudah memuat varian untuk RAR dan PDF,
dan `Backend` sudah punya `Rar` dan `Pdf`. Jadi menambahkan kedua backend itu tidak perlu
mengubah tipe yang sudah ada.

## 8. Aturan yang tidak boleh dilanggar

Diambil dari bagian Global Constraints spec. Aturan ini yang membuat proyek tetap modular.

- `zydra-core` **tidak boleh** bergantung pada crate UI apa pun.
- `zydra-core` **tidak boleh** memuat kalimat yang dilihat pengguna. Tidak ada `println!`
  atau string pesan berbahasa manusia di dalamnya.
- `zydra-core` **tidak boleh** memanggil `std::process::exit`.
- Kesalahan masukan pengguna memakai `CrackFailure`. Kesalahan prasarana memakai
  `CoreError`. Tidak boleh tumpang tindih.
- Semua angka ajaib hidup di `constants.rs`.
- Akhir baris LF di repositori (diatur `.gitattributes`).
- Lisensi `GPL-3.0-or-later`; atribusi MIT `hamedA2/Zydra` dan catatan lisensi UnRAR
  wajib ada di `NOTICE` saat Plan A3.

## 9. Konvensi commit

Baris pertama: `type(scope): ringkasan` yang informatif dan searchable, sekaligus witty —
dan **tiap commit jokesnya berbeda**, jangan template. Body 2–3 baris berisi "kenapa"-nya
secara teknis. Cerita panjang masuk ke berkas ini, bukan ke commit message.

Identitas commit: `zabuzanotdev <290258795+zabuzanotdev@users.noreply.github.com>`
(sudah menjadi default global di mesin ini). Verifikasi dengan
`git log -1 --format="%an <%ae>"` setelah commit.
