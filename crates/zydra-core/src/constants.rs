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

/// Batas jumlah byte entri ZIP yang dibaca saat memverifikasi kandidat.
///
/// Verifikasi ZIP harus membaca entri sampai habis karena ZipCrypto hanya menyediakan
/// 1 dari 256 bit untuk validasi, sehingga password salah bisa lolos di header. Batas ini
/// mencegah arsip dengan entri raksasa menghabiskan memori dan waktu.
pub const DEFAULT_ZIP_MAX_ENTRY_BYTES: u64 = 64 * 1024 * 1024;
/// Nilai `workers` yang berarti "ikuti jumlah CPU".
pub const DEFAULT_WORKERS: usize = 0;

/// Jumlah worker minimum yang selalu dibuat.
pub const MIN_WORKERS: usize = 1;

/// Ukuran buffer pembaca wordlist.
pub const WORDLIST_READ_BUFFER_BYTES: usize = 64 * 1024;
