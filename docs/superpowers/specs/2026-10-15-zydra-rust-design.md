# Zydra Rust — Design Spec

Tanggal: 2026-10-15
Status: disetujui, siap masuk rencana implementasi
Repositori: `zabuzanotdev/zydra-rs` (privat, akun GitHub gh 1)
Lokasi lokal: `D:\zydra-rs`

---

## 1. Ringkasan

Port ulang total `Zydra.py` (686 baris, satu kelas `Zydra` yang menangani seluruh aplikasi) menjadi
program Rust dengan tiga frontend di atas satu pustaka inti:

- `zydra-core` — logika pemulihan password. Tidak tahu apa pun soal UI.
- `zydra-cli` — pengganti `Zydra.py` untuk pemakaian terminal dan skrip.
- `zydra-gui` — aplikasi desktop bertema "Cyber Industrial" yang jalan di Windows, macOS, dan Linux.

Tujuan mutu yang diminta secara eksplisit:

1. **Tidak ada god object.** Setiap unit punya satu tanggung jawab dan batas antarmuka yang jelas.
2. **Tidak ada kalimat hardcode.** Semua teks yang dilihat pengguna berasal dari katalog terjemahan.
3. **Modular dan mudah diperbaiki.** Menambah format, sumber kandidat, atau bahasa hanya menambah satu
   berkas, bukan menyunting logika yang sudah ada.
4. **UI cantik, lintas platform, responsif di segala ukuran jendela.**

## 2. Non-tujuan

- Bukan alat untuk menyerang file milik orang lain. Cakupan pemakaian ada di README: file sendiri,
  audit dengan izin, dan latihan CTF.
- Bukan implementasi GPU/OpenCL. Semua pekerjaan di CPU.
- Bukan pengganti hashcat. Tidak ada dukungan puluhan algoritma hash, tidak ada aturan gaya hashcat
  lengkap; hanya subset yang dinyatakan di bagian 13.
- Tidak menyentuh proyek Android yang sudah ada di repo `zydra`. Repo ini berdiri sendiri.

## 3. Keputusan yang sudah dikunci

| # | Keputusan | Pilihan |
|---|---|---|
| 1 | Kerangka UI | `eframe` / `egui` |
| 2 | Cakupan frontend | Pustaka inti + GUI + CLI |
| 3 | Backend RAR | crate `unrar` (libunrar dibundel), RAR4 + RAR5 |
| 4 | Pengelolaan teks | `fluent`, locale `en` dan `id` |
| 5 | Lokasi proyek | Repo baru terpisah `zabuzanotdev/zydra-rs` |
| 6 | Arah visual | Cyber Industrial (dark slate + aksen cyan/hijau) |
| 7 | Susunan layar | Sidebar + Workspace, wajib responsif |
| 8 | Fitur | Paritas penuh + salin/buka hasil, resume, pengaturan lanjutan, mask + rules, ekspor log |
| 9 | Distribusi | CI uji 3 OS + rilis otomatis saat push tag |
| 10 | Lisensi | GPL-3.0-or-later, atribusi MIT asli di `NOTICE` |

## 4. Arsitektur dan aturan dependensi

Empat crate dalam satu workspace Cargo. Arah dependensi hanya satu arah:

```
zydra-gui ──┐
            ├──> zydra-i18n ──> zydra-core
zydra-cli ──┘
```

Aturan yang dijaga:

- `zydra-core` tidak boleh bergantung pada `egui`, `clap`, `indicatif`, atau crate UI apa pun.
- `zydra-core` tidak boleh memuat kalimat yang dilihat pengguna. Kesalahan dikembalikan sebagai enum
  bertipe; penerjemahan terjadi di `zydra-i18n`.
- `zydra-core` tidak boleh memanggil `std::process::exit`.
- Hanya `zydra-i18n` yang memuat berkas locale.

Aturan ini ditegakkan di CI lewat pemeriksaan `cargo tree` pada `zydra-core`: kalau ada crate UI muncul
di pohon dependensinya, build gagal.

## 5. Struktur direktori

