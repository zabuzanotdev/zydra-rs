# Zydra Rust — Rencana A1: Workspace, Model, Kandidat, dan Mesin

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Membangun fondasi `zydra-core` — workspace Cargo, model domain, sumber kandidat, dan mesin
pemecah paralel yang sudah teruji dengan tester palsu.

**Architecture:** Satu crate pustaka murni (`zydra-core`) tanpa dependensi UI apa pun. Model domain
adalah enum dan struct bertipe tanpa teks pengguna. Sumber kandidat dan tester adalah dua trait yang
saling tidak tahu, dan mesin paralel hanya tahu kedua trait itu — sehingga menambah format atau metode
serangan tidak menyentuh mesin.

**Tech Stack:** Rust stable (edition 2021), `bitflags` 2, `thiserror` 1, `crossbeam-channel` 0.5,
`num_cpus` 1, `tempfile` (dev).

## Global Constraints

- Rust stable, edition 2021, `rust-version = "1.75"`.
- Lisensi setiap crate: `GPL-3.0-or-later`.
- `zydra-core` **tidak boleh** bergantung pada crate UI apa pun (`egui`, `eframe`, `clap`, `indicatif`, `rfd`, `arboard`, `opener`, `owo-colors`).
- `zydra-core` **tidak boleh** memuat kalimat yang dilihat pengguna. Tidak ada `println!`, `eprintln!`, atau string pesan berbahasa manusia di dalamnya.
- `zydra-core` **tidak boleh** memanggil `std::process::exit`, `std::process::abort`, atau `panic!` untuk kesalahan yang bisa dipicu masukan pengguna.
- Semua angka ajaib hidup di `crates/zydra-core/src/constants.rs`.
- Akhir baris LF di repositori (sudah diatur `.gitattributes`).
- `cargo fmt --check` dan `cargo clippy --all-targets -- -D warnings` harus bersih di setiap akhir tugas.
- Kode di dalam rencana ini ilustratif, bukan hasil `rustfmt`. Beberapa barisnya melewati 100 kolom, jadi **jalankan `cargo fmt` lebih dulu**, baru `cargo fmt --check`. Perbedaan antara keduanya bukan kesalahan.
- Kesalahan masukan pengguna memakai `CrackFailure`. Kesalahan prasarana memakai `CoreError`. Tidak ada tumpang tindih.

## File Structure

| Berkas | Tanggung jawab |
|---|---|
| `Cargo.toml` | Deklarasi workspace dan daftar dependensi bersama. Tanpa logika |
| `rust-toolchain.toml` | Versi toolchain dan komponen `rustfmt`, `clippy` |
| `rustfmt.toml` | Gaya format |
| `clippy.toml` | Batas MSRV untuk lint |
| `crates/zydra-core/Cargo.toml` | Metadata crate dan dependensi |
| `crates/zydra-core/src/lib.rs` | Re-export publik. Tanpa logika |
| `crates/zydra-core/src/constants.rs` | Semua angka ajaib |
| `crates/zydra-core/src/error.rs` | `CoreError` (kesalahan prasarana) |
| `crates/zydra-core/src/model/mod.rs` | Deklarasi submodul model |
| `crates/zydra-core/src/model/charset.rs` | `CharsetSet`, `alphabet()`, `total_combinations()`, `parse()` |
| `crates/zydra-core/src/model/target.rs` | `TargetKind`, `CrackTarget`, deteksi jenis berkas |
| `crates/zydra-core/src/model/failure.rs` | `CrackFailure`, `Backend` |
| `crates/zydra-core/src/model/progress.rs` | `TryingData`, `CrackProgress` |
| `crates/zydra-core/src/candidate/mod.rs` | Trait `CandidateSource` |
| `crates/zydra-core/src/candidate/wordlist.rs` | `WordlistSource` |
| `crates/zydra-core/src/candidate/bruteforce.rs` | `BruteForceSource` |
| `crates/zydra-core/src/tester/mod.rs` | Trait `PasswordTester`, `TesterFactory`, `TesterInfo`, `EncryptionKind`, `MatchOutcome` |
| `crates/zydra-core/src/engine/mod.rs` | Deklarasi submodul engine |
| `crates/zydra-core/src/engine/cancel.rs` | `CancelToken` |
| `crates/zydra-core/src/engine/stats.rs` | `SpeedMeter` |
| `crates/zydra-core/src/engine/crack.rs` | `EngineConfig`, `EngineEvent`, `run()` |

---

### Task 1: Workspace scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `rustfmt.toml`
- Create: `clippy.toml`
- Create: `crates/zydra-core/Cargo.toml`
- Create: `crates/zydra-core/src/lib.rs`

**Interfaces:**
- Consumes: tidak ada.
- Produces: crate `zydra_core` (nama paket `zydra-core`) yang bisa di-`cargo build`.

- [ ] **Step 1: Buat berkas workspace**

`Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = ["crates/zydra-core"]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "GPL-3.0-or-later"
repository = "https://github.com/zabuzanotdev/zydra-rs"
authors = ["zabuzanotdev"]

[workspace.dependencies]
bitflags = "2"
crossbeam-channel = "0.5"
num_cpus = "1"
thiserror = "1"
tempfile = "3"

[profile.release]
lto = "thin"
codegen-units = 1
```

`rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

`rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
```

`clippy.toml`:

```toml
msrv = "1.75"
```

- [ ] **Step 2: Buat crate `zydra-core`**

Jalankan:

```bash
cargo new --lib --vcs none crates/zydra-core
```

`crates/zydra-core/Cargo.toml`:

```toml
[package]
name = "zydra-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description = "Logika pemulihan password untuk arsip terenkripsi. Tanpa dependensi UI."

[dependencies]
bitflags.workspace = true
crossbeam-channel.workspace = true
num_cpus.workspace = true
thiserror.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

`crates/zydra-core/src/lib.rs`:

```rust
//! Logika pemulihan password untuk ZIP, RAR, PDF, dan berkas `shadow` Unix.
//!
//! Crate ini tidak memuat dependensi antarmuka pengguna dan tidak memuat kalimat yang
//! ditampilkan kepada pengguna. Kesalahan dikembalikan sebagai tipe; penerjemahan menjadi
//! teks adalah tanggung jawab crate `zydra-i18n`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
```

- [ ] **Step 3: Verifikasi build**

Run: `cargo build --workspace`
Expected: sukses tanpa peringatan.

Run: `cargo test --workspace`
Expected: `running 0 tests ... test result: ok. 0 passed`.

- [ ] **Step 4: Verifikasi lint**

Run: `cargo fmt --check`
Expected: tidak ada keluaran.

Run: `cargo clippy --all-targets -- -D warnings`
Expected: sukses tanpa peringatan.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml rust-toolchain.toml rustfmt.toml clippy.toml crates/zydra-core
git commit -m "chore(workspace): give the core a room of its own"
```

---

### Task 2: `model::charset` (plus `constants` dan `error`)

**Files:**
- Create: `crates/zydra-core/src/constants.rs`
- Create: `crates/zydra-core/src/error.rs`
- Create: `crates/zydra-core/src/model/mod.rs`
- Create: `crates/zydra-core/src/model/charset.rs`
- Modify: `crates/zydra-core/src/lib.rs`

**Interfaces:**
- Consumes: tidak ada.
- Produces:
  - `constants::{DEFAULT_BATCH_SIZE: usize, DEFAULT_PROGRESS_INTERVAL_MS: u64, DEFAULT_MAX_COMBINATIONS: u64, DEFAULT_MAX_BRUTE_LENGTH: u8, SPEED_SMOOTHING_FACTOR: f32, SESSION_SAVE_INTERVAL_SECS: u64, DEFAULT_LOG_CAPACITY: usize, DEFAULT_ZIP_VERIFY_BYTES: usize, DEFAULT_WORKERS: usize, MIN_WORKERS: usize, WORDLIST_READ_BUFFER_BYTES: usize}`
  - `error::CoreError`
  - `model::charset::{CharsetSet, LOWERCASE_CHARS, UPPERCASE_CHARS, DIGIT_CHARS, SYMBOL_CHARS, SPACE_CHARS}`
  - `CharsetSet::alphabet(self) -> String`
  - `CharsetSet::total_combinations(self, min: u8, max: u8) -> u64`
  - `CharsetSet::parse(spec: &str) -> Result<CharsetSet, CoreError>`
  - `CharsetSet::is_empty(self) -> bool` (dari `bitflags`)

- [ ] **Step 1: Tulis tes yang gagal**

Buat `crates/zydra-core/src/model/charset.rs` dengan hanya blok tes di bawah, plus `use super::*;`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercase_alphabet_is_the_26_letters_in_order() {
        assert_eq!(CharsetSet::LOWERCASE.alphabet(), "abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn combined_alphabet_has_no_duplicate_characters() {
        let alphabet = (CharsetSet::LOWERCASE | CharsetSet::UPPERCASE | CharsetSet::DIGITS
            | CharsetSet::SYMBOLS | CharsetSet::SPACE)
            .alphabet();
        let mut chars: Vec<char> = alphabet.chars().collect();
        let before = chars.len();
        chars.sort_unstable();
        chars.dedup();
        assert_eq!(chars.len(), before);
    }

    #[test]
    fn combinations_for_one_length_is_the_alphabet_size() {
        assert_eq!(CharsetSet::LOWERCASE.total_combinations(1, 1), 26);
    }

    #[test]
    fn combinations_sum_over_every_length() {
        let set = CharsetSet::LOWERCASE | CharsetSet::DIGITS;
        assert_eq!(set.total_combinations(1, 2), 36 + 36 * 36);
    }

    #[test]
    fn combinations_saturate_instead_of_wrapping() {
        let set = CharsetSet::LOWERCASE | CharsetSet::UPPERCASE | CharsetSet::DIGITS
            | CharsetSet::SYMBOLS | CharsetSet::SPACE;
        assert_eq!(set.total_combinations(1, 60), u64::MAX);
    }

    #[test]
    fn combinations_of_an_empty_set_is_zero() {
        assert_eq!(CharsetSet::empty().total_combinations(1, 4), 0);
    }

    #[test]
    fn combinations_of_an_inverted_range_is_zero() {
        assert_eq!(CharsetSet::LOWERCASE.total_combinations(4, 2), 0);
    }

    #[test]
    fn parse_accepts_every_canonical_class_name() {
        let set = CharsetSet::parse("lowercase,uppercase,digits,symbols,space").unwrap();
        assert_eq!(
            set,
            CharsetSet::LOWERCASE
                | CharsetSet::UPPERCASE
                | CharsetSet::DIGITS
                | CharsetSet::SYMBOLS
                | CharsetSet::SPACE
        );
    }

    #[test]
    fn parse_is_order_independent_and_tolerates_whitespace() {
        assert_eq!(
            CharsetSet::parse("digits, lowercase").unwrap(),
            CharsetSet::parse("lowercase,digits").unwrap()
        );
    }

    #[test]
    fn parse_rejects_an_unknown_class_and_names_it() {
        let error = CharsetSet::parse("lowercase,letters").unwrap_err();
        assert!(matches!(error, CoreError::CharsetParse { ref name } if name == "letters"));
    }

    #[test]
    fn parse_rejects_an_empty_specification() {
        assert!(CharsetSet::parse("").is_err());
        assert!(CharsetSet::parse(",,,").is_err());
    }
}
```

