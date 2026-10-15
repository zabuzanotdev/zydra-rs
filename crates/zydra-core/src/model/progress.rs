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
