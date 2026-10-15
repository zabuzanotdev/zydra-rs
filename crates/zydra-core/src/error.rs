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