```
zydra-rs/
├── Cargo.toml                       # [workspace] — daftar anggota saja
├── rust-toolchain.toml              # channel stable + komponen fmt, clippy
├── rustfmt.toml
├── clippy.toml
├── .gitignore
├── LICENSE                          # teks kanonik GPL-3.0
├── NOTICE                           # atribusi MIT hamedA2/Zydra + catatan lisensi UnRAR
├── README.md                        # ringkasan ID + EN
├── docs/superpowers/specs/          # dokumen ini dan rencana implementasi
├── .github/workflows/
│   ├── ci.yml
│   └── release.yml
└── crates/
    ├── zydra-core/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs               # re-export publik; tanpa logika
    │       ├── constants.rs         # semua angka ajaib
    │       ├── error.rs             # CoreError
    │       ├── model/
    │       │   ├── mod.rs
    │       │   ├── target.rs        # CrackTarget, TargetKind, deteksi jenis
    │       │   ├── failure.rs       # CrackFailure
    │       │   ├── progress.rs      # CrackProgress
    │       │   ├── charset.rs       # CharsetKey, CharsetSet, total_combinations
    │       │   ├── mask.rs          # MaskPattern
    │       │   └── rules.rs         # Rule, RuleSet
    │       ├── candidate/
    │       │   ├── mod.rs           # trait CandidateSource
    │       │   ├── wordlist.rs      # pembaca wordlist bertahap
    │       │   ├── bruteforce.rs    # generator odometer
    │       │   ├── mask.rs          # generator berbasis mask
    │       │   └── rules.rs         # pembungkus aturan mangling
    │       ├── tester/
    │       │   ├── mod.rs           # trait PasswordTester, TesterFactory, TesterInfo
    │       │   ├── factory.rs       # DefaultTesterFactory
    │       │   ├── zip.rs
    │       │   ├── rar.rs
    │       │   ├── pdf.rs
    │       │   └── shadow.rs
    │       ├── engine/
    │       │   ├── mod.rs
    │       │   ├── crack.rs         # orkestrasi worker
    │       │   ├── stats.rs         # kecepatan, ETA
    │       │   └── cancel.rs        # token pembatalan
    │       ├── session/
    │       │   ├── mod.rs
    │       │   └── store.rs         # simpan/muat progres resume
    │       ├── config/
    │       │   ├── mod.rs
    │       │   └── app_config.rs    # pengaturan yang dipersistenkan
    │       └── log/
    │           ├── mod.rs
    │           └── entry.rs         # LogEntry berbasis key
    ├── zydra-i18n/
    │   ├── Cargo.toml
    │   ├── locales/
    │   │   ├── en.ftl
    │   │   └── id.ftl
    │   └── src/
    │       ├── lib.rs               # Locale, Messages
    │       ├── failure.rs           # CrackFailure -> teks
    │       ├── progress.rs          # CrackProgress -> teks
    │       └── log.rs               # LogEntry -> teks
    ├── zydra-cli/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs              # tipis: panggil cli::run
    │       ├── cli.rs               # definisi argumen clap
    │       ├── run.rs               # orkestrasi
    │       ├── render.rs            # progres + warna + laporan akhir
    │       └── json.rs              # keluaran terstruktur
    └── zydra-gui/
        ├── Cargo.toml
        ├── assets/fonts/            # Inter + JetBrains Mono (OFL)
        └── src/
            ├── main.rs              # tipis: jalankan eframe
            ├── app.rs               # ZydraApp — composition root
            ├── controller.rs        # jembatan UI <-> engine
            ├── theme.rs             # semua token visual
            ├── fonts.rs
            ├── responsive.rs        # satu-satunya tempat breakpoint
            ├── widgets/
            │   ├── mod.rs
            │   ├── card.rs
            │   ├── stat_tile.rs
            │   ├── progress_bar.rs
            │   ├── segmented.rs
            │   ├── file_row.rs
            │   ├── log_console.rs
            │   ├── result_banner.rs
            │   └── status_pill.rs
            └── panels/
                ├── mod.rs
                ├── sidebar.rs
                ├── workspace.rs
                └── settings.rs
```

## 6. `zydra-core` secara rinci

### 6.1 `model`

`TargetKind`: `Zip`, `Rar`, `Pdf`, `Shadow`, `Unsupported`.
`CrackTarget { kind: TargetKind, path: PathBuf, display_name: String }`.
Deteksi jenis dilakukan `TargetKind::detect(&Path) -> TargetKind`: ekstensi `zip`/`rar`/`pdf`, dan nama
berkas yang memuat `shadow` (tanpa membedakan huruf besar-kecil). Sisanya `Unsupported`.
Deteksi berdasarkan ekstensi saja, tidak membaca isi berkas — berbeda dari versi Python yang juga hanya
memakai ekstensi, tetapi sekarang hasilnya enum bertipe, bukan string.

`CrackFailure` — enum, tanpa teks. Berisi **kesalahan yang disebabkan masukan pengguna**; kesalahan
prasarana (I/O, berkas rusak) tinggal di `CoreError` (6.9). Pemisahan ini yang membuat lapisan tepi tahu
mana yang perlu diterjemahkan untuk pengguna dan mana yang cukup dicatat sebagai kesalahan teknis.

```
PasswordNotFound
UnsupportedFormat { display_name: String }
EmptyWordlist
FileUnreadable { reason: String }
NoCharsetSelected
InvalidLengthRange { min: u8, max: u8, allowed_max: u8 }
InvalidMask { pattern: String, reason: String }
InvalidRule { rule: String, reason: String }
TooManyCombinations { total: u64, limit: u64 }
NoCandidate
BackendUnavailable { backend: Backend, reason: String }
SessionFingerprintMismatch
```

`Rar5NotSupported` dihapus dari daftar: keputusan 3 menyatakan RAR4 dan RAR5 keduanya didukung, jadi
varian itu tidak punya keadaan yang bisa diwakilinya. Kalau build dibuat tanpa dukungan RAR, yang terjadi
adalah `BackendUnavailable { backend: Backend::Rar, .. }` — satu keadaan, satu cara dinyatakan.

Tidak ada varian `Cancelled` di sini: pembatalan adalah kejadian mesin, bukan kegagalan pemulihan
password, sehingga diwakili `EngineEvent::Cancelled` (lihat 6.4). Satu keadaan hanya punya satu cara
untuk dinyatakan.

`CrackProgress`:

```
Trying(TryingData)
Found  { password: String, detail: Option<String> }
Failed { failure: CrackFailure }
```

dengan

```
TryingData {
    tried: u64,
    total: u64,
    speed_per_second: u64,
    elapsed: Duration,
    eta: Option<Duration>,
}
```

`TryingData::fraction()` mengembalikan `f32` yang dijepit ke `0.0..=1.0`; pembagian dengan total nol
mengembalikan `0.0`, bukan `NaN` (versi Python tidak menjaga hal ini). `TryingData` sengaja dipisah dari
enum supaya state UI "sedang berjalan" (9.3) tidak bisa memuat varian `Found` atau `Failed` yang tidak
masuk akal di sana.

Kelas karakter: `lowercase`, `uppercase`, `digits`, `symbols`, `space`.
`CharsetSet` adalah bitflag dengan satu konstanta per kelas; `alphabet()` menyusunnya dalam urutan tetap
dan membuang duplikat; `total_combinations(min, max)` menjenuhkan ke `u64::MAX` saat melimpah, bukan
membungkus. Tidak ada varian `letters` — kombinasi `LOWERCASE | UPPERCASE` yang dipakai. `CharsetSet`
adalah **satu-satunya** tipe untuk konsep ini: nama-nama kelas adalah konstanta bitflag di dalamnya,
bukan enum terpisah. Dua tipe untuk satu konsep hanya menghasilkan kode konversi yang harus dijaga
tanpa manfaat.