- [ ] **Step 2: Jalankan tes untuk memastikan gagal**

Run: `cargo test -p zydra-core charset`
Expected: gagal kompilasi dengan `cannot find type CharsetSet in this scope`.

- [ ] **Step 3: Tulis implementasi minimal**

`crates/zydra-core/src/constants.rs`:

```rust
//! Setiap angka ajaib di crate ini. Tidak ada angka telanjang di modul lain.

/// Jumlah kandidat yang diambil satu worker dalam sekali kunci.
pub const DEFAULT_BATCH_SIZE: usize = 64;

/// Jarak antar peristiwa progres, dalam milidetik.
pub const DEFAULT_PROGRESS_INTERVAL_MS: u64 = 350;

/// Batas jumlah kombinasi sebelum serangan ditolak.
pub const DEFAULT_MAX_COMBINATIONS: u64 = 20_000_000;

/// Panjang maksimum yang diizinkan untuk serangan brute force.
pub const DEFAULT_MAX_BRUTE_LENGTH: u8 = 6;

/// Bobot sampel terbaru saat menghitung kecepatan rata-rata bergerak.
pub const SPEED_SMOOTHING_FACTOR: f32 = 0.3;

/// Jarak antar penyimpanan posisi sesi, dalam detik.
pub const SESSION_SAVE_INTERVAL_SECS: u64 = 5;

/// Jumlah maksimum entri log yang disimpan di memori.
pub const DEFAULT_LOG_CAPACITY: usize = 5_000;

/// Jumlah byte entri terenkripsi yang dibaca untuk memverifikasi kandidat ZIP.
pub const DEFAULT_ZIP_VERIFY_BYTES: usize = 4_096;

/// Nilai `workers` yang berarti "ikuti jumlah CPU".
pub const DEFAULT_WORKERS: usize = 0;

/// Jumlah worker minimum yang selalu dibuat.
pub const MIN_WORKERS: usize = 1;

/// Ukuran buffer pembaca wordlist.
pub const WORDLIST_READ_BUFFER_BYTES: usize = 64 * 1024;
```

`crates/zydra-core/src/error.rs`:

```rust
//! Kesalahan prasarana. Kesalahan yang dipicu masukan pengguna ada di
//! [`crate::model::failure::CrackFailure`].
//!
//! Pesan di sini bersifat teknis dan ditujukan untuk catatan. Lapisan antarmuka selalu
//! memetakannya ke kunci terjemahan, bukan menampilkan pesan ini apa adanya.

use thiserror::Error;

/// Kesalahan yang bukan disebabkan masukan pengguna.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Operasi berkas gagal.
    #[error("io: {source}")]
    Io {
        /// Kesalahan asli dari sistem.
        #[from]
        source: std::io::Error,
    },

    /// Spesifikasi kelas karakter tidak dikenal.
    #[error("unknown charset class: {name}")]
    CharsetParse {
        /// Nama kelas yang tidak dikenal.
        name: String,
    },

    /// Nilai konfigurasi tidak masuk akal.
    #[error("invalid configuration: {detail}")]
    ConfigInvalid {
        /// Penjelasan teknis.
        detail: String,
    },

    /// Berkas sesi rusak atau tidak bisa dibaca.
    #[error("session problem: {detail}")]
    Session {
        /// Penjelasan teknis.
        detail: String,
    },
}
```

`crates/zydra-core/src/model/mod.rs`:

```rust
//! Tipe domain. Tidak ada teks yang ditampilkan kepada pengguna di sini.

pub mod charset;
```

`crates/zydra-core/src/model/charset.rs` (implementasi, di atas blok tes):

```rust
//! Kelas karakter untuk serangan brute force.

use bitflags::bitflags;

use crate::error::CoreError;

/// Huruf kecil.
pub const LOWERCASE_CHARS: &str = "abcdefghijklmnopqrstuvwxyz";

/// Huruf besar.
pub const UPPERCASE_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Angka.
pub const DIGIT_CHARS: &str = "0123456789";

/// Tanda baca.
pub const SYMBOL_CHARS: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// Spasi.
pub const SPACE_CHARS: &str = " ";

bitflags! {
    /// Himpunan kelas karakter. Nama kelas adalah konstanta di dalam tipe ini; tidak ada
    /// enum terpisah untuk konsep yang sama.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CharsetSet: u8 {
        /// Huruf kecil.
        const LOWERCASE = 0b0_0001;
        /// Huruf besar.
        const UPPERCASE = 0b0_0010;
        /// Angka.
        const DIGITS    = 0b0_0100;
        /// Tanda baca.
        const SYMBOLS   = 0b0_1000;
        /// Spasi.
        const SPACE     = 0b1_0000;
    }
}

impl CharsetSet {
    /// Menyusun alfabet dalam urutan tetap: huruf kecil, huruf besar, angka, tanda baca,
    /// spasi.
    pub fn alphabet(self) -> String {
        let mut out = String::new();
        if self.contains(CharsetSet::LOWERCASE) {
            out.push_str(LOWERCASE_CHARS);
        }
        if self.contains(CharsetSet::UPPERCASE) {
            out.push_str(UPPERCASE_CHARS);
        }
        if self.contains(CharsetSet::DIGITS) {
            out.push_str(DIGIT_CHARS);
        }
        if self.contains(CharsetSet::SYMBOLS) {
            out.push_str(SYMBOL_CHARS);
        }
        if self.contains(CharsetSet::SPACE) {
            out.push_str(SPACE_CHARS);
        }
        out
    }

    /// Jumlah kandidat untuk panjang `min` sampai `max`. Menjenuh ke `u64::MAX` saat
    /// melimpah, tidak membungkus.
    pub fn total_combinations(self, min: u8, max: u8) -> u64 {
        let base = self.alphabet().chars().count() as u64;
        if base == 0 || max < min {
            return 0;
        }
        let mut total: u64 = 0;
        for length in min..=max {
            let mut power: u64 = 1;
            for _ in 0..length {
                match power.checked_mul(base) {
                    Some(value) => power = value,
                    None => return u64::MAX,
                }
            }
            match total.checked_add(power) {
                Some(value) => total = value,
                None => return u64::MAX,
            }
        }
        total
    }

    /// Mengurai daftar nama kelas yang dipisah koma. Segmen kosong diabaikan.
    pub fn parse(spec: &str) -> Result<CharsetSet, CoreError> {
        let mut set = CharsetSet::empty();
        for raw in spec.split(',') {
            let name = raw.trim();
            if name.is_empty() {
                continue;
            }
            match name {
                "lowercase" => set |= CharsetSet::LOWERCASE,
                "uppercase" => set |= CharsetSet::UPPERCASE,
                "digits" => set |= CharsetSet::DIGITS,
                "symbols" => set |= CharsetSet::SYMBOLS,
                "space" => set |= CharsetSet::SPACE,
                other => {
                    return Err(CoreError::CharsetParse {
                        name: other.to_string(),
                    })
                }
            }
        }
        if set.is_empty() {
            return Err(CoreError::CharsetParse {
                name: spec.to_string(),
            });
        }
        Ok(set)
    }
}
```

Perbarui `crates/zydra-core/src/lib.rs`:

```rust
//! Logika pemulihan password untuk ZIP, RAR, PDF, dan berkas `shadow` Unix.
//!
//! Crate ini tidak memuat dependensi antarmuka pengguna dan tidak memuat kalimat yang
//! ditampilkan kepada pengguna. Kesalahan dikembalikan sebagai tipe; penerjemahan menjadi
//! teks adalah tanggung jawab crate `zydra-i18n`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod constants;
pub mod error;
pub mod model;
```

- [ ] **Step 4: Jalankan tes untuk memastikan lolos**

Run: `cargo test -p zydra-core charset`
Expected: `test result: ok. 11 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/zydra-core
git commit -m "feat(model): teach the core its alphabet"
```

---

### Task 3: `model::target`

**Files:**
- Create: `crates/zydra-core/src/model/target.rs`
- Modify: `crates/zydra-core/src/model/mod.rs`

**Interfaces:**
- Consumes: tidak ada.
- Produces:
  - `model::target::TargetKind` dengan varian `Zip`, `Rar`, `Pdf`, `Shadow`, `Unsupported`
  - `TargetKind::detect(path: &Path) -> TargetKind`
  - `TargetKind::as_str(self) -> &'static str`
  - `model::target::CrackTarget { kind: TargetKind, path: PathBuf, display_name: String }`
  - `CrackTarget::from_path(path: impl Into<PathBuf>) -> CrackTarget`

- [ ] **Step 1: Tulis tes yang gagal**

