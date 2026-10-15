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
| Spec desain | `docs/superpowers/specs/2026-10-15-zydra-rust-design.md` | Disetujui, dikoreksi 7 kali |
| Plan A1 | `docs/superpowers/plans/2026-10-15-zydra-core-engine.md` | 9 tugas, selesai semua |
| `zydra-core` | `crates/zydra-core/` | 74 tes, fmt bersih, clippy bersih |

Isi `zydra-core` yang sudah ada:

```
src/
├── lib.rs            deklarasi modul
├── constants.rs      semua angka ajaib
├── error.rs          CoreError (kesalahan prasarana)
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
│   └── mod.rs        trait PasswordTester, TesterFactory, TesterInfo, MatchOutcome
└── engine/
    ├── cancel.rs     CancelToken
    ├── stats.rs      SpeedMeter (rata-rata bergerak)
    └── crack.rs      EngineConfig, EngineEvent, run()
```

Yang **belum ada sama sekali**: `attack`, `log`, tester konkret (zip/rar/pdf/shadow),
`factory`, crate `zydra-i18n`, crate `zydra-cli`, crate `zydra-gui`, CI, README,
`LICENSE`, `NOTICE`.

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

**Plan A2 — menyelesaikan `zydra-core`.** Perkiraan 7 tugas:

1. `attack` — `AttackPlan` (Dictionary | BruteForce), `AttackLimits`, `validate`,
   `total`, `build_source`. Ini sambungan bersama yang dipakai CLI dan GUI supaya
   validasi tidak ditulis dua kali.
2. `log` — `LogLevel`, `LogEntry` (menyimpan key i18n + argumen, bukan kalimat jadi),
   `LogBuffer` berupa cincin berbatas.
3. `tester::zip` + tes integrasi (tes membuat arsip terenkripsi sendiri, ZipCrypto dan
   AES-256, lalu memecahkannya — tidak perlu berkas biner di repositori).
4. `tester::shadow` + tes (berkas shadow sintetis dengan hash yang dihitung `pwhash`).
5. `tester::rar` + tes (fixture 7-Zip, atau `#[ignore]` dengan alasan jujur).
6. `tester::pdf` + fixture kecil terenkripsi yang di-commit.
7. `tester::factory` + tes end-to-end lewat `engine::run` pada berkas ZIP nyata.

**Plan A3 — frontend terminal.** `zydra-i18n` (fluent, locale `en` + `id`, disematkan
dengan `include_str!`) dan `zydra-cli` (clap, `indicatif`, kode keluar 0/1/2/3, `--json`),
plus README, `LICENSE` GPL-3.0, dan `NOTICE`. Setelah A3, CLI sudah menggantikan
`Zydra.py` untuk semua format.

**Plan B — GUI.** `zydra-gui` dengan egui: tema Cyber Industrial, layout Sidebar +
Workspace yang responsif (breakpoint di 1100 / 720 / 420 px), widget, controller.

**Plan C — fitur lanjutan + rilis.** `session` (resume), `config`, mask, rules, panel
settings, ekspor log, CI 3 OS, dan rilis otomatis saat tag.

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