`MaskPattern::parse(&str)` menerima token gaya hashcat: `?l` `?u` `?d` `?s` `?b` `?a` dan karakter
literal. `?a` adalah gabungan semua kelas. Pola kosong, token tidak dikenal, atau `?` di akhir adalah
kesalahan parse, bukan panik.

`RuleSet` memuat daftar `Rule`. Aturan yang didukung (subset yang dinyatakan, bukan seluruh hashcat):
`Capitalize`, `Upper`, `Lower`, `AppendDigits { max: u8 }`, `PrependDigits { max: u8 }`, `Leet`,
`Reverse`, `AppendSymbols`. Setiap aturan adalah transformasi murni `&str -> Vec<String>`; tidak ada
aturan yang bergantung pada keadaan luar.

### 6.2 `candidate`

```rust
pub trait CandidateSource: Send {
    fn next_candidate(&mut self) -> Option<String>;
    fn total(&self) -> u64;
    fn position(&self) -> u64;
    fn skip(&mut self, count: u64) {
        for _ in 0..count {
            if self.next_candidate().is_none() { break; }
        }
    }
}
```

- `WordlistSource` — membaca bertahap dengan `BufReader`, memotong `\r` dan `\n`, mengganti byte yang
  bukan UTF-8 secara lossy (versi Python memakai `errors="ignore"`). `total()` dihitung sekali di awal
  dengan satu lintasan cepat. `skip()` dioptimalkan dengan memajukan pembaca tanpa mengalokasikan string.
- `BruteForceSource` — generator odometer atas `CharsetSet` untuk panjang `min..=max`, tanpa
  mengumpulkan seluruh ruang ke memori.
- `MaskSource` — sama seperti brute force, tetapi tiap posisi punya himpunan karakternya sendiri.
- `RulesSource` — membungkus sumber dasar dan mengeluarkan hasil setiap aturan. `total()` = total dasar
  dikali jumlah aturan. Menghormati `skip()` dengan menghitung ulang indeks dasar dan indeks aturan.

### 6.3 `tester`

```rust
pub enum MatchOutcome { NoMatch, Matched { detail: Option<String> } }

pub enum EncryptionKind {
    None,
    ZipCrypto,
    ZipAes256,
    RarCrypto4,
    RarCrypto5,
    PdfStandard,
    UnixCrypt(UnixCryptScheme),
}

pub enum UnixCryptScheme { Des, Md5, Sha256, Sha512, Bcrypt, Other(String) }

pub struct TesterInfo {
    pub kind: TargetKind,
    pub encryption: EncryptionKind,
    pub entries: usize,
}

pub trait PasswordTester: Send {
    fn try_password(&mut self, password: &str) -> MatchOutcome;
    fn info(&self) -> &TesterInfo;
}

pub trait TesterFactory: Send + Sync {
    fn create(&self, target: &CrackTarget) -> Result<Box<dyn PasswordTester>, CrackFailure>;
}
```

`try_password` menerima `&mut self` karena sebagian backend perlu memuat ulang dokumen; setiap worker
memiliki tester sendiri, jadi tidak ada state bersama dan tidak ada penguncian.

Satu tester per worker berarti `create` dipanggil N kali. Factory bertanggung jawab melakukan pekerjaan
mahal sekali saja (membaca header, memeriksa jenis enkripsi) dan menyimpannya sebagai `Arc` yang
dibagikan ke tester berikutnya.

`DefaultTesterFactory` memetakan jenis target ke tester. Tester RAR memeriksa tanda tangan berkas:
`Rar!\x1a\x07\x00` = RAR4, `Rar!\x1a\x07\x01\x00` = RAR5. Keduanya didukung.

`shadow.rs` mem-parse baris `/etc/shadow`: ambil kolom ke-0 sebagai nama pengguna dan kolom ke-1 sebagai
hash; lewati entri yang diawali `*`, `!`, atau `!!` (akun terkunci, tidak bisa dipecahkan). Baris tanpa
`:` dilewati. Verifikasi memakai crate `pwhash` sehingga berjalan di Windows juga — versi Python gagal
total di Windows karena modul `crypt` tidak ada.

### 6.4 `engine`

```rust
pub struct EngineConfig {
    pub workers: usize,
    pub batch_size: usize,
    pub progress_interval: Duration,
}

pub enum EngineEvent {
    Started { total: u64 },
    Progress(TryingData),
    Found { password: String, detail: Option<String> },
    Exhausted,
    Cancelled,
    Failed(CrackFailure),
}

pub fn run(
    source: Box<dyn CandidateSource>,
    factory: Arc<dyn TesterFactory>,
    target: CrackTarget,
    config: EngineConfig,
    cancel: CancelToken,
) -> Receiver<EngineEvent>
```

`factory` dan `target` diterima sebagai nilai milik (bukan referensi) karena `run` mengembalikan segera
dan pekerjaannya berlanjut di thread terpisah: data yang dipindahkan ke thread wajib berumur `'static`.
`Arc<dyn TesterFactory>` membuat factory bisa dibagikan ke semua worker tanpa duplikasi pekerjaan mahal
seperti membaca header arsip.

Cara kerja:

1. `factory.create` dipanggil `config.workers` kali. Kegagalan pada percobaan pertama langsung
   menghasilkan `EngineEvent::Failed` dan mengakhiri fungsi.
2. Sumber kandidat diletakkan di balik `Mutex`; setiap worker menarik satu batch (default 64 kandidat)
   lalu mengujinya tanpa memegang kunci.
3. Worker berhenti saat `cancel` aktif atau saat menemukan kecocokan. Worker pertama yang cocok
   memenangkan `compare_exchange` pada flag bersama, lalu mengirim `Found`.
