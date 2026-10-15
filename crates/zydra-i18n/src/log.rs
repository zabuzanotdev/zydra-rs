//! Menyusun satu baris log dari sebuah entri.
//!
//! Entri menyimpan kunci terjemahan dan argumennya, bukan kalimat jadi. Karena itu baris
//! log selalu tampil dalam bahasa yang sedang aktif, bahkan untuk kejadian yang tercatat
//! sebelum bahasa diganti.

use zydra_core::log::{LogEntry, LogLevel};

use crate::Messages;

/// Satu baris log lengkap: waktu, tingkat, lalu teksnya.
pub(crate) fn describe(messages: &Messages, entry: &LogEntry) -> String {
    let borrowed: Vec<(&str, &str)> = entry
        .args
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    format!(
        "{} {} {}",
        format_time(entry.at),
        messages.text(level_key(entry.level)),
        messages.text_args(&entry.key, &borrowed)
    )
}

fn level_key(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Info => "log-level-info",
        LogLevel::Warn => "log-level-warn",
        LogLevel::Error => "log-level-error",
        LogLevel::Success => "log-level-success",
    }
}

/// Waktu sebagai `HH:MM:SS` dalam UTC.
///
/// UTC dipakai karena jam lokal akan membutuhkan basis data zona waktu, dan untuk catatan
/// operasi waktu yang tidak ambigu justru lebih berguna.
pub(crate) fn format_time(unix_seconds: i64) -> String {
    let seconds_of_day = unix_seconds.rem_euclid(86_400);
    let hours = seconds_of_day / 3_600;
    let minutes = (seconds_of_day % 3_600) / 60;
    let seconds = seconds_of_day % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Locale;

    #[test]
    fn a_log_line_contains_the_time_the_level_and_the_text() {
        let messages = Messages::new(Locale::En);
        // Waktu disusun dari komponennya supaya niatnya terbaca dan tidak bergantung pada
        // nilai epoch yang dihitung tangan.
        let seconds_of_day = 9 * 3_600 + 41 * 60 + 2;
        let entry =
            LogEntry::now(LogLevel::Info, "log-target-selected").with_arg("name", "vault.zip");
        let entry = LogEntry {
            at: 20_000 * 86_400 + seconds_of_day,
            ..entry
        };

        let line = messages.log_line(&entry);
        assert!(line.starts_with("09:41:02 "), "unexpected line: {line}");
        assert!(line.contains("INFO"), "unexpected line: {line}");
        assert!(line.contains("vault.zip"), "unexpected line: {line}");
    }

    #[test]
    fn the_level_word_follows_the_active_language() {
        let entry = LogEntry {
            at: 0,
            level: LogLevel::Error,
            key: "error-no-match".to_string(),
            args: Default::default(),
        };
        let english = Messages::new(Locale::En).log_line(&entry);
        let indonesian = Messages::new(Locale::Id).log_line(&entry);

        assert!(english.contains("ERROR"), "unexpected line: {english}");
        assert!(
            indonesian.contains("GALAT"),
            "unexpected line: {indonesian}"
        );
        assert!(
            indonesian.contains("Tidak ada kandidat"),
            "unexpected line: {indonesian}"
        );
    }

    #[test]
    fn midnight_is_formatted_as_zero_hours() {
        assert_eq!(format_time(0), "00:00:00");
    }

    #[test]
    fn times_are_derived_from_the_seconds_of_the_day() {
        assert_eq!(format_time(86_400 + 61), "00:01:01");
        assert_eq!(format_time(3_600 + 120), "01:02:00");
    }

    #[test]
    fn a_negative_timestamp_does_not_produce_a_broken_time() {
        assert_eq!(format_time(-1), "23:59:59");
    }
}
