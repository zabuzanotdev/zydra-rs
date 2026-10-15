//! Memetakan kegagalan pemulihan menjadi kunci terjemahan.
//!
//! Pemetaan ini hidup di lapisan terjemahan, bukan di `zydra-core`. Core tidak boleh tahu
//! bagaimana kegagalannya akan ditulis, dan lapisan ini tidak boleh tahu bagaimana
//! kegagalan itu dideteksi.

use zydra_core::model::failure::{Backend, CrackFailure};

/// Kunci terjemahan beserta argumennya.
pub(crate) type Description = (&'static str, Vec<(String, String)>);

/// Memetakan sebuah kegagalan ke kunci dan argumennya.
pub(crate) fn describe(failure: &CrackFailure) -> Description {
    match failure {
        CrackFailure::PasswordNotFound => ("error-no-match", vec![]),
        CrackFailure::UnsupportedFormat { display_name } => {
            ("error-unsupported-format", vec![arg("name", display_name)])
        }
        CrackFailure::EmptyWordlist => ("error-empty-wordlist", vec![]),
        // Alasan teknisnya sengaja tidak ditampilkan; ia masuk ke catatan, bukan ke layar.
        CrackFailure::FileUnreadable { .. } => ("error-file-unreadable", vec![]),
        CrackFailure::NoCharsetSelected => ("error-no-charset", vec![]),
        CrackFailure::InvalidLengthRange {
            min,
            max,
            allowed_max,
        } => (
            "error-bad-range",
            vec![
                arg("min", min),
                arg("max", max),
                arg("allowed_max", allowed_max),
            ],
        ),
        CrackFailure::InvalidMask { pattern, .. } => {
            ("error-invalid-mask", vec![arg("pattern", pattern)])
        }
        CrackFailure::InvalidRule { rule, .. } => ("error-invalid-rule", vec![arg("rule", rule)]),
        CrackFailure::TooManyCombinations { total, limit } => (
            "error-too-many-combinations",
            vec![arg("total", total), arg("limit", limit)],
        ),
        CrackFailure::NoCandidate => ("error-no-candidate", vec![]),
        CrackFailure::BackendUnavailable { backend, .. } => (
            "error-backend-unavailable",
            vec![arg("backend", backend_name(*backend))],
        ),
        CrackFailure::SessionFingerprintMismatch => ("error-session-mismatch", vec![]),
    }
}

/// Nama backend yang aman ditampilkan.
pub(crate) fn backend_name(backend: Backend) -> &'static str {
    match backend {
        Backend::Zip => "ZIP",
        Backend::Rar => "RAR",
        Backend::Pdf => "PDF",
        Backend::Shadow => "shadow",
    }
}

fn arg(name: &str, value: impl ToString) -> (String, String) {
    (name.to_string(), value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Locale;
    use crate::Messages;

    #[test]
    fn every_failure_maps_to_a_key_that_exists_in_both_locales() {
        let failures = [
            CrackFailure::PasswordNotFound,
            CrackFailure::UnsupportedFormat {
                display_name: "x".into(),
            },
            CrackFailure::EmptyWordlist,
            CrackFailure::FileUnreadable { reason: "r".into() },
            CrackFailure::NoCharsetSelected,
            CrackFailure::InvalidLengthRange {
                min: 1,
                max: 2,
                allowed_max: 3,
            },
            CrackFailure::InvalidMask {
                pattern: "?".into(),
                reason: "r".into(),
            },
            CrackFailure::InvalidRule {
                rule: "z".into(),
                reason: "r".into(),
            },
            CrackFailure::TooManyCombinations { total: 1, limit: 2 },
            CrackFailure::NoCandidate,
            CrackFailure::BackendUnavailable {
                backend: Backend::Rar,
                reason: "r".into(),
            },
            CrackFailure::SessionFingerprintMismatch,
        ];

        for locale in Locale::ALL {
            let messages = Messages::new(locale);
            for failure in &failures {
                let (key, _) = describe(failure);
                let text = messages.failure(failure);
                assert_ne!(text, key, "missing translation for {key} in {locale:?}");
                assert!(!text.is_empty());
            }
        }
    }

    #[test]
    fn a_failure_with_arguments_gets_them_substituted() {
        let messages = Messages::new(Locale::En);
        let text = messages.failure(&CrackFailure::TooManyCombinations {
            total: 20_000_000,
            limit: 10,
        });
        assert!(text.contains("20000000"), "unexpected text: {text}");
        assert!(text.contains("10"), "unexpected text: {text}");
    }

    #[test]
    fn a_technical_reason_is_never_shown_to_the_user() {
        let messages = Messages::new(Locale::En);
        let text = messages.failure(&CrackFailure::FileUnreadable {
            reason: "C:\\secret\\path: access denied".into(),
        });
        assert!(
            !text.contains("secret"),
            "leaked a technical reason: {text}"
        );
    }
}