4. Satu thread pengawas mengirim `Progress` setiap `progress_interval` (default 350 ms) selama pekerjaan
   berjalan, dan mengirim `Exhausted`, `Cancelled`, atau `Failed` di akhir.
5. Fungsi mengembalikan `Receiver<EngineEvent>` dan segera kembali; pemanggil membaca channel dari loop
   frame UI atau loop tick CLI. Ini yang membuat UI tidak pernah membeku.

Thread biasa, bukan `tokio`. Beban kerjanya CPU-bound, jadi runtime async hanya menambah dependensi dan
titik kegagalan tanpa manfaat.

`stats.rs` menghitung kecepatan dengan rata-rata bergerak eksponensial (faktor halus 0,3) alih-alih
rata-rata kumulatif dari awal, sehingga ETA ikut menyesuaikan saat kecepatan berubah. Versi Python
memakai rata-rata kumulatif sehingga ETA-nya makin melenceng pada serangan panjang.

`cancel.rs` membungkus `Arc<AtomicBool>` menjadi `CancelToken` dengan `Clone`, `cancel()`, dan
`is_cancelled()`. Satu token dapat dipakai bersama UI, CLI, dan penangan sinyal.

### 6.5 `session`

```rust
pub struct TargetFingerprint {
    pub path: PathBuf,
    pub size: u64,
    pub modified_unix: i64,
    pub kind: TargetKind,
}

pub enum AttackPlan {
    Dictionary { wordlist: PathBuf },
    BruteForce { charset: CharsetSet, min: u8, max: u8 },
    Mask { pattern: String },
    WithRules { base: Box<AttackPlan>, rules: RuleSet },
}

pub struct SessionState {
    pub version: u32,
    pub fingerprint: TargetFingerprint,
    pub plan: AttackPlan,
    pub position: u64,
    pub updated_at: i64,
}
```

