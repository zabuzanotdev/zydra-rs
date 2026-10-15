//! Merangkum langkah kemajuan menjadi satu kalimat.

use std::time::Duration;

use zydra_core::model::progress::CrackProgress;

use crate::Messages;

/// Kalimat ringkas untuk sebuah langkah kemajuan.
pub(crate) fn describe(messages: &Messages, progress: &CrackProgress) -> String {
    match progress {
        CrackProgress::Trying(data) => {
            let speed = data.speed_per_second.to_string();
            let eta = match data.eta {
                Some(eta) => format_duration(eta),
                None => messages.text("stat-value-unknown"),
            };
            messages.text_args(
                "progress-line",
                &[
                    ("tried", &data.tried.to_string()),
                    ("total", &data.total.to_string()),
                    ("speed", &speed),
                    ("eta", &eta),
                ],
            )
        }
        CrackProgress::Found { password, .. } => {
            messages.text_args("result-key", &[("value", password)])
        }
        CrackProgress::Failed { failure } => messages.failure(failure),
    }
}

/// Durasi sebagai `HH:MM:SS`, atau `MM:SS` kalau di bawah satu jam.
pub(crate) fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Locale;
    use zydra_core::model::failure::CrackFailure;
    use zydra_core::model::progress::TryingData;

    fn trying(speed: u64, eta: Option<Duration>) -> CrackProgress {
        CrackProgress::Trying(TryingData {
            tried: 1_500,
            total: 10_000,
            speed_per_second: speed,
            elapsed: Duration::from_secs(3),
            eta,
        })
    }

    #[test]
    fn a_trying_step_mentions_both_counts() {
        let messages = Messages::new(Locale::En);
        let text = messages.progress(&trying(500, Some(Duration::from_secs(17))));
        assert!(text.contains("1500"), "unexpected text: {text}");
        assert!(text.contains("10000"), "unexpected text: {text}");
    }

    #[test]
    fn an_unknown_estimate_is_spelled_out() {
        let messages = Messages::new(Locale::En);
        let text = messages.progress(&trying(0, None));
        assert!(text.contains("n/a"), "unexpected text: {text}");
    }

    #[test]
    fn a_failure_step_uses_the_failure_text() {
        let messages = Messages::new(Locale::Id);
        let text = messages.progress(&CrackProgress::Failed {
            failure: CrackFailure::EmptyWordlist,
        });
        assert_eq!(text, "Wordlist tidak memuat satu kandidat pun.");
    }

    #[test]
    fn short_durations_are_minutes_and_seconds() {
        assert_eq!(format_duration(Duration::from_secs(0)), "00:00");
        assert_eq!(format_duration(Duration::from_secs(59)), "00:59");
        assert_eq!(format_duration(Duration::from_secs(61)), "01:01");
        assert_eq!(format_duration(Duration::from_secs(3_599)), "59:59");
    }

    #[test]
    fn long_durations_gain_an_hour_field() {
        assert_eq!(format_duration(Duration::from_secs(3_600)), "01:00:00");
        assert_eq!(format_duration(Duration::from_secs(7_384)), "02:03:04");
    }
}
