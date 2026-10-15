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
        let delta_seconds = now
            .saturating_duration_since(self.previous_at)
            .as_secs_f32();
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