Disimpan sebagai JSON di direktori data milik OS (lewat crate `directories`):
Windows `%APPDATA%\Zydra\sessions\`, macOS `~/Library/Application Support/Zydra/sessions/`,
Linux `~/.local/share/zydra/sessions/`. Nama berkas diturunkan dari hash sidik jari, jadi satu target
punya satu berkas sesi.

Saat memuat: `version` harus cocok, dan `fingerprint` harus sama persis. Kalau tidak, hasilnya
`CrackFailure::SessionFingerprintMismatch` — aplikasi tidak menebak-nebak dan tidak melanjutkan dari
posisi yang salah. Resume bekerja dengan `source.skip(position)`.

`position` diperbarui setiap 5 detik selama serangan berjalan (dan sekali lagi saat berhenti), bukan
setiap kandidat, supaya tidak membanjiri disk.

### 6.6 `config`

`AppConfig` disimpan sebagai TOML di direktori konfigurasi OS:

```
language: "auto" | "en" | "id"
workers: usize          (0 = otomatis mengikuti jumlah CPU)
max_combinations: u64   (default 20_000_000)
max_brute_length: u8    (default 6)
batch_size: usize       (default 64)
progress_interval_ms: u64 (default 350)
rar_mode: "auto" | "rar4" | "rar5"
zip_max_entry_bytes: usize (default 67_108_864)
log_keep_entries: usize (default 5000)
```

`rar_mode` menentukan bagaimana tester RAR menafsirkan arsip: `auto` (bawaan) membaca tanda tangan
berkas dan memilih RAR4 atau RAR5; `rar4` dan `rar5` memaksa satu tafsiran. Kalau mode dipaksa tetapi
tanda tangan berkas menyatakan versi lain, tester mengembalikan
`CrackFailure::BackendUnavailable` beserta alasannya, bukan diam-diam mencoba versi yang salah.

Berkas konfigurasi yang rusak atau tidak lengkap tidak membuat aplikasi gagal jalan: nilai yang tidak
terbaca diganti nilai default, dan satu entri log peringatan dicatat.

### 6.7 `log`

```rust
pub struct LogEntry {
    pub at: i64,
    pub level: LogLevel,
    pub key: String,
    pub args: BTreeMap<String, String>,
}
```

Log menyimpan **key dan argumen**, bukan kalimat jadi. Akibatnya log bisa ditampilkan dan diekspor dalam
bahasa yang sedang aktif, dan bahasa bisa diganti setelah kejadian tercatat. `LogLevel`: `Info`,
`Warn`, `Error`, `Success`.

Cincin penyimpanan dibatasi `log_keep_entries` supaya serangan berjam-jam tidak menghabiskan memori.
Ekspor menulis satu baris per entri: `HH:MM:SS LEVEL teks`.

### 6.8 `constants.rs`

Semua angka ajaib berkumpul di sini: ukuran batch, interval progres, batas kombinasi, panjang brute
force maksimum, jumlah byte verifikasi, faktor penghalusan kecepatan, interval simpan sesi, ukuran
jendela minimum, batas breakpoint GUI. Tidak ada angka telanjang yang tersebar di modul lain.

### 6.9 `error.rs`

`CoreError` memuat **kesalahan prasarana**, bukan kesalahan masukan pengguna (yang tinggal di
`CrackFailure`, 6.1):

```
Io(std::io::Error)
ConfigInvalid { detail: String }
SessionCorrupt { detail: String }
SessionIo { detail: String }
```

Pesan `thiserror` di sini bersifat teknis dan ditujukan untuk catatan/log, bukan untuk ditampilkan ke
pengguna. Lapisan tepi (`zydra-cli`, `zydra-gui`) selalu memetakan `CoreError` ke satu key i18n umum
(`error-unknown`) ditambah detail teknis di log — sehingga tidak ada kalimat berbahasa Inggris yang
bocor ke antarmuka pengguna.

## 7. Backend format

| Format | Crate | Cara verifikasi | Perbaikan atas versi Python |
|---|---|---|---|
| ZIP | `zip` (fitur `aes-crypto`) | Buka arsip dengan `ZipArchive::by_index_decrypt`, lalu **baca entri pertama sampai habis**. AES-256 punya verifikasi HMAC sehingga pasti; ZipCrypto hanya memvalidasi 1 dari 256 bit, jadi password salah bisa lolos di header dan hanya ketahuan dari CRC32 di akhir entri. Bacaan dibatasi `zip_max_entry_bytes`; kalau batas tercapai, kandidat tetap dilaporkan cocok tetapi `detail` menyatakan verifikasinya terpotong | Python memanggil `extractall` ke disk dan membuat `temp_directory` berisi salinan arsip. Rust tidak menulis berkas sama sekali |
| RAR | `unrar` (libunrar dibundel) | `Archive::with_password(path, pw)`, lalu `open_for_processing()`, `read_header()`, dan `read()` entri pertama. Password salah menghasilkan `Err` | Python hanya RAR4 dan bergantung paket `rarfile` |
| PDF | `lopdf` | `Document::load_with_password(path, pw)`, lalu pastikan `!was_encrypted()` atau `encryption_state.is_some()`, dan halaman bisa diakses | Python **menjalankan binary `qpdf`** yang harus dipasang lebih dulu. Rust murni |
| shadow | `pwhash` | `pwhash::unix::verify(password, hash)` — mengenali sendiri format modular (`$1$`, `$5$`, `$6$`, `$2y$`) | Python memakai modul `crypt` yang hanya ada di Unix; di Windows fitur ini mati total. Rust jalan di ketiganya |

Catatan penting soal ZIP: `by_index_decrypt` pada crate `zip` secara eksplisit memperingatkan bahwa
fungsi itu "sometimes accepts wrong password" karena ZipCrypto hanya menyediakan 1 dari 256 bit untuk
validasi. Karena itu verifikasi **wajib** membaca entri sampai habis dan bersandar pada CRC32, bukan
pada keberhasilan pembukaan header. Ini juga alasan `zip_verify_bytes` dihapus dari konfigurasi:
memeriksa beberapa kilobyte pertama tidak membuktikan apa pun untuk ZipCrypto.

Untuk PDF, berkas hasil dekripsi tidak ditulis otomatis. Setelah password ditemukan, pengguna menekan
tombol simpan dan memilih lokasi; baru dokumen ditulis ke sana. Ini menghindari efek samping tersembunyi.

Verifikasi RAR5 dilakukan di tester yang sama; perbedaan header ditangani oleh libunrar, sedangkan
pemeriksaan tanda tangan hanya dipakai untuk mengisi `TesterInfo::encryption`.

## 8. `zydra-i18n`

- Dua berkas locale: `locales/en.ftl` dan `locales/id.ftl`.
- Keduanya disematkan ke dalam binary dengan `include_str!`. Tidak ada berkas yang bisa hilang saat
  aplikasi dipindahkan atau dipasang.
- `Locale`: `En`, `Id`. `Locale::detect()` membaca locale sistem lewat crate `sys-locale`; nilai tak
  dikenal jatuh ke `En`.
- `Messages` membungkus `FluentBundle` untuk locale aktif dan menyediakan:
  - `text(&self, key: &str) -> String`
  - `text_args(&self, key: &str, args: &[(&str, &str)]) -> String`
  - `failure(&self, failure: &CrackFailure) -> String`
  - `progress(&self, progress: &CrackProgress) -> String`
  - `log_line(&self, entry: &LogEntry) -> String`
- `Locale` bisa diganti saat aplikasi berjalan; seluruh teks ikut berubah pada frame berikutnya.

Cakupan key (nama awal, boleh bertambah, tidak boleh berkurang tanpa alasan):

```
app-title, app-subtitle, app-status-active, app-window-title
sidebar-target-label, sidebar-target-empty, sidebar-target-browse, sidebar-target-change
sidebar-method-dictionary, sidebar-method-brute-force, sidebar-method-mask
sidebar-wordlist-label, sidebar-wordlist-empty, sidebar-wordlist-browse, sidebar-wordlist-lines
sidebar-charset-label, charset-lowercase, charset-uppercase, charset-digits, charset-symbols, charset-space
sidebar-mask-label, sidebar-mask-placeholder, sidebar-mask-hint
sidebar-rules-label, rule-capitalize, rule-upper, rule-lower, rule-append-digits, rule-prepend-digits, rule-leet, rule-reverse, rule-append-symbols
sidebar-length-min, sidebar-length-max, sidebar-combinations
action-start, action-stop, action-copy-password, action-copied, action-open-folder, action-save-decrypted, action-export-log, action-resume, action-open-settings, action-close
workspace-status-idle, workspace-status-analyzing, workspace-status-cracking-zip, workspace-status-cracking-rar, workspace-status-cracking-pdf, workspace-status-cracking-shadow, workspace-status-decrypting, workspace-status-done
stat-speed, stat-eta, stat-elapsed, stat-tried, stat-value-unknown, stat-value-speed, stat-value-tried
log-panel-title, log-panel-empty, log-panel-exported
result-granted, result-denied, result-key, result-detail
error-unsupported-format, error-empty-wordlist, error-no-match, error-file-unreadable, error-no-charset, error-bad-range, error-invalid-mask, error-invalid-rule, error-too-many-combinations, error-no-candidate, error-backend-unavailable, error-session-mismatch, error-unknown
settings-title, settings-language, settings-language-auto, settings-workers, settings-workers-auto, settings-max-combinations, settings-max-brute-length, settings-rar-mode, settings-rar-auto, settings-rar4, settings-rar5, settings-zip-verify-bytes, settings-save, settings-reset
cli-about, cli-help-file, cli-help-wordlist, cli-help-charset, cli-help-min, cli-help-max, cli-help-mask, cli-help-rules, cli-help-threads, cli-help-lang, cli-help-json, cli-help-resume, cli-help-quiet
```

Pemetaan `CrackFailure` ke key bersifat satu-satu dan hidup di `zydra-i18n/src/failure.rs`, bukan di
`zydra-core`.

## 9. GUI

### 9.1 Tema

Seluruh nilai visual berada di `theme.rs` sebagai satu struktur `Theme` dengan nilai bawaan. Tidak ada
warna atau radius yang ditulis langsung di dalam panel atau widget.

| Token | Nilai |
|---|---|
| `bg_app` | `#0B0F14` |
| `bg_panel` | `#111823` |
| `bg_console` | `#080C11` |
| `bg_track` | `#1A2531` |
| `border` | `#1E2B3A` |
| `border_strong` | `#2A3B4F` |
| `text` | `#E6EDF3` |
| `text_muted` | `#8B9BB0` |
| `accent` | `#22D3EE` |
| `success` | `#4ADE80` |
| `warning` | `#FBBF24` |
| `error` | `#F87171` |
| `radius_card` | 8 |
| `radius_control` | 6 |
| `radius_pill` | 999 |
| `space_unit` | 4 |

