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
