//! Logika pemulihan password untuk ZIP, RAR, PDF, dan berkas `shadow` Unix.
//!
//! Crate ini tidak memuat dependensi antarmuka pengguna dan tidak memuat kalimat yang
//! ditampilkan kepada pengguna. Kesalahan dikembalikan sebagai tipe; penerjemahan menjadi
//! teks adalah tanggung jawab crate `zydra-i18n`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod attack;
pub mod candidate;
pub mod constants;
pub mod engine;
pub mod error;
pub mod log;
pub mod model;
pub mod session;
pub mod tester;
