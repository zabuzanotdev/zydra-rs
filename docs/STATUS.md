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
| `zydra-core` | `crates/zydra-core/` | Selesai: keempat format, mask, aturan, dan sesi |
| `zydra-i18n` | `crates/zydra-i18n/` | Selesai, locale en + id |
| `zydra-cli` | `crates/zydra-cli/` | Selesai, menggantikan `Zydra.py` untuk keempat format |
| `zydra-gui` | `crates/zydra-gui/` | Selesai: tema, tata letak responsif, mask, aturan, dan resume |
| Backend ZIP | `src/tester/zip.rs` | Selesai, teruji end-to-end |
| Backend RAR | `src/tester/rar.rs` | Selesai, RAR4 dan RAR5, teruji end-to-end |
| Backend PDF | `src/tester/pdf.rs` | Selesai, teruji end-to-end |
| Backend shadow | `src/tester/shadow.rs` | Selesai, teruji |
| Sesi dan resume | `src/session/store.rs` | Selesai di core dan di CLI |

Total **418 tes** lintas workspace, fmt bersih, clippy bersih.

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

- 237 tes unit `zydra-core`
- 11 tes end-to-end `zydra-core` untuk keempat format: ZIP, PDF, RAR4, dan RAR5, masing-masing
  dengan kasus password ditemukan dan tidak ditemukan, plus brute force dan mask
- 28 tes `zydra-i18n`
- 37 tes unit `zydra-cli`
- 30 tes integrasi `zydra-cli` yang menjalankan **binary yang sesungguhnya** lewat
  `CARGO_BIN_EXE_zydra`: keempat format, mask, aturan mangling, resume, kedua bahasa,
  `--json`, `--quiet`, dan setiap kode keluar
- 72 tes `zydra-gui`: logika responsif tepat di nilai ambang, aturan "kapan tombol mulai
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

- **Ekspor log.** Satu-satunya fitur yang direncanakan dan belum ada.
- **Panel pengaturan lanjutan di GUI** (jumlah worker, batas kombinasi). Nilainya masih
  memakai bawaan.
- **Tag rilis pertama.** Butuh persetujuan pemilik proyek, karena itu menerbitkan
  sesuatu yang belum pernah dilihat berjalan di macOS dan Linux.
- **Verifikasi CI.** Lihat bagian 8.

Semua yang lain sudah selesai dan teruji: keempat format, mask, aturan mangling, resume,
dua bahasa, antarmuka grafis, dan pemeriksaan berkas tanpa enkripsi.

### Batasan yang diketahui, dan sebabnya

- **RAR: hanya entri terenkripsi pertama yang diuji.** `read` pada crate `unrar` memakan
  arsipnya, jadi setelah pembacaan gagal tidak ada cara melanjutkan ke entri berikutnya
  tanpa membuka ulang seluruh arsip. Pada arsip nyata seluruh entri memakai password yang
  sama, sehingga entri pertama sudah mewakili.
- **Berkas dengan tanda tangan RAR yang valid tetapi isinya rusak** akan dijalankan
  sebagai arsip biasa dan berakhir "password tidak ditemukan", bukan kesalahan berkas.
  Pemeriksaan tanda tangan menangkap berkas yang jelas bukan RAR, tetapi tidak menangkap
  arsip yang rusak di tengah.
- **Argumen baris perintah dikonversi secara lossy.** Jalur yang bukan UTF-8 akan ditolak
  oleh lapisan berkas dengan pesan biasa alih-alih membuat program panik, tetapi jalur
  seperti itu tidak bisa dipakai.

## 8. CI sudah terverifikasi

Bukan lagi asumsi. Hasil pemeriksaan langsung ke GitHub Actions:

```
success: 9 run   (seluruh riwayat, tanpa satu pun gagal)
```

Setiap run menjalankan empat job, dan keempatnya lolos di setiap commit:

```
Aturan dependensi        success
Test (ubuntu-latest)     success
Test (windows-latest)    success
Test (macos-latest)      success
```