Tipografi: Inter untuk teks antarmuka, JetBrains Mono untuk angka, persentase, nama berkas, dan konsol
log. Keduanya berlisensi SIL OFL 1.1 sehingga aman digabung dengan GPL-3.0. Kalau berkas font tidak
tersedia saat build, aplikasi jatuh ke font bawaan egui dan tetap berjalan; tidak ada kegagalan keras.

Angka selalu memakai lebar tabular supaya tidak bergetar saat nilai berubah.

### 9.2 Layout dan responsif

Layout adalah Sidebar + Workspace. Semua keputusan responsif hanya ada di `responsive.rs`, yang
mengembalikan struktur `LayoutMode` dari lebar jendela:

| Lebar | Mode | Perilaku |
|---|---|---|
| `>= 1100` | `Wide` | Sidebar 240 px tetap di kiri. Statistik 3 kolom. Konsol log tinggi penuh |
| `720 .. 1100` | `Medium` | Sidebar 200 px, bisa diciutkan. Statistik 2 kolom. Konsol log 6 baris |
| `< 720` | `Narrow` | Sidebar pindah ke atas sebagai panel yang bisa dibuka-tutup. Statistik 1 kolom. Konsol log 4 baris. Semua tombol lebar penuh |
| `< 420` | `Compact` | Seperti `Narrow`, ditambah label panjang disingkat dan blok statistik berubah jadi daftar bertumpuk |

Jendela memiliki ukuran minimum 360×480 supaya layout `Compact` selalu bisa ditampilkan. Tinggi
minimum konsol log dijaga agar tidak pernah nol.

### 9.3 Alur data

```
UI (app.rs)                     controller.rs                 core
   │ tekan Start ────────────────>│
   │                              │ baca UiState -> AttackPlan
   │                              │ session::store::save
   │                              │ engine::run(source, factory, ...)
   │                              │<── Receiver<EngineEvent>
   │<── drain per frame ──────────│
   │ perbarui UiState             │
   │ render                       │
```

`app.rs` tidak pernah memanggil `engine` secara langsung dan tidak pernah memanggil tester. Ia hanya
memegang `UiState` dan `Controller`. Semua pekerjaan berat berjalan di thread milik engine; loop UI
hanya menarik event dari channel. Karena itu UI tidak pernah membeku dan tombol Stop selalu responsif.

`UiState`:

```rust
pub enum UiPhase {
    Idle,
    Analyzing,
    Running { progress: TryingData },
    Done { password: String, detail: Option<String> },
    Error { failure: CrackFailure },
}
```

`UiState` juga memegang pilihan yang belum dijalankan (target, metode, wordlist, charset, mask, aturan,
panjang, panel mana yang terbuka). Ini adalah state UI, bukan state domain.

### 9.4 Panel

- `sidebar` — pemilih target, pemilih metode (segmented: Dictionary / Brute force / Mask), berkas
  wordlist, kotak centang charset, kotak mask, kotak centang aturan, panjang min/max, jumlah kombinasi
  yang dihitung langsung, tombol Start dan Stop.
- `workspace` — header status dengan pil keadaan, persentase besar, bilah progres, tile statistik
  (kecepatan, waktu berjalan, ETA, sudah dicoba), konsol log, spanduk hasil dengan tombol salin dan
  buka folder.
- `settings` — panel yang bisa dibuka: bahasa, jumlah worker, batas kombinasi, panjang brute force
  maksimum, mode RAR, byte verifikasi ZIP, tombol simpan dan kembalikan ke bawaan.

### 9.5 Widget

`card`, `stat_tile`, `progress_bar`, `segmented`, `file_row`, `log_console`, `result_banner`,
`status_pill`. Setiap widget adalah fungsi murni `fn(&mut Ui, &Theme, ...)` yang menerima data dan tidak
memegang state aplikasi. Tidak ada widget yang membaca `AppConfig` atau memanggil `engine`.

### 9.6 Crate pendukung

`rfd` untuk dialog berkas native, `arboard` untuk clipboard, `opener` untuk membuka folder hasil,
`sys-locale` untuk mendeteksi bahasa sistem, `directories` untuk lokasi konfigurasi dan data.

## 10. CLI

`clap` dengan turunan. Argumen lama dari `Zydra.py` tetap bekerja supaya perintah yang sudah ada tidak
rusak:

| Lama | Baru | Keterangan |
|---|---|---|
| `-f <file>` | `-f`, `--file` | berkas target |
| `-d <wordlist>` | `-d`, `--wordlist` | mode dictionary |
| `-b <charset>` | `-b`, `--charset` | `lowercase,uppercase,digits,symbols,space` (daftar dipisah koma). Nilai `letters` diterima sebagai alias lama dan berarti `lowercase,uppercase` — ini alias di lapisan CLI saja, bukan varian di `CharsetSet` |
| `-m <n>` | `-m`, `--min-length` | panjang minimum |
| `-x <n>` | `-x`, `--max-length` | panjang maksimum |