`crates/zydra-core/src/model/target.rs`, hanya blok tes:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_zip_rar_and_pdf_by_extension() {
        assert_eq!(TargetKind::detect(Path::new("vault.zip")), TargetKind::Zip);
        assert_eq!(TargetKind::detect(Path::new("vault.rar")), TargetKind::Rar);
        assert_eq!(TargetKind::detect(Path::new("vault.pdf")), TargetKind::Pdf);
    }

    #[test]
    fn extension_matching_ignores_case() {
        assert_eq!(TargetKind::detect(Path::new("VAULT.ZIP")), TargetKind::Zip);
        assert_eq!(TargetKind::detect(Path::new("Vault.PdF")), TargetKind::Pdf);
    }

    #[test]
    fn detects_a_shadow_file_by_name_without_extension() {
        assert_eq!(TargetKind::detect(Path::new("/etc/shadow")), TargetKind::Shadow);
        assert_eq!(TargetKind::detect(Path::new("SHADOW")), TargetKind::Shadow);
        assert_eq!(TargetKind::detect(Path::new("shadow.bak")), TargetKind::Shadow);
    }

    #[test]
    fn a_known_extension_wins_over_the_shadow_name() {
        assert_eq!(TargetKind::detect(Path::new("shadow.zip")), TargetKind::Zip);
    }

    #[test]
    fn anything_else_is_unsupported() {
        assert_eq!(TargetKind::detect(Path::new("notes.txt")), TargetKind::Unsupported);
        assert_eq!(TargetKind::detect(Path::new("noext")), TargetKind::Unsupported);
    }

    #[test]
    fn from_path_keeps_the_file_name_as_the_display_name() {
        let target = CrackTarget::from_path("/tmp/vault.zip");
        assert_eq!(target.kind, TargetKind::Zip);
        assert_eq!(target.display_name, "vault.zip");
    }

    #[test]
    fn kind_as_str_is_stable_for_persistence() {
        assert_eq!(TargetKind::Zip.as_str(), "zip");
        assert_eq!(TargetKind::Shadow.as_str(), "shadow");
        assert_eq!(TargetKind::Unsupported.as_str(), "unsupported");
    }
}
```

- [ ] **Step 2: Jalankan tes untuk memastikan gagal**

Run: `cargo test -p zydra-core target`
Expected: gagal kompilasi dengan `cannot find type TargetKind`.

- [ ] **Step 3: Tulis implementasi minimal**

```rust
//! Jenis berkas target dan deskriptornya.

use std::path::{Path, PathBuf};

/// Jenis berkas yang bisa dicoba.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetKind {
    /// Arsip ZIP.
    Zip,
    /// Arsip RAR.
    Rar,
    /// Dokumen PDF.
    Pdf,
    /// Berkas `shadow` Unix.
    Shadow,
    /// Tidak dikenali.
    Unsupported,
}

impl TargetKind {
    /// Menentukan jenis dari nama berkas. Ekstensi diperiksa lebih dulu; baru kemudian
    /// nama yang memuat `shadow` tanpa membedakan huruf besar-kecil.
    pub fn detect(path: &Path) -> TargetKind {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let extension = name.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
        match extension {
            "zip" => TargetKind::Zip,
            "rar" => TargetKind::Rar,
            "pdf" => TargetKind::Pdf,
            _ if name.contains("shadow") => TargetKind::Shadow,
            _ => TargetKind::Unsupported,
        }
    }

    /// Identifier stabil untuk berkas sesi dan konfigurasi.
    pub fn as_str(self) -> &'static str {
        match self {
            TargetKind::Zip => "zip",
            TargetKind::Rar => "rar",
            TargetKind::Pdf => "pdf",
            TargetKind::Shadow => "shadow",
            TargetKind::Unsupported => "unsupported",
        }
    }
}

/// Berkas target beserta nama tampilannya.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrackTarget {
    /// Jenis berkas.
    pub kind: TargetKind,
    /// Lokasi berkas.
    pub path: PathBuf,
    /// Nama yang ditampilkan di antarmuka.
    pub display_name: String,
}

impl CrackTarget {
    /// Membuat target dari sebuah path. Nama berkas dipakai sebagai nama tampilan; kalau
    /// path tidak punya nama berkas, seluruh path dipakai.
    pub fn from_path(path: impl Into<PathBuf>) -> CrackTarget {
        let path = path.into();
        let kind = TargetKind::detect(&path);
        let display_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        CrackTarget {
            kind,
            path,
            display_name,
        }
    }
}
```

Tambahkan `pub mod target;` ke `crates/zydra-core/src/model/mod.rs`.

- [ ] **Step 4: Jalankan tes untuk memastikan lolos**

Run: `cargo test -p zydra-core target`
Expected: `test result: ok. 7 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/zydra-core
git commit -m "feat(model): recognise a target without asking its extension twice"
```

---

### Task 4: `model::failure` dan `model::progress`

**Files:**
- Create: `crates/zydra-core/src/model/failure.rs`
- Create: `crates/zydra-core/src/model/progress.rs`
- Modify: `crates/zydra-core/src/model/mod.rs`

**Interfaces:**
- Consumes: tidak ada.
- Produces:
  - `model::failure::Backend` (`Zip`, `Rar`, `Pdf`, `Shadow`)
  - `model::failure::CrackFailure` (varian sesuai spec bagian 6.1, tanpa `Rar5NotSupported`)
  - `model::progress::TryingData { tried: u64, total: u64, speed_per_second: u64, elapsed: Duration, eta: Option<Duration> }`
  - `TryingData::fraction(&self) -> f32`
  - `model::progress::CrackProgress` (`Trying(TryingData)`, `Found { password, detail }`, `Failed { failure }`)

- [ ] **Step 1: Tulis tes yang gagal**

`crates/zydra-core/src/model/progress.rs`, hanya blok tes:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn data(tried: u64, total: u64) -> TryingData {
        TryingData {
            tried,
            total,
            speed_per_second: 0,
            elapsed: Duration::from_secs(1),
            eta: None,
        }
    }

    #[test]
    fn fraction_is_the_ratio_of_tried_to_total() {
        assert_eq!(data(50, 100).fraction(), 0.5);
    }

    #[test]
    fn fraction_clamps_when_tried_exceeds_total() {
        assert_eq!(data(120, 100).fraction(), 1.0);
    }

    #[test]
    fn fraction_of_a_zero_total_is_zero_and_never_nan() {
        let value = data(0, 0).fraction();
        assert_eq!(value, 0.0);
        assert!(!value.is_nan());
    }

    #[test]
    fn fraction_of_nothing_tried_is_zero() {
        assert_eq!(data(0, 100).fraction(), 0.0);
    }
}
```

- [ ] **Step 2: Jalankan tes untuk memastikan gagal**

Run: `cargo test -p zydra-core progress`
Expected: gagal kompilasi dengan `cannot find type TryingData`.

- [ ] **Step 3: Tulis implementasi minimal**

`crates/zydra-core/src/model/failure.rs`:

```rust
//! Kesalahan yang disebabkan masukan pengguna atau keadaan target.
//!
//! Kesalahan prasarana ada di [`crate::error::CoreError`].

/// Backend format yang mungkin tidak tersedia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backend {
    /// Backend ZIP.
    Zip,
    /// Backend RAR.
    Rar,
    /// Backend PDF.
    Pdf,
    /// Backend berkas `shadow`.
    Shadow,
}

/// Sebab sebuah upaya pemulihan berhenti tanpa menemukan password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrackFailure {
    /// Semua kandidat sudah dicoba dan tidak ada yang cocok.
    PasswordNotFound,
    /// Jenis berkas tidak dikenali.
    UnsupportedFormat {
        /// Nama berkas yang dipakai pengguna.
        display_name: String,
    },
    /// Wordlist tidak memuat satu kandidat pun.
    EmptyWordlist,
    /// Berkas target tidak bisa dibaca.
    FileUnreadable {
        /// Penjelasan teknis.
        reason: String,
    },
    /// Tidak ada kelas karakter yang dipilih.
    NoCharsetSelected,
    /// Rentang panjang tidak masuk akal.
    InvalidLengthRange {
        /// Panjang minimum yang diminta.
        min: u8,
        /// Panjang maksimum yang diminta.
        max: u8,
        /// Panjang maksimum yang diizinkan konfigurasi.
        allowed_max: u8,
    },
    /// Pola mask tidak bisa diurai.
    InvalidMask {
        /// Pola yang dikirim pengguna.
        pattern: String,
        /// Penjelasan teknis.
        reason: String,
    },
    /// Aturan mangling tidak dikenal.
    InvalidRule {
        /// Aturan yang dikirim pengguna.
        rule: String,
        /// Penjelasan teknis.
        reason: String,
    },
    /// Jumlah kombinasi melampaui batas yang diizinkan.
    TooManyCombinations {
        /// Jumlah yang dihitung.
        total: u64,
        /// Batas yang berlaku.
        limit: u64,
    },
    /// Tidak ada kandidat yang bisa dicoba.
    NoCandidate,
    /// Backend untuk format ini tidak tersedia pada build ini.
    BackendUnavailable {
        /// Backend yang dimaksud.
        backend: Backend,
        /// Penjelasan teknis.
        reason: String,
    },
    /// Berkas sesi tidak cocok dengan berkas target yang sedang dipilih.
    SessionFingerprintMismatch,
}
```

`crates/zydra-core/src/model/progress.rs`:

```rust
//! Kemajuan sebuah upaya pemulihan.

use std::time::Duration;

use crate::model::failure::CrackFailure;

/// Cuplikan keadaan saat upaya sedang berjalan.
///
/// Tipe ini sengaja terpisah dari [`CrackProgress`] supaya keadaan "sedang berjalan" di
/// antarmuka tidak mungkin memuat hasil akhir.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TryingData {
    /// Jumlah kandidat yang sudah dicoba.
    pub tried: u64,
    /// Jumlah kandidat yang akan dicoba.
    pub total: u64,
    /// Kecepatan rata-rata bergerak, kandidat per detik.
    pub speed_per_second: u64,
    /// Waktu sejak upaya dimulai.
    pub elapsed: Duration,
    /// Perkiraan waktu tersisa. `None` selama kecepatan belum bisa diukur.
    pub eta: Option<Duration>,
}

impl TryingData {
    /// Bagian yang sudah selesai, dijepit ke `0.0..=1.0`. Total nol menghasilkan `0.0`,
    /// bukan `NaN`.
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.tried as f64 / self.total as f64).clamp(0.0, 1.0) as f32
    }
}

/// Hasil satu langkah upaya pemulihan.
#[derive(Debug, Clone, PartialEq)]
pub enum CrackProgress {
    /// Upaya sedang berjalan.
    Trying(TryingData),
    /// Password ditemukan.
    Found {
        /// Password yang cocok.
        password: String,
        /// Keterangan tambahan dari backend, misalnya nama pengguna pada berkas `shadow`.
        detail: Option<String>,
    },
    /// Upaya berhenti tanpa hasil.
    Failed {
        /// Sebabnya.
        failure: CrackFailure,
    },
}
```

Tambahkan `pub mod failure;` dan `pub mod progress;` ke `crates/zydra-core/src/model/mod.rs`.

- [ ] **Step 4: Jalankan tes untuk memastikan lolos**

Run: `cargo test -p zydra-core progress`
Expected: `test result: ok. 4 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/zydra-core
git commit -m "feat(model): say how far along we are without dividing by zero"
```

---

### Task 5: Trait `CandidateSource` dan `WordlistSource`

**Files:**
- Create: `crates/zydra-core/src/candidate/mod.rs`
- Create: `crates/zydra-core/src/candidate/wordlist.rs`
- Modify: `crates/zydra-core/src/lib.rs`

