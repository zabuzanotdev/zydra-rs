//! Pemformatan angka dan durasi.
//!
//! Tinggal di sini, bukan di masing-masing antarmuka, karena keduanya harus menampilkan
//! angka dengan cara yang sama. Kalau CLI dan GUI punya salinannya sendiri, cepat atau
//! lambat keduanya akan berbeda dan tidak ada yang tahu mana yang benar.

use std::time::Duration;

/// Angka dengan pemisah ribuan.
///
/// `14344391` menjadi `14,344,391`. Jumlah kandidat pada serangan nyata mudah mencapai
/// puluhan juta, dan angka tanpa pemisah praktis tidak bisa dibaca sekilas.
pub fn count(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(character);
    }
    out
}

/// Durasi sebagai `HH:MM:SS`, atau `MM:SS` kalau di bawah satu jam.
pub fn duration(value: Duration) -> String {
    let total_seconds = value.as_secs();
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

/// Bagian yang sudah selesai sebagai persentase dengan tiga angka di belakang koma.
///
/// Total nol menghasilkan `0.000`, bukan `NaN`.
pub fn percent(fraction: f32) -> String {
    let value = if fraction.is_nan() {
        0.0
    } else {
        fraction.clamp(0.0, 1.0)
    };
    format!("{:.3}", value * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_numbers_have_no_separator() {
        assert_eq!(count(0), "0");
        assert_eq!(count(7), "7");
        assert_eq!(count(999), "999");
    }

    #[test]
    fn thousands_are_separated() {
        assert_eq!(count(1_000), "1,000");
        assert_eq!(count(14_344_391), "14,344,391");
    }

    #[test]
    fn separators_land_on_every_third_digit_from_the_right() {
        assert_eq!(count(1_234_567_890), "1,234,567,890");
        assert_eq!(count(10_000), "10,000");
        assert_eq!(count(100_000), "100,000");
    }

    #[test]
    fn durations_under_an_hour_omit_the_hour_field() {
        assert_eq!(duration(Duration::from_secs(0)), "00:00");
        assert_eq!(duration(Duration::from_secs(59)), "00:59");
        assert_eq!(duration(Duration::from_secs(61)), "01:01");
        assert_eq!(duration(Duration::from_secs(3_599)), "59:59");
    }

    #[test]
    fn durations_over_an_hour_include_it() {
        assert_eq!(duration(Duration::from_secs(3_600)), "01:00:00");
        assert_eq!(duration(Duration::from_secs(7_384)), "02:03:04");
    }

    #[test]
    fn percentages_keep_three_decimals() {
        assert_eq!(percent(0.0), "0.000");
        assert_eq!(percent(0.5), "50.000");
        assert_eq!(percent(1.0), "100.000");
    }

    #[test]
    fn percentages_are_clamped_and_never_nan() {
        assert_eq!(percent(2.0), "100.000");
        assert_eq!(percent(-1.0), "0.000");
        assert_eq!(percent(f32::NAN), "0.000");
    }
}