Tambahan: `--mask <pattern>`, `--rules <list>`, `--threads <n>`, `--lang <en|id>`, `--json`, `--quiet`,
`--resume`, `--no-resume`, `--config <path>`, `--version`, `--help`.

Progres digambar dengan `indicatif`. Warna memakai `owo-colors` dengan deteksi dukungan warna, dan
mode `--quiet` menghasilkan keluaran polos tanpa warna sama sekali (versi Python selalu memaksa warna
dan selalu mencetak banner). Semua teks bantuan berasal dari katalog i18n, jadi `--lang id --help`
menghasilkan bantuan berbahasa Indonesia.

Kode keluar: `0` password ditemukan, `1` tidak ditemukan, `2` kesalahan penggunaan, `3` kesalahan
berkas atau backend. Versi Python selalu memakai `exit(0)` sehingga tidak bisa dipakai di skrip.

Keluaran `--json` memuat satu objek per baris untuk peristiwa, dan satu objek ringkasan di akhir.

## 11. Fitur tambahan

### 11.1 Salin dan buka hasil
Tombol salin memakai `arboard`. Untuk PDF, tombol simpan menulis dokumen yang sudah didekripsi ke lokasi
pilihan pengguna; setelah itu tombol buka folder memanggil `opener`.

### 11.2 Resume
Dijelaskan di 6.5. Di UI, tombol Resume muncul di panel hasil hanya jika ada berkas sesi yang sidik
jarinya cocok dengan target yang sedang dipilih. Kalau tidak cocok, tombol tidak ditampilkan dan alasannya
tercatat di log.

### 11.3 Pengaturan lanjutan
Panel settings menulis ke `AppConfig`. Perubahan berlaku untuk serangan berikutnya; serangan yang sedang
berjalan tidak diubah di tengah jalan.

### 11.4 Mask dan rules
`MaskPattern` dan `RuleSet` menjadi `CandidateSource` sehingga mesin paralel tidak perlu diubah sama
sekali. Jumlah kombinasi mask dihitung dari ukuran tiap posisi dan tunduk pada `max_combinations` yang
sama seperti mode lain. `RulesSource` melipatgandakan total kandidat dasar; angka hasil perkalian ini
yang ditampilkan di UI supaya pengguna tahu berapa banyak percobaan yang akan dilakukan.

### 11.5 Ekspor log
Menulis seluruh `LogEntry` yang tersimpan ke berkas teks di lokasi pilihan pengguna, diterjemahkan ke
bahasa aktif saat ekspor. Di CLI, `--json` memakai jalur yang sama.

## 12. Pengujian

### 12.1 Unit test di `zydra-core`
- `charset`: jumlah kombinasi, penjenuhan saat melimpah, pembuangan duplikat, urutan alfabet stabil.
- `bruteforce`: urutan keluaran untuk alfabet kecil (dibandingkan dengan daftar yang dihitung tangan),
  `total()` cocok dengan jumlah keluaran nyata, `skip()` mendarat di posisi yang benar.
- `mask`: penguraian pola benar dan salah, `total()` cocok dengan jumlah keluaran nyata.
- `rules`: setiap aturan pada masukan tetap, `total()` = dasar dikali aturan, `skip()` konsisten.
- `wordlist`: baris dengan `\r\n`, baris kosong, byte bukan UTF-8, penghitungan baris.
- `shadow`: baris valid, akun terkunci, baris rusak, kolom kurang.
- `session`: sidik jari cocok dan tidak cocok, versi berbeda, berkas rusak.
- `stats`: ETA stabil saat kecepatan konstan, menyesuaikan saat kecepatan berubah, tidak ada pembagian
  dengan nol.
- `engine`: dengan tester palsu yang cocok pada kandidat ke-N, `Found` muncul dengan password yang benar;
  pembatalan menghasilkan `Cancelled` dan tidak ada worker yang menggantung.

### 12.2 Integrasi
- **ZIP**: tes membuat arsip ZIP terenkripsi sendiri memakai crate `zip` (ZipCrypto dan AES-256), lalu
  memecahkannya dengan wordlist kecil dan dengan brute force. Tidak perlu berkas biner di repositori.
- **shadow**: tes membangun berkas shadow sintetis dengan hash yang dihitung memakai crate `pwhash`
  untuk sha512crypt dan md5crypt, lalu memverifikasi bahwa password yang benar ditemukan dan yang salah
  tidak.
- **PDF**: berkas PDF terenkripsi kecil disimpan sebagai fixture di repositori (ukurannya di bawah 20 KB)
  dengan password yang didokumentasikan di komentar tes.
- **RAR**: kalau fixture RAR terenkripsi bisa dibuat di mesin pengembangan memakai 7-Zip, fixture kecil
  disimpan dan tesnya aktif di CI. Kalau tidak bisa, tesnya ditandai `#[ignore]` dengan komentar yang
  menyebutkan alasannya secara jujur, dan README menyatakan bahwa RAR belum tercakup tes otomatis.
  Tidak ada klaim "teruji" untuk sesuatu yang belum teruji.

### 12.3 GUI
Logika `responsive.rs` diuji sebagai fungsi murni: lebar masuk, `LayoutMode` keluar, termasuk nilai
batas tepat di 1100, 720, dan 420. Tidak ada tes yang memerlukan jendela sungguhan.

### 12.4 Pemeriksaan mutu
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, dan `cargo test --workspace` harus
lolos di ketiga sistem operasi.

## 13. CI/CD

`.github/workflows/ci.yml` — pada setiap push dan pull request, matriks
`[ubuntu-latest, macos-latest, windows-latest]`:
1. `cargo fmt --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test --workspace`
4. pemeriksaan aturan dependensi: `zydra-core` tidak boleh memuat crate UI
5. `cargo build --release`