**Interfaces:**
- Consumes: `error::CoreError`, `constants::WORDLIST_READ_BUFFER_BYTES`.
- Produces:
  - `candidate::CandidateSource` dengan `next_candidate(&mut self) -> Option<String>`, `total(&self) -> u64`, `position(&self) -> u64`, `skip(&mut self, count: u64)` (punya implementasi bawaan).
  - `candidate::wordlist::WordlistSource::open(path: &Path) -> Result<WordlistSource, CoreError>`

- [ ] **Step 1: Tulis tes yang gagal**

`crates/zydra-core/src/candidate/wordlist.rs`, hanya blok tes:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(bytes).unwrap();
        file.flush().unwrap();
        file
    }

    fn drain(source: &mut WordlistSource) -> Vec<String> {
        let mut out = Vec::new();
        while let Some(candidate) = source.next_candidate() {
            out.push(candidate);
        }
        out
    }

    #[test]
    fn reads_lines_in_order() {
        let file = write_temp(b"alpha\nbravo\ncharlie\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(drain(&mut source), vec!["alpha", "bravo", "charlie"]);
    }

    #[test]
    fn strips_carriage_returns() {
        let file = write_temp(b"alpha\r\nbravo\r\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(drain(&mut source), vec!["alpha", "bravo"]);
    }

    #[test]
    fn a_last_line_without_a_newline_is_still_a_candidate() {
        let file = write_temp(b"alpha\nbravo");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(drain(&mut source), vec!["alpha", "bravo"]);
    }

    #[test]
    fn empty_lines_are_skipped_and_not_counted() {
        let file = write_temp(b"alpha\n\n\nbravo\n\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(source.total(), 2);
        assert_eq!(drain(&mut source), vec!["alpha", "bravo"]);
    }

    #[test]
    fn a_line_of_only_spaces_is_a_real_candidate() {
        let file = write_temp(b"alpha\n \n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(drain(&mut source), vec!["alpha", " "]);
    }

    #[test]
    fn invalid_utf8_is_replaced_instead_of_dropping_the_line() {
        let file = write_temp(b"alp\xffha\nbravo\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        let candidates = drain(&mut source);
        assert_eq!(candidates.len(), 2);
        assert!(candidates[0].starts_with("alp"));
        assert!(candidates[0].ends_with("ha"));
        assert_eq!(candidates[1], "bravo");
    }

    #[test]
    fn total_matches_the_number_of_candidates_actually_yielded() {
        let file = write_temp(b"one\ntwo\n\nthree\nfour\r\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        let total = source.total();
        assert_eq!(total, drain(&mut source).len() as u64);
    }

    #[test]
    fn position_counts_yielded_candidates() {
        let file = write_temp(b"one\ntwo\nthree\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(source.position(), 0);
        source.next_candidate();
        source.next_candidate();
        assert_eq!(source.position(), 2);
    }

    #[test]
    fn skip_lands_on_the_next_candidate() {
        let file = write_temp(b"one\ntwo\nthree\nfour\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        source.skip(2);
        assert_eq!(source.position(), 2);
        assert_eq!(source.next_candidate().as_deref(), Some("three"));
    }

    #[test]
    fn skip_counts_only_real_candidates() {
        let file = write_temp(b"one\n\ntwo\n\nthree\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        source.skip(1);
        assert_eq!(source.next_candidate().as_deref(), Some("two"));
    }

    #[test]
    fn skipping_past_the_end_yields_nothing_and_does_not_panic() {
        let file = write_temp(b"one\ntwo\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        source.skip(99);
        assert_eq!(source.next_candidate(), None);
        assert_eq!(source.next_candidate(), None);
    }

    #[test]
    fn opening_a_missing_file_is_an_error() {
        assert!(WordlistSource::open(std::path::Path::new("no-such-file-here.txt")).is_err());
    }
}
```

- [ ] **Step 2: Jalankan tes untuk memastikan gagal**

Run: `cargo test -p zydra-core wordlist`
Expected: gagal kompilasi dengan `cannot find type WordlistSource`.

- [ ] **Step 3: Tulis implementasi minimal**

`crates/zydra-core/src/candidate/mod.rs`:

```rust
//! Sumber kandidat password. Mesin tidak peduli dari mana kandidat berasal.

pub mod wordlist;

/// Deretan kandidat password.
///
/// Implementasi harus bisa dipindahkan ke thread lain. Urutan kandidat harus stabil
/// supaya [`CandidateSource::skip`] bisa dipakai untuk melanjutkan serangan.
pub trait CandidateSource: Send {
    /// Kandidat berikutnya, atau `None` kalau sudah habis.
    fn next_candidate(&mut self) -> Option<String>;

    /// Jumlah kandidat yang akan dihasilkan. Boleh menjenuh pada `u64::MAX`.
    fn total(&self) -> u64;

    /// Jumlah kandidat yang sudah dihasilkan.
    fn position(&self) -> u64;

    /// Melewati `count` kandidat tanpa mengembalikannya.
    ///
    /// Implementasi bawaan ini selalu benar tetapi bisa lambat. Sumber yang bisa
    /// menghitung posisinya secara langsung sebaiknya menggantinya.
    fn skip(&mut self, count: u64) {
        let mut remaining = count;
        while remaining > 0 {
            if self.next_candidate().is_none() {
                break;
            }
            remaining -= 1;
        }
    }
}
```

`crates/zydra-core/src/candidate/wordlist.rs`:

```rust
//! Sumber kandidat dari berkas wordlist.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::candidate::CandidateSource;
use crate::constants::WORDLIST_READ_BUFFER_BYTES;
use crate::error::CoreError;

/// Memotong akhir baris dari potongan byte. Hanya `\n` dan `\r` yang dibuang; spasi
/// dipertahankan supaya password berspasi tetap bisa dicoba.
fn strip_eol(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && (bytes[end - 1] == b'\n' || bytes[end - 1] == b'\r') {
        end -= 1;
    }
    &bytes[..end]
}

/// Membaca kandidat satu per satu dari sebuah berkas teks.
pub struct WordlistSource {
    reader: BufReader<File>,
    buffer: Vec<u8>,
    total: u64,
    position: u64,
    finished: bool,
}

impl WordlistSource {
    /// Membuka wordlist dan menghitung jumlah kandidatnya sekali di awal.
    ///
    /// Baris yang kosong setelah akhir baris dipotong tidak dihitung dan tidak
    /// dihasilkan.
    pub fn open(path: &Path) -> Result<WordlistSource, CoreError> {
        let total = count_candidates(path)?;
        let file = File::open(path)?;
        Ok(WordlistSource {
            reader: BufReader::with_capacity(WORDLIST_READ_BUFFER_BYTES, file),
            buffer: Vec::new(),
            total,
            position: 0,
            finished: false,
        })
    }

    /// Membaca potongan berikutnya ke buffer. `false` berarti berkas sudah habis atau
    /// tidak bisa dibaca lagi.
    fn read_chunk(&mut self) -> bool {
        if self.finished {
            return false;
        }
        self.buffer.clear();
        match self.reader.read_until(b'\n', &mut self.buffer) {
            Ok(0) | Err(_) => {
                self.finished = true;
                false
            }
            Ok(_) => true,
        }
    }
}

fn count_candidates(path: &Path) -> Result<u64, CoreError> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(WORDLIST_READ_BUFFER_BYTES, file);
    let mut buffer = Vec::new();
    let mut count = 0u64;
    loop {
        buffer.clear();
        match reader.read_until(b'\n', &mut buffer)? {
            0 => break,
            _ => {
                if !strip_eol(&buffer).is_empty() {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

impl CandidateSource for WordlistSource {
    fn next_candidate(&mut self) -> Option<String> {
        loop {
            if !self.read_chunk() {
                return None;
            }
            let line = strip_eol(&self.buffer);
            if line.is_empty() {
                continue;
            }
            self.position += 1;
            return Some(String::from_utf8_lossy(line).into_owned());
        }
    }

    fn total(&self) -> u64 {
        self.total
    }

    fn position(&self) -> u64 {
        self.position
    }

    fn skip(&mut self, count: u64) {
        let mut remaining = count;
        while remaining > 0 {
            if !self.read_chunk() {
                return;
            }
            if strip_eol(&self.buffer).is_empty() {
                continue;
            }
            self.position += 1;
            remaining -= 1;
        }
    }
}
```

Tambahkan `pub mod candidate;` ke `crates/zydra-core/src/lib.rs`.

- [ ] **Step 4: Jalankan tes untuk memastikan lolos**

Run: `cargo test -p zydra-core wordlist`
Expected: `test result: ok. 12 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/zydra-core
git commit -m "feat(candidate): read a wordlist without hoarding it in memory"
```

---

### Task 6: `BruteForceSource`

**Files:**
- Create: `crates/zydra-core/src/candidate/bruteforce.rs`
- Modify: `crates/zydra-core/src/candidate/mod.rs`

**Interfaces:**
- Consumes: `candidate::CandidateSource`, `model::charset::CharsetSet`, `model::failure::CrackFailure`.
- Produces:
  - `candidate::bruteforce::BruteForceSource::new(charset: CharsetSet, min_length: u8, max_length: u8) -> Result<BruteForceSource, CrackFailure>`
  - implementasi `CandidateSource` untuk `BruteForceSource`, termasuk `skip` yang melompat langsung tanpa membangun string.

- [ ] **Step 1: Tulis tes yang gagal**

`crates/zydra-core/src/candidate/bruteforce.rs`, hanya blok tes:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn drain(source: &mut BruteForceSource) -> Vec<String> {
        let mut out = Vec::new();
        while let Some(candidate) = source.next_candidate() {
            out.push(candidate);
        }
        out
    }

    fn source(min: u8, max: u8) -> BruteForceSource {
        BruteForceSource::from_alphabet(vec!['a', 'b'], min, max).unwrap()
    }

    #[test]
    fn single_length_walks_the_alphabet() {
        assert_eq!(drain(&mut source(1, 1)), vec!["a", "b"]);
    }

    #[test]
    fn longer_lengths_follow_shorter_ones() {
        assert_eq!(
            drain(&mut source(1, 2)),
            vec!["a", "b", "aa", "ab", "ba", "bb"]
        );
    }

    #[test]
    fn total_matches_the_number_of_candidates_yielded() {
        let mut brute = source(1, 3);
        let total = brute.total();
        assert_eq!(total, drain(&mut brute).len() as u64);
    }

    #[test]
    fn total_uses_the_charset_arithmetic() {
        let brute = BruteForceSource::new(CharsetSet::LOWERCASE, 1, 2).unwrap();
        assert_eq!(brute.total(), 26 + 26 * 26);
    }

    #[test]
    fn position_counts_yielded_candidates() {
        let mut brute = source(1, 2);
        assert_eq!(brute.position(), 0);
        brute.next_candidate();
        brute.next_candidate();
        assert_eq!(brute.position(), 2);
    }

    #[test]
    fn skip_lands_on_the_next_candidate() {
        let mut brute = source(1, 2);
        brute.skip(2);
        assert_eq!(brute.position(), 2);
        assert_eq!(brute.next_candidate().as_deref(), Some("aa"));
    }

    #[test]
    fn skip_one_moves_a_single_step() {
        let mut brute = source(1, 2);
        brute.skip(1);
        assert_eq!(brute.next_candidate().as_deref(), Some("b"));
    }

    #[test]
    fn skipping_the_whole_space_yields_nothing() {
        let mut brute = source(1, 2);
        brute.skip(6);
        assert_eq!(brute.next_candidate(), None);
    }

    #[test]
    fn skipping_past_the_end_yields_nothing_and_does_not_panic() {
        let mut brute = source(1, 2);
        brute.skip(1_000);
        assert_eq!(brute.next_candidate(), None);
    }

    #[test]
    fn skipping_zero_changes_nothing() {
        let mut brute = source(1, 2);
        brute.skip(0);
        assert_eq!(brute.next_candidate().as_deref(), Some("a"));
    }

    #[test]
    fn an_empty_charset_is_rejected() {
        assert_eq!(
            BruteForceSource::new(CharsetSet::empty(), 1, 2).unwrap_err(),
            CrackFailure::NoCharsetSelected
        );
    }

    #[test]
    fn a_zero_minimum_is_rejected() {
        assert!(matches!(
            BruteForceSource::new(CharsetSet::LOWERCASE, 0, 2).unwrap_err(),
            CrackFailure::InvalidLengthRange { .. }
        ));
    }

    #[test]
    fn a_maximum_below_the_minimum_is_rejected() {
        assert!(matches!(
            BruteForceSource::new(CharsetSet::LOWERCASE, 3, 2).unwrap_err(),
            CrackFailure::InvalidLengthRange { .. }
        ));
    }
}
```

- [ ] **Step 2: Jalankan tes untuk memastikan gagal**

Run: `cargo test -p zydra-core bruteforce`
Expected: gagal kompilasi dengan `cannot find type BruteForceSource`.

- [ ] **Step 3: Tulis implementasi minimal**

```rust
//! Sumber kandidat hasil penelusuran seluruh kombinasi karakter.

use crate::candidate::CandidateSource;
use crate::model::charset::CharsetSet;
use crate::model::failure::CrackFailure;

/// Menghasilkan setiap kombinasi dari sebuah alfabet untuk panjang `min_length` sampai
/// `max_length`, tanpa menyimpan seluruh ruang ke memori.
#[derive(Debug)]
pub struct BruteForceSource {
    alphabet: Vec<char>,
    min_length: u8,
    max_length: u8,
    indices: Vec<usize>,
    current_length: u8,
    started: bool,
    emitted: u64,
    total: u64,
}

impl BruteForceSource {
    /// Membuat sumber dari sebuah himpunan kelas karakter.
    pub fn new(
        charset: CharsetSet,
        min_length: u8,
        max_length: u8,
    ) -> Result<BruteForceSource, CrackFailure> {
        Self::from_alphabet(charset.alphabet().chars().collect(), min_length, max_length)
    }

    /// Membuat sumber dari alfabet yang sudah jadi. Dipakai oleh pengujian dan oleh
    /// sumber mask.
    pub fn from_alphabet(
        alphabet: Vec<char>,
        min_length: u8,
        max_length: u8,
    ) -> Result<BruteForceSource, CrackFailure> {
        if alphabet.is_empty() {
            return Err(CrackFailure::NoCharsetSelected);
        }
        if min_length == 0 || max_length < min_length {
            return Err(CrackFailure::InvalidLengthRange {
                min: min_length,
                max: max_length,
                allowed_max: max_length,
            });
        }
        let base = alphabet.len() as u64;
        let mut total: u64 = 0;
        for length in min_length..=max_length {
            let mut power: u64 = 1;
            for _ in 0..length {
                match power.checked_mul(base) {
                    Some(value) => power = value,
                    None => {
                        total = u64::MAX;
                        break;
                    }
                }
            }
            match total.checked_add(power) {
                Some(value) => total = value,
                None => {
                    total = u64::MAX;
                    break;
                }
            }
        }
        Ok(BruteForceSource {
            alphabet,
            min_length,
            max_length,
            indices: Vec::new(),
            current_length: min_length,
            started: false,
            emitted: 0,
            total,
        })
    }

    /// Memajukan odometer satu langkah. `false` berarti seluruh ruang sudah habis.
    fn advance(&mut self) -> bool {
        let mut position = self.indices.len();
        loop {
            if position == 0 {
                if self.current_length >= self.max_length {
                    return false;
                }
                self.current_length += 1;
                self.indices = vec![0; self.current_length as usize];
                return true;
            }
            position -= 1;
            self.indices[position] += 1;
            if self.indices[position] < self.alphabet.len() {
                return true;
            }
            self.indices[position] = 0;
        }
    }

    /// Menempatkan odometer tepat **sebelum** kandidat ke-`index`, sehingga panggilan
    /// `next_candidate` berikutnya menghasilkan kandidat ke-`index`.
    ///
    /// `next_candidate` selalu memajukan odometer lebih dulu, jadi menempatkannya
    /// *pada* `index` akan menghasilkan kandidat `index + 1`. Karena itu posisinya
    /// adalah pendahulu `index - 1`.
    ///
    /// `index` dijepit ke `total` supaya melompat melewati akhir ruang menghasilkan
    /// `None` pada panggilan berikutnya, bukan panik.
    fn seek(&mut self, index: u64) {
        if index == 0 {
            self.started = false;
            self.emitted = 0;
            self.indices.clear();
            self.current_length = self.min_length;
            return;
        }
        let base = self.alphabet.len() as u64;
        let target = index.min(self.total);
        let mut remaining = target - 1;
        let mut length = self.min_length;
        loop {
            let block = base.saturating_pow(length as u32);
            if remaining < block || length >= self.max_length {
                let mut digits = vec![0usize; length as usize];
                let mut value = remaining;
                let mut position = length as usize;
                while position > 0 {
                    position -= 1;
                    digits[position] = (value % base) as usize;
                    value /= base;
                }
                self.current_length = length;
                self.indices = digits;
                self.started = true;
                self.emitted = target;
                return;
            }
            remaining -= block;
            length += 1;
        }
    }
}

impl CandidateSource for BruteForceSource {
    fn next_candidate(&mut self) -> Option<String> {
        if self.started {
            if !self.advance() {
                return None;
            }
        } else {
            self.started = true;
            self.current_length = self.min_length;
            self.indices = vec![0; self.min_length as usize];
        }
        self.emitted += 1;
        Some(
            self.indices
                .iter()
                .map(|&index| self.alphabet[index])
                .collect(),
        )
    }

    fn total(&self) -> u64 {
        self.total
    }

    fn position(&self) -> u64 {
        self.emitted
    }

    fn skip(&mut self, count: u64) {
        if count == 0 {
            return;
        }
        let target = self.emitted.saturating_add(count);
        self.seek(target);
    }
}
```

Tambahkan `pub mod bruteforce;` ke `crates/zydra-core/src/candidate/mod.rs`.

- [ ] **Step 4: Jalankan tes untuk memastikan lolos**

Run: `cargo test -p zydra-core bruteforce`
Expected: `test result: ok. 13 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/zydra-core
git commit -m "feat(candidate): walk every combination and jump straight to the Nth"
```

---

### Task 7: Trait tester

**Files:**
- Create: `crates/zydra-core/src/tester/mod.rs`
- Modify: `crates/zydra-core/src/lib.rs`

**Interfaces:**
- Consumes: `model::target::TargetKind`.
- Produces:
  - `tester::MatchOutcome` (`NoMatch`, `Matched { detail: Option<String> }`)
  - `tester::EncryptionKind` (`None`, `ZipCrypto`, `ZipAes256`, `RarCrypto4`, `RarCrypto5`, `PdfStandard`, `UnixCrypt(UnixCryptScheme)`)
  - `tester::UnixCryptScheme` (`Des`, `Md5`, `Sha256`, `Sha512`, `Bcrypt`, `Other(String)`)
  - `tester::TesterInfo { kind: TargetKind, encryption: EncryptionKind, entries: usize }`
  - `tester::PasswordTester` dengan `try_password(&mut self, password: &str) -> MatchOutcome`, `info(&self) -> &TesterInfo`
  - `tester::TesterFactory` dengan `create(&self, target: &CrackTarget) -> Result<Box<dyn PasswordTester>, CrackFailure>`

- [ ] **Step 1: Tulis tes yang gagal**

`crates/zydra-core/src/tester/mod.rs`, hanya blok tes:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::target::CrackTarget;

    struct AlwaysMatches {
        info: TesterInfo,
    }

    impl PasswordTester for AlwaysMatches {
        fn try_password(&mut self, _password: &str) -> MatchOutcome {
            MatchOutcome::Matched { detail: None }
        }
        fn info(&self) -> &TesterInfo {
            &self.info
        }
    }

    struct FixedFactory;

    impl TesterFactory for FixedFactory {
        fn create(&self, _target: &CrackTarget) -> Result<Box<dyn PasswordTester>, CrackFailure> {
            Ok(Box::new(AlwaysMatches {
                info: TesterInfo {
                    kind: TargetKind::Zip,
                    encryption: EncryptionKind::ZipCrypto,
                    entries: 1,
                },
            }))
        }
    }

    #[test]
    fn a_tester_reports_its_own_description() {
        let target = CrackTarget::from_path("vault.zip");
        let mut tester = FixedFactory.create(&target).unwrap();
        assert_eq!(tester.info().kind, TargetKind::Zip);
        assert_eq!(tester.info().encryption, EncryptionKind::ZipCrypto);
        assert_eq!(tester.info().entries, 1);
        assert_eq!(
            tester.try_password("anything"),
            MatchOutcome::Matched { detail: None }
        );
    }

    #[test]
    fn a_factory_can_be_shared_across_threads() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FixedFactory>();
    }

    #[test]
    fn a_tester_can_be_moved_to_another_thread() {
        fn assert_send<T: Send>() {}
        assert_send::<Box<dyn PasswordTester>>();
    }
}
```

- [ ] **Step 2: Jalankan tes untuk memastikan gagal**

Run: `cargo test -p zydra-core tester`
Expected: gagal kompilasi dengan `cannot find trait PasswordTester`.

- [ ] **Step 3: Tulis implementasi minimal**

```rust
//! Kontrak untuk menguji satu kandidat password terhadap sebuah target.
//!
//! Mesin pemecah hanya mengenal trait di modul ini. Implementasi nyata untuk tiap format
//! tinggal di modul `zip`, `rar`, `pdf`, dan `shadow` (rencana A2).

use crate::model::failure::CrackFailure;
use crate::model::target::{CrackTarget, TargetKind};

/// Hasil satu percobaan password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchOutcome {
    /// Kandidat tidak cocok.
    NoMatch,
    /// Kandidat cocok.
    Matched {
        /// Keterangan tambahan, misalnya nama pengguna pada berkas `shadow`.
        detail: Option<String>,
    },
}

/// Skema hash `crypt(3)` pada berkas `shadow`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnixCryptScheme {
    /// `descrypt`.
    Des,
    /// `md5crypt`.
    Md5,
    /// `sha256crypt`.
    Sha256,
    /// `sha512crypt`.
    Sha512,
    /// `bcrypt`.
    Bcrypt,
    /// Skema lain, disimpan apa adanya.
    Other(String),
}

/// Jenis enkripsi yang terdeteksi pada target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionKind {
    /// Berkas tidak terenkripsi.
    None,
    /// ZIP dengan ZipCrypto.
    ZipCrypto,
    /// ZIP dengan AES-256.
    ZipAes256,
    /// RAR4.
    RarCrypto4,
    /// RAR5.
    RarCrypto5,
    /// PDF standard security handler.
    PdfStandard,
    /// Hash `crypt(3)`.
    UnixCrypt(UnixCryptScheme),
}

/// Keterangan tentang target yang sedang diuji.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TesterInfo {
    /// Jenis berkas.
    pub kind: TargetKind,
    /// Enkripsi yang terdeteksi.
    pub encryption: EncryptionKind,
    /// Jumlah entri di dalam arsip, atau `1` untuk berkas tunggal.
    pub entries: usize,
}

/// Menguji satu kandidat password.
///
/// Setiap worker memegang pengujinya sendiri, sehingga `&mut self` aman dipakai tanpa
/// penguncian.
pub trait PasswordTester: Send {
    /// Mencoba satu password.
    fn try_password(&mut self, password: &str) -> MatchOutcome;

    /// Keterangan tentang target.
    fn info(&self) -> &TesterInfo;
}

/// Membuat penguji untuk sebuah target.
///
/// Dipanggil satu kali per worker. Implementasi bertanggung jawab melakukan pekerjaan
/// mahal, seperti membaca header arsip, satu kali saja dan membagikannya ke penguji
/// berikutnya.
pub trait TesterFactory: Send + Sync {
    /// Membuat penguji, atau mengembalikan sebab target tidak bisa diuji.
    fn create(&self, target: &CrackTarget) -> Result<Box<dyn PasswordTester>, CrackFailure>;
}
```

Tambahkan `pub mod tester;` ke `crates/zydra-core/src/lib.rs`.

- [ ] **Step 4: Jalankan tes untuk memastikan lolos**

Run: `cargo test -p zydra-core tester`
Expected: `test result: ok. 3 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/zydra-core
git commit -m "feat(tester): define the seam where formats plug in"
```

---

### Task 8: `engine::cancel` dan `engine::stats`

**Files:**
- Create: `crates/zydra-core/src/engine/mod.rs`
- Create: `crates/zydra-core/src/engine/cancel.rs`
- Create: `crates/zydra-core/src/engine/stats.rs`
- Modify: `crates/zydra-core/src/lib.rs`

**Interfaces:**
- Consumes: `constants::SPEED_SMOOTHING_FACTOR`, `model::progress::TryingData`.
- Produces:
  - `engine::cancel::CancelToken` dengan `new()`, `cancel(&self)`, `is_cancelled(&self) -> bool`, `Clone`, `Default`
  - `engine::stats::SpeedMeter` dengan `new(start: Instant) -> SpeedMeter` dan `sample(&mut self, tried: u64, total: u64, now: Instant) -> TryingData`

- [ ] **Step 1: Tulis tes yang gagal**

`crates/zydra-core/src/engine/cancel.rs`, hanya blok tes:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_token_is_not_cancelled() {
        assert!(!CancelToken::new().is_cancelled());
    }

    #[test]
    fn cancelling_is_visible_through_a_clone() {
        let token = CancelToken::new();
        let handle = token.clone();
        token.cancel();
        assert!(handle.is_cancelled());
    }

    #[test]
    fn the_default_token_is_not_cancelled() {
        assert!(!CancelToken::default().is_cancelled());
    }

    #[test]
    fn a_token_can_be_moved_to_another_thread() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CancelToken>();
    }
}
```

`crates/zydra-core/src/engine/stats.rs`, hanya blok tes:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn at(start: Instant, millis: u64) -> Instant {
        start + Duration::from_millis(millis)
    }

    #[test]
    fn the_first_sample_has_no_speed_and_no_estimate() {
        let start = Instant::now();
        let mut meter = SpeedMeter::new(start);
        let sample = meter.sample(0, 1_000, start);
        assert_eq!(sample.speed_per_second, 0);
        assert_eq!(sample.eta, None);
        assert_eq!(sample.elapsed, Duration::ZERO);
    }

    #[test]
    fn a_steady_rate_is_measured_correctly() {
        let start = Instant::now();
        let mut meter = SpeedMeter::new(start);
        meter.sample(0, 10_000, start);
        // Sampel pertama hanya menetapkan titik acuan; kecepatan muncul di sampel kedua.
        meter.sample(1_000, 10_000, at(start, 1_000));
        let sample = meter.sample(2_000, 10_000, at(start, 2_000));
        assert!(
            sample.speed_per_second >= 900 && sample.speed_per_second <= 1_100,
            "unexpected speed: {}",
            sample.speed_per_second
        );
    }

    #[test]
    fn elapsed_grows_with_time() {
        let start = Instant::now();
        let mut meter = SpeedMeter::new(start);
        let sample = meter.sample(0, 100, at(start, 2_500));
        assert_eq!(sample.elapsed, Duration::from_millis(2_500));
    }

    #[test]
    fn a_zero_speed_produces_no_estimate_instead_of_dividing_by_zero() {
        let start = Instant::now();
        let mut meter = SpeedMeter::new(start);
        meter.sample(0, 100, start);
        let sample = meter.sample(0, 100, at(start, 1_000));
        assert_eq!(sample.speed_per_second, 0);
        assert_eq!(sample.eta, None);
    }

    #[test]
    fn the_estimate_shrinks_as_work_completes() {
        let start = Instant::now();
        let mut meter = SpeedMeter::new(start);
        meter.sample(0, 10_000, start);
        meter.sample(1_000, 10_000, at(start, 1_000));
        let early = meter.sample(2_000, 10_000, at(start, 2_000));
        let late = meter.sample(8_000, 10_000, at(start, 8_000));
        assert!(late.eta.unwrap() < early.eta.unwrap());
    }

    #[test]
    fn a_sudden_spike_is_smoothed_instead_of_taken_at_face_value() {
        let start = Instant::now();
        let mut meter = SpeedMeter::new(start);
        meter.sample(0, 100_000, start);
        meter.sample(100, 100_000, at(start, 1_000));
        // Lonjakan: 100 kali lebih cepat dari sebelumnya.
        let sample = meter.sample(10_100, 100_000, at(start, 2_000));
        assert!(
            sample.speed_per_second < 5_000,
            "smoothing did not hold the speed back: {}",
            sample.speed_per_second
        );
    }

    #[test]
    fn an_empty_total_does_not_produce_nan() {
        let start = Instant::now();
        let mut meter = SpeedMeter::new(start);
        let sample = meter.sample(0, 0, at(start, 500));
        assert!(!sample.fraction().is_nan());
        assert_eq!(sample.fraction(), 0.0);
    }
}
```

- [ ] **Step 2: Jalankan tes untuk memastikan gagal**

Run: `cargo test -p zydra-core engine`
Expected: gagal kompilasi dengan `cannot find type CancelToken`.

- [ ] **Step 3: Tulis implementasi minimal**

`crates/zydra-core/src/engine/mod.rs`:

```rust
//! Mesin pemecah paralel.

pub mod cancel;
pub mod stats;
```

`crates/zydra-core/src/engine/cancel.rs`:

```rust
//! Tanda pembatalan yang bisa dibagikan ke semua worker.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Tanda berbagi untuk menghentikan pekerjaan yang sedang berjalan.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    /// Membuat tanda yang belum dibatalkan.
    pub fn new() -> CancelToken {
        CancelToken(Arc::new(AtomicBool::new(false)))
    }

    /// Meminta semua worker berhenti.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Apakah pembatalan sudah diminta.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
```

`crates/zydra-core/src/engine/stats.rs`:

```rust
//! Pengukuran kecepatan dan perkiraan waktu selesai.

use std::time::{Duration, Instant};

use crate::constants::SPEED_SMOOTHING_FACTOR;
use crate::model::progress::TryingData;

/// Menghitung kecepatan rata-rata bergerak dari cuplikan berulang.
///
/// Rata-rata bergerak dipakai supaya perkiraan waktu ikut menyesuaikan ketika kecepatan
/// berubah; rata-rata kumulatif dari awal justru makin melenceng pada serangan panjang.
#[derive(Debug)]
pub struct SpeedMeter {
    start: Instant,
    previous_at: Instant,
    previous_tried: u64,
    smoothed_speed: f32,
}

impl SpeedMeter {
    /// Membuat pengukur dengan titik acuan awal.
    pub fn new(start: Instant) -> SpeedMeter {
        SpeedMeter {
            start,
            previous_at: start,
            previous_tried: 0,
            smoothed_speed: 0.0,
        }
    }

    /// Mencatat kemajuan dan mengembalikan cuplikan terbaru.
    pub fn sample(&mut self, tried: u64, total: u64, now: Instant) -> TryingData {
        let delta_seconds = now.saturating_duration_since(self.previous_at).as_secs_f32();
        let delta_tried = tried.saturating_sub(self.previous_tried) as f32;
        if delta_seconds > 0.0 {
            let instant_speed = delta_tried / delta_seconds;
            self.smoothed_speed = if self.smoothed_speed <= 0.0 {
                instant_speed
            } else {
                SPEED_SMOOTHING_FACTOR * instant_speed
                    + (1.0 - SPEED_SMOOTHING_FACTOR) * self.smoothed_speed
            };
        }
        self.previous_at = now;
        self.previous_tried = tried;

        let speed_per_second = self.smoothed_speed.max(0.0).round() as u64;
        let remaining = total.saturating_sub(tried);
        let eta = if speed_per_second > 0 {
            Some(Duration::from_secs_f64(
                remaining as f64 / speed_per_second as f64,
            ))
        } else {
            None
        };

        TryingData {
            tried,
            total,
            speed_per_second,
            elapsed: now.saturating_duration_since(self.start),
            eta,
        }
    }
}
```

Tambahkan `pub mod engine;` ke `crates/zydra-core/src/lib.rs`.

- [ ] **Step 4: Jalankan tes untuk memastikan lolos**

Run: `cargo test -p zydra-core engine`
Expected: `test result: ok. 11 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/zydra-core
git commit -m "feat(engine): measure speed and let anyone call it off"
```

---

### Task 9: `engine::crack`

**Files:**
- Create: `crates/zydra-core/src/engine/crack.rs`
- Modify: `crates/zydra-core/src/engine/mod.rs`

**Interfaces:**
- Consumes: `candidate::CandidateSource`, `engine::cancel::CancelToken`, `engine::stats::SpeedMeter`, `model::failure::CrackFailure`, `model::progress::TryingData`, `model::target::CrackTarget`, `tester::{MatchOutcome, PasswordTester, TesterFactory}`, `constants::{DEFAULT_BATCH_SIZE, DEFAULT_PROGRESS_INTERVAL_MS, DEFAULT_WORKERS, MIN_WORKERS}`.
- Produces:
  - `engine::crack::EngineConfig { workers: usize, batch_size: usize, progress_interval: Duration }` dengan `Default`
  - `engine::crack::EngineEvent` (`Started { total }`, `Progress(TryingData)`, `Found { password, detail }`, `Exhausted`, `Cancelled`, `Failed(CrackFailure)`)
  - `engine::crack::run(source: Box<dyn CandidateSource>, factory: Arc<dyn TesterFactory>, target: CrackTarget, config: EngineConfig, cancel: CancelToken) -> crossbeam_channel::Receiver<EngineEvent>`

- [ ] **Step 1: Tulis tes yang gagal**

`crates/zydra-core/src/engine/crack.rs`, hanya blok tes:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::bruteforce::BruteForceSource;
    use crate::model::charset::CharsetSet;
    use crate::model::target::TargetKind;
    use crate::tester::{EncryptionKind, MatchOutcome, PasswordTester, TesterFactory, TesterInfo};

    struct FakeTester {
        secret: String,
        delay: Duration,
        info: TesterInfo,
    }

    impl PasswordTester for FakeTester {
        fn try_password(&mut self, password: &str) -> MatchOutcome {
            if !self.delay.is_zero() {
                std::thread::sleep(self.delay);
            }
            if password == self.secret {
                MatchOutcome::Matched { detail: None }
            } else {
                MatchOutcome::NoMatch
            }
        }

        fn info(&self) -> &TesterInfo {
            &self.info
        }
    }

    struct FakeFactory {
        secret: String,
        delay: Duration,
        failure: Option<CrackFailure>,
    }

    impl FakeFactory {
        fn new(secret: &str) -> FakeFactory {
            FakeFactory {
                secret: secret.to_string(),
                delay: Duration::ZERO,
                failure: None,
            }
        }

        fn with_delay(secret: &str, delay: Duration) -> FakeFactory {
            FakeFactory {
                secret: secret.to_string(),
                delay,
                failure: None,
            }
        }

        fn failing(failure: CrackFailure) -> FakeFactory {
            FakeFactory {
                secret: String::new(),
                delay: Duration::ZERO,
                failure: Some(failure),
            }
        }
    }

    impl TesterFactory for FakeFactory {
        fn create(&self, _target: &CrackTarget) -> Result<Box<dyn PasswordTester>, CrackFailure> {
            if let Some(failure) = &self.failure {
                return Err(failure.clone());
            }
            Ok(Box::new(FakeTester {
                secret: self.secret.clone(),
                delay: self.delay,
                info: TesterInfo {
                    kind: TargetKind::Zip,
                    encryption: EncryptionKind::ZipCrypto,
                    entries: 1,
                },
            }))
        }
    }

    struct EmptySource;

    impl CandidateSource for EmptySource {
        fn next_candidate(&mut self) -> Option<String> {
            None
        }
        fn total(&self) -> u64 {
            0
        }
        fn position(&self) -> u64 {
            0
        }
    }

    /// Sumber sintetis dengan jumlah kandidat yang pasti. Dipakai untuk tes yang butuh
    /// pekerjaan berlangsung cukup lama agar progres sempat terkirim, tanpa bergantung
    /// pada ukuran ruang brute force.
    struct CountingSource {
        remaining: u64,
        total: u64,
        position: u64,
    }

    impl CountingSource {
        fn new(total: u64) -> CountingSource {
            CountingSource {
                remaining: total,
                total,
                position: 0,
            }
        }
    }

    impl CandidateSource for CountingSource {
        fn next_candidate(&mut self) -> Option<String> {
            if self.remaining == 0 {
                return None;
            }
            self.remaining -= 1;
            self.position += 1;
            Some(format!("candidate-{}", self.position))
        }
        fn total(&self) -> u64 {
            self.total
        }
        fn position(&self) -> u64 {
            self.position
        }
    }

    fn target() -> CrackTarget {
        CrackTarget::from_path("vault.zip")
    }

    fn config() -> EngineConfig {
        EngineConfig {
            workers: 2,
            batch_size: 8,
            progress_interval: Duration::from_millis(10),
        }
    }

    fn collect(receiver: Receiver<EngineEvent>) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        loop {
            match receiver.recv_timeout(Duration::from_secs(10)) {
                Ok(event) => {
                    let terminal = matches!(
                        event,
                        EngineEvent::Found { .. }
                            | EngineEvent::Exhausted
                            | EngineEvent::Cancelled
                            | EngineEvent::Failed(_)
                    );
                    events.push(event);
                    if terminal {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        events
    }

    fn small_source() -> Box<dyn CandidateSource> {
        Box::new(BruteForceSource::new(CharsetSet::LOWERCASE, 1, 2).unwrap())
    }

    #[test]
    fn started_is_always_the_first_event() {
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::new("zz")),
            target(),
            config(),
            CancelToken::new(),
        ));
        assert!(matches!(events.first(), Some(EngineEvent::Started { .. })));
    }

    #[test]
    fn started_reports_the_total_from_the_source() {
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::new("zz")),
            target(),
            config(),
            CancelToken::new(),
        ));
        assert_eq!(events.first(), Some(&EngineEvent::Started { total: 26 + 676 }));
    }

    #[test]
    fn finds_a_password_that_is_in_the_candidate_space() {
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::new("ba")),
            target(),
            config(),
            CancelToken::new(),
        ));
        let found = events
            .iter()
            .find_map(|event| match event {
                EngineEvent::Found { password, .. } => Some(password.clone()),
                _ => None,
            })
            .expect("no password was found");
        assert_eq!(found, "ba");
    }

    #[test]
    fn reports_exhaustion_when_nothing_matches() {
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::new("not-in-the-space")),
            target(),
            config(),
            CancelToken::new(),
        ));
        assert!(matches!(events.last(), Some(EngineEvent::Exhausted)));
    }

    #[test]
    fn a_cancel_before_starting_reports_cancelled() {
        let cancel = CancelToken::new();
        cancel.cancel();
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::new("ba")),
            target(),
            config(),
            cancel,
        ));
        assert!(matches!(events.last(), Some(EngineEvent::Cancelled)));
        assert!(!events
            .iter()
            .any(|event| matches!(event, EngineEvent::Found { .. })));
    }

    #[test]
    fn an_empty_source_fails_immediately_with_no_candidate() {
        let events = collect(run(
            Box::new(EmptySource),
            Arc::new(FakeFactory::new("ba")),
            target(),
            config(),
            CancelToken::new(),
        ));
        assert_eq!(
            events,
            vec![EngineEvent::Failed(CrackFailure::NoCandidate)]
        );
    }

    #[test]
    fn a_failing_factory_reports_its_failure() {
        let failure = CrackFailure::BackendUnavailable {
            backend: crate::model::failure::Backend::Rar,
            reason: "built without rar support".to_string(),
        };
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::failing(failure.clone())),
            target(),
            config(),
            CancelToken::new(),
        ));
        assert_eq!(events.last(), Some(&EngineEvent::Failed(failure)));
    }

    #[test]
    fn progress_events_arrive_while_work_is_running() {
        // 400 kandidat x 1 ms, dibagi 2 worker, berlangsung sekitar 200 ms. Interval
        // progres 10 ms, jadi beberapa peristiwa pasti sempat terkirim.
        let events = collect(run(
            Box::new(CountingSource::new(400)),
            Arc::new(FakeFactory::with_delay("no-match", Duration::from_millis(1))),
            target(),
            config(),
            CancelToken::new(),
        ));
        let progress_count = events
            .iter()
            .filter(|event| matches!(event, EngineEvent::Progress(_)))
            .count();
        assert!(
            progress_count >= 1,
            "no progress event arrived: {events:?}"
        );
    }

    #[test]
    fn many_workers_still_exhaust_the_whole_space() {
        let mut wide = config();
        wide.workers = 4;
        let events = collect(run(
            Box::new(CountingSource::new(200)),
            Arc::new(FakeFactory::with_delay("no-match", Duration::from_micros(200))),
            target(),
            wide,
            CancelToken::new(),
        ));
        assert!(
            matches!(events.last(), Some(EngineEvent::Exhausted)),
            "the run did not finish cleanly: {events:?}"
        );
    }
}
```

- [ ] **Step 2: Jalankan tes untuk memastikan gagal**

Run: `cargo test -p zydra-core crack`
Expected: gagal kompilasi dengan `cannot find function run`.

- [ ] **Step 3: Tulis implementasi minimal**

```rust
//! Menjalankan penelusuran kandidat secara paralel.
//!
//! Modul ini tidak tahu format apa pun dan tidak tahu dari mana kandidat berasal; ia
//! hanya mengenal [`CandidateSource`] dan [`TesterFactory`].

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver};

use crate::candidate::CandidateSource;
use crate::constants::{
    DEFAULT_BATCH_SIZE, DEFAULT_PROGRESS_INTERVAL_MS, DEFAULT_WORKERS, MIN_WORKERS,
};
use crate::engine::cancel::CancelToken;
use crate::engine::stats::SpeedMeter;
use crate::model::failure::CrackFailure;
use crate::model::progress::TryingData;
use crate::model::target::CrackTarget;
use crate::tester::{MatchOutcome, PasswordTester, TesterFactory};

/// Pengaturan satu kali penelusuran.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Jumlah worker. `0` berarti mengikuti jumlah CPU.
    pub workers: usize,
    /// Jumlah kandidat yang diambil satu worker dalam sekali kunci.
    pub batch_size: usize,
    /// Jarak antar peristiwa progres.
    pub progress_interval: Duration,
}

impl Default for EngineConfig {
    fn default() -> EngineConfig {
        EngineConfig {
            workers: DEFAULT_WORKERS,
            batch_size: DEFAULT_BATCH_SIZE,
            progress_interval: Duration::from_millis(DEFAULT_PROGRESS_INTERVAL_MS),
        }
    }
}

/// Peristiwa yang dikirim mesin ke pemanggil.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    /// Penelusuran dimulai. Selalu peristiwa pertama.
    Started {
        /// Jumlah kandidat yang akan dicoba.
        total: u64,
    },
    /// Kemajuan terbaru.
    Progress(TryingData),
    /// Password ditemukan. Peristiwa ini menutup penelusuran.
    Found {
        /// Password yang cocok.
        password: String,
        /// Keterangan tambahan dari backend.
        detail: Option<String>,
    },
    /// Semua kandidat habis tanpa hasil.
    Exhausted,
    /// Penelusuran dihentikan atas permintaan.
    Cancelled,
    /// Penelusuran tidak bisa dijalankan.
    Failed(CrackFailure),
}

/// Menjalankan penelusuran dan segera mengembalikan kanal peristiwa.
///
/// Pekerjaan berjalan di thread terpisah, jadi pemanggil bisa membaca kanal dari loop
/// gambar antarmuka atau loop keluaran terminal tanpa pernah membeku. Pemanggil berhenti
/// membaca setelah peristiwa penutup pertama (`Found`, `Exhausted`, `Cancelled`, atau
/// `Failed`).
pub fn run(
    source: Box<dyn CandidateSource>,
    factory: Arc<dyn TesterFactory>,
    target: CrackTarget,
    config: EngineConfig,
    cancel: CancelToken,
) -> Receiver<EngineEvent> {
    let (sender, receiver) = unbounded();

    let total = source.total();
    if total == 0 {
        let _ = sender.send(EngineEvent::Failed(CrackFailure::NoCandidate));
        return receiver;
    }

    let workers = if config.workers == DEFAULT_WORKERS {
        num_cpus::get()
    } else {
        config.workers
    }
    .max(MIN_WORKERS);
    let batch_size = config.batch_size.max(1);
    let interval = config.progress_interval;

    thread::spawn(move || {
        let _ = sender.send(EngineEvent::Started { total });

        let mut testers: Vec<Box<dyn PasswordTester>> = Vec::with_capacity(workers);
        for _ in 0..workers {
            match factory.create(&target) {
                Ok(tester) => testers.push(tester),
                Err(failure) => {
                    let _ = sender.send(EngineEvent::Failed(failure));
                    return;
                }
            }
        }

        let shared_source = Arc::new(Mutex::new(source));
        let found = Arc::new(AtomicBool::new(false));
        let tried = Arc::new(AtomicU64::new(0));
        let finished = Arc::new(AtomicBool::new(false));

        let mut workers_handles = Vec::with_capacity(workers);
        for mut tester in testers {
            let shared_source = Arc::clone(&shared_source);
            let found = Arc::clone(&found);
            let tried = Arc::clone(&tried);
            let cancel = cancel.clone();
            let sender = sender.clone();
            workers_handles.push(thread::spawn(move || {
                loop {
                    if found.load(Ordering::SeqCst) || cancel.is_cancelled() {
                        break;
                    }
                    let batch = match take_batch(&shared_source, batch_size) {
                        Some(batch) => batch,
                        None => break,
                    };
                    for password in batch {
                        if found.load(Ordering::SeqCst) || cancel.is_cancelled() {
                            break;
                        }
                        tried.fetch_add(1, Ordering::Relaxed);
                        if let MatchOutcome::Matched { detail } = tester.try_password(&password) {
                            if !found.swap(true, Ordering::SeqCst) {
                                let _ = sender.send(EngineEvent::Found { password, detail });
                            }
                            break;
                        }
                    }
                }
            }));
        }

        let progress_handle = {
            let tried = Arc::clone(&tried);
            let found = Arc::clone(&found);
            let finished = Arc::clone(&finished);
            let sender = sender.clone();
            thread::spawn(move || {
                let start = Instant::now();
                let mut meter = SpeedMeter::new(start);
                while !finished.load(Ordering::Relaxed) && !found.load(Ordering::SeqCst) {
                    thread::sleep(interval);
                    if finished.load(Ordering::Relaxed) || found.load(Ordering::SeqCst) {
                        break;
                    }
                    let sample = meter.sample(tried.load(Ordering::Relaxed), total, Instant::now());
                    if sender.send(EngineEvent::Progress(sample)).is_err() {
                        break;
                    }
                }
            })
        };

        for handle in workers_handles {
            let _ = handle.join();
        }
        finished.store(true, Ordering::Relaxed);
        let _ = progress_handle.join();

        if found.load(Ordering::SeqCst) {
            // Peristiwa Found sudah dikirim oleh worker yang menemukannya.
        } else if cancel.is_cancelled() {
            let _ = sender.send(EngineEvent::Cancelled);
        } else {
            let _ = sender.send(EngineEvent::Exhausted);
        }
    });

    receiver
}

/// Mengambil sekumpulan kandidat sekaligus, lalu melepas kunci sebelum mengujinya.
fn take_batch(
    source: &Arc<Mutex<Box<dyn CandidateSource>>>,
    size: usize,
) -> Option<Vec<String>> {
    let mut guard = source.lock().ok()?;
    let mut batch = Vec::with_capacity(size);
    for _ in 0..size {
        match guard.next_candidate() {
            Some(candidate) => batch.push(candidate),
            None => break,
        }
    }
    if batch.is_empty() {
        None
    } else {
        Some(batch)
    }
}
```

Tambahkan `pub mod crack;` ke `crates/zydra-core/src/engine/mod.rs`.

- [ ] **Step 4: Jalankan tes untuk memastikan lolos**

Run: `cargo test -p zydra-core crack`
Expected: `test result: ok. 9 passed`.

Run: `cargo test --workspace`
Expected: seluruh tes dari tugas 1 sampai 9 lolos.

- [ ] **Step 5: Verifikasi lint dan commit**

Run: `cargo fmt --check`
Expected: tidak ada keluaran.

Run: `cargo clippy --all-targets -- -D warnings`
Expected: sukses tanpa peringatan.

```bash
git add crates/zydra-core
git commit -m "feat(engine): run the search on every core and stop on the first match"
```

---

## Self-Review

**Cakupan spec.** Bagian spec yang harus terpenuhi di rencana ini dan tugas yang mengerjakannya:

| Bagian spec | Tugas |
|---|---|
| 4 aturan dependensi (`zydra-core` tanpa UI) | Task 1 (batasan global) |
| 5 struktur direktori `zydra-core` | Task 1–9 |
| 6.1 `TargetKind`, `CrackTarget` | Task 3 |
| 6.1 `CrackFailure`, `Backend` | Task 4 |
| 6.1 `TryingData`, `CrackProgress` | Task 4 |
| 6.1 `CharsetSet` | Task 2 |
| 6.2 `CandidateSource`, `WordlistSource` | Task 5 |
| 6.2 `BruteForceSource` | Task 6 |
| 6.3 `PasswordTester`, `TesterFactory`, `TesterInfo`, `EncryptionKind`, `MatchOutcome` | Task 7 |
| 6.4 `EngineConfig`, `EngineEvent`, `run()` | Task 9 |
| 6.4 `CancelToken` | Task 8 |
| 6.4 `SpeedMeter` dengan rata-rata bergerak | Task 8 |
| 6.8 `constants.rs` | Task 2 |
| 6.9 `CoreError` | Task 2 |
| 12.1 unit test untuk charset, bruteforce, wordlist, stats, engine | Task 2, 5, 6, 8, 9 |

Yang **sengaja belum** dikerjakan di rencana ini dan menjadi tanggung jawab rencana berikutnya:
`model::mask` dan `model::rules` (rencana C), `candidate::{mask,rules}` (rencana C),
`tester::{factory,zip,rar,pdf,shadow}` (rencana A2), `session`, `config`, `log` (rencana C),
`zydra-i18n` (rencana A2), `zydra-cli` (rencana A2), `zydra-gui` (rencana B), CI (rencana C).

**Pemeriksaan placeholder.** Tidak ada "TBD", "TODO", atau "implementasikan nanti". Setiap langkah
kode memuat kode sebenarnya. Satu catatan eksplisit saya tinggalkan di Task 9 tentang impor `Sender`
yang harus dihilangkan — itu instruksi pasti, bukan placeholder.

**Konsistensi tipe.** Diperiksa silang antar tugas:

- `CandidateSource::skip` didefinisikan di Task 5 dengan implementasi bawaan, dipakai ulang di Task 6
  dengan implementasi yang lebih cepat. Nama dan tanda tangan sama.
- `CrackFailure::InvalidLengthRange` dibuat di Task 4 dengan tiga medan, dan diisi di Task 6 dengan
  ketiga medan itu.
- `TryingData` dibuat di Task 4, dipakai oleh `SpeedMeter::sample` di Task 8 dan oleh
  `EngineEvent::Progress` di Task 9. Medannya sama.
- `EngineConfig` memakai `DEFAULT_WORKERS` sebagai penanda "otomatis" di Task 9, dan konstanta itu
  dibuat di Task 2. Task 9 membandingkan dengan `DEFAULT_WORKERS`, bukan dengan angka `0` telanjang.
- `TesterFactory::create` menerima `&CrackTarget` di Task 7 dan dipanggil dengan `&target` di Task 9.
- `PasswordTester::info` mengembalikan `&TesterInfo` di Task 7, dan `FakeTester` di Task 9 menyimpan
  `TesterInfo` sebagai medan supaya bisa mengembalikan referensi.
- `EncryptionKind` dan `UnixCryptScheme` dibuat di Task 7; `FakeTester` di Task 9 memakai
  `EncryptionKind::ZipCrypto` yang memang ada di daftar itu.