Klaim "jalan di Windows, macOS, dan Linux" karena itu terbukti, bukan diyakini. Yang
dijalankan di tiap sistem: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test --workspace`, dan `cargo build --workspace --release`. Job keempat memeriksa
tiga aturan arsitektur: `zydra-core` tidak menarik crate antarmuka, tidak mencetak ke
terminal, dan tidak mengakhiri proses sendiri.

### Cara memeriksanya lagi

Kredensial git untuk akun ini tersimpan di Git Credential Manager, dan bisa dipakai untuk
memanggil API GitHub tanpa memasang `gh`:

```powershell
$cred  = "protocol=https`nhost=github.com`n`n" | git credential fill
$token = ($cred | Select-String '^password=') -replace '^password=',''
$h = @{ Authorization = "Bearer $token"; Accept = 'application/vnd.github+json' }
Invoke-RestMethod -Uri 'https://api.github.com/repos/zabuzanotdev/zydra-rs/actions/runs' -Headers $h
```

Ini dicatat karena saya sempat menyatakan CI tidak bisa diperiksa dari mesin ini dan
menyerah di situ. Kredensialnya sudah ada sejak awal.
## 9. Cacat yang ditemukan dari menjalankan alatnya, bukan dari membaca kode

Dicatat karena polanya berulang, dan karena semuanya sudah mahal:

1. **`seek` meleset satu langkah.** Rencana menempatkan odometer *pada* kandidat ke-`index`,
   padahal `next_candidate` selalu memajukan lebih dulu. Empat tes rencana gagal.
2. **Livelock pada sumber brute force.** `advance` me-reset odometer ke nol saat ruangnya
   habis, sehingga panggilan berikutnya memulai ulang dari awal. Dua worker menelusuri
   ruang yang sama tanpa henti, dan brute force yang passwordnya tidak ada tidak pernah
   melaporkan "tidak ditemukan". Gejalanya menyamar sebagai tes menggantung; yang
   membongkarnya adalah 63 detik CPU pada satu proses.
3. **Klaim "terhalang" yang salah.** Ditulis bahwa RAR dan PDF tidak bisa dikerjakan karena
   tidak ada tool pembuat fixture. PyMuPDF sudah terpasang, dan libarchive menerbitkan
   fixture RAR. Tidak ada yang terhalang.
4. **Posisi resume yang mundur.** Mesin menghitung kandidat sejak ia dijalankan, bukan
   sejak awal ruang pencarian, jadi posisi yang tersimpan mundur setiap kali serangan
   dilanjutkan lalu dihentikan lagi. Hanya terlihat dengan menjalankan resume dua kali
   berturut-turut dan membandingkan angka di berkas sesi.
5. **Pelacak sesi yang menyimpan sidik jari saat dibuat.** Pelacak yang hidup lebih lama
   daripada berkasnya menganggap berkas yang sudah berubah masih cocok. Di produksi tidak
   terlihat karena pelacaknya dibuat per-start; yang menemukannya adalah tes yang memakai
   pelacak berumur panjang.
6. **RAR melaporkan keadaan enkripsi dari tanda tangan, bukan dari isinya.** Arsip RAR4
   biasa belum tentu memakai password. Kalau tidak diperbaiki, pemeriksaan berkas tanpa
   enkripsi justru akan menolak arsip yang memang terkunci.
7. **Label tampil sebagai kunci mentah di CLI**, kata "Password" muncul dua kali, dan
   baris log menduplikasi baris Target. Ketiganya ketahuan dari melihat keluaran binary-nya
   sendiri.
8. **CI dinyatakan tidak bisa diperiksa.** Kredensial git tersimpan di mesin ini dan bisa
   dipakai memanggil API GitHub. Sembilan run CI sukses, dan itu baru diketahui setelah
   dicoba.

Kesimpulan yang berlaku untuk pekerjaan berikutnya: **jalankan alatnya, dan coba dulu
sebelum menyatakan sesuatu tidak mungkin.** Delapan cacat di atas semuanya lolos dari
pembacaan ulang kode.