Linux memasang `libgtk-3-dev` dan `libxkbcommon-dev` (dibutuhkan `rfd` dan `winit`). Cache memakai
`Swatinem/rust-cache`.

`.github/workflows/release.yml` — pada push tag `v*`, membangun tiga target:
`x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` (plus
`x86_64-apple-darwin`), mengemasnya (`.zip` untuk Windows, `.tar.gz` untuk Linux dan macOS, masing-masing
memuat binary, `LICENSE`, `NOTICE`, dan `README.md`), lalu mengunggahnya sebagai aset GitHub Release.

## 14. Lisensi dan atribusi

- Proyek ini **GPL-3.0-or-later**. Berkas `LICENSE` memakai teks kanonik GPL-3.0.
- Proyek asli `hamedA2/Zydra` berlisensi MIT, Copyright (c) 2019 hamedA2. MIT kompatibel dengan GPL-3.0,
  jadi karya turunan ini didistribusikan di bawah GPL dengan kredit asli tetap disertakan di `NOTICE`.
- crate `unrar` mengompilasi libunrar, yang tunduk pada lisensi UnRAR: boleh dipakai untuk mengekstrak,
  dilarang dipakai untuk membuat kompresor RAR. Tool ini hanya mengekstrak, jadi pemakaiannya sesuai.
  Batasan ini dicantumkan di `NOTICE` dan di bagian lisensi README, karena binary hasil build memuat
  komponen berlisensi UnRAR.
- Font Inter dan JetBrains Mono berlisensi SIL OFL 1.1; berkas lisensinya disertakan di
  `crates/zydra-gui/assets/fonts/`.

## 15. Tahapan implementasi

Setiap tahap harus meninggalkan repositori dalam keadaan bisa di-build dan tesnya lolos.

1. **Fondasi** — workspace, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore`,
   `constants.rs`, `error.rs`, `model::{target,failure,progress,charset}`, kerangka `zydra-i18n`.
2. **Kandidat dan mesin** — `candidate::{wordlist,bruteforce}`, `engine::{crack,stats,cancel}`,
   lengkap dengan unit test dan tes dengan tester palsu.
3. **ZIP** — `tester::{mod,factory,zip}`, `zydra-cli` minimal dengan argumen lama. Setelah tahap ini CLI
   sudah bisa menggantikan `Zydra.py` untuk berkas ZIP.
4. **Sisa backend** — `tester::{shadow,pdf,rar}`, fixture dan tes integrasi.
5. **GUI** — `theme`, `fonts`, `responsive`, `widgets`, `panels`, `app`, `controller`.
6. **Fitur lanjutan** — `session::store`, `log`, `config`, mask, rules, panel settings, ekspor log,
   salin dan buka hasil.
7. **Rilis** — `ci.yml`, `release.yml`, README, NOTICE, pemeriksaan akhir lintas platform.

## 16. Risiko dan mitigasi

| Risiko | Dampak | Mitigasi |
|---|---|---|
| `unrar` butuh compiler C++ saat build | Build gagal di lingkungan tanpa compiler | MSVC, gcc, dan clang sudah tersedia di runner CI. Persyaratan dicatat di README. Kalau build RAR tetap gagal, fitur RAR bisa dimatikan lewat flag cargo tanpa mengorbankan format lain |
| Fixture RAR terenkripsi tidak bisa dibuat | Tes RAR end-to-end tidak jalan | Tes ditandai `#[ignore]` dengan alasan jujur; README menyatakan RAR belum tercakup tes otomatis |
| Font perlu diunduh sekali | Repositori tanpa font, tampilan memakai font bawaan | Aplikasi tetap berjalan dengan font bawaan egui; font dibundel menyusul sebagai langkah terpisah |
| Enam fitur tambahan memperbesar lingkup | Paritas penuh tertunda | Tahapan di bagian 15 menempatkan paritas penuh di tahap 1–4, sebelum fitur tambahan di tahap 6 |
| Lisensi UnRAR | Binary memuat komponen berlisensi terbatas | Dicantumkan di `NOTICE` dan README; pemakaian hanya untuk ekstraksi sehingga sesuai |
| Mode Mask dan rules meluas tanpa kendali | Permukaan kode membengkak | Cakupan aturan dibatasi pada daftar di 6.1; aturan baru hanya ditambah kalau diminta |

## 17. Kriteria penerimaan

1. `cargo build --release` berhasil di Windows, macOS, dan Linux lewat CI.
2. `cargo test --workspace` lolos di ketiga sistem.
3. `cargo clippy --all-targets -- -D warnings` bersih di ketiga sistem.
4. `zydra-core` tidak memuat dependensi UI apa pun, dibuktikan oleh pemeriksaan di CI.
5. Tidak ada kalimat yang dilihat pengguna di luar `locales/*.ftl`. Dibuktikan dengan pencarian string
   literal pada kode selain katalog.
6. CLI memecahkan berkas ZIP, PDF, dan shadow dengan perintah yang sama seperti `Zydra.py`
   (`-f`, `-d`, `-b`, `-m`, `-x`).
7. GUI memecahkan berkas ZIP dari awal sampai selesai tanpa membeku, dengan tombol Stop yang merespons.
8. GUI tetap terpakai dan tidak ada elemen terpotong pada lebar jendela 360, 420, 720, 1100, dan 1600 px.
9. Mengganti bahasa di panel settings mengubah seluruh teks antarmuka tanpa memulai ulang aplikasi.
10. Resume melanjutkan serangan brute force dari posisi yang tersimpan, dan menolak dengan pesan jelas
    kalau berkas target sudah berubah.
11. Ekspor log menghasilkan berkas teks dalam bahasa yang sedang aktif.
12. Push tag menghasilkan tiga paket rilis yang bisa diunduh dari GitHub Releases.
13. `NOTICE` memuat atribusi MIT `hamedA2/Zydra` dan catatan lisensi UnRAR.
