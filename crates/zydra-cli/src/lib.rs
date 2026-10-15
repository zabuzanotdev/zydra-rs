//! Baris perintah Zydra.
//!
//! Crate ini hanya mengurus masukan, keluaran, dan kode keluar. Semua aturan pemulihan
//! password tinggal di `zydra-core`, dan semua kalimat tinggal di `zydra-i18n`.

pub mod cli;
pub mod json;
pub mod render;

use std::sync::Arc;
use std::time::{Duration, Instant};

use zydra_core::attack::{AttackLimits, AttackPlan};
use zydra_core::constants::SESSION_SAVE_INTERVAL_SECS;
use zydra_core::engine::cancel::CancelToken;
use zydra_core::engine::crack::{run, EngineConfig, EngineEvent};
use zydra_core::log::{LogEntry, LogLevel};
use zydra_core::model::charset::CharsetSet;
use zydra_core::model::failure::{Backend, CrackFailure};
use zydra_core::model::target::CrackTarget;
use zydra_core::session::store::SessionLookup;
use zydra_core::session::SessionTracker;
use zydra_core::tester::factory::DefaultTesterFactory;
use zydra_core::tester::{EncryptionKind, TesterFactory};
use zydra_i18n::{format, Locale, Messages};

use crate::cli::Args;
use crate::render::Reporter;

/// Selesai tanpa masalah. `--help` dan `--version` juga memakai kode ini.
pub const EXIT_OK: u8 = 0;
/// Password ditemukan.
pub const EXIT_FOUND: u8 = EXIT_OK;
/// Tidak ada kandidat yang cocok.
pub const EXIT_NOT_FOUND: u8 = 1;
/// Argumen tidak sah.
pub const EXIT_USAGE: u8 = 2;
/// Berkas target atau backend bermasalah.
pub const EXIT_TARGET: u8 = 3;

/// Menjalankan satu kali pemulihan dan mengembalikan kode keluar.
pub fn run_with_args(args: &[String]) -> u8 {
    // Bahasa harus diketahui sebelum clap dibangun, karena teks bantuan ikut diterjemahkan.
    let messages = match resolve_messages(args) {
        Ok(messages) => messages,
        Err(value) => {
            let messages = Messages::detected();
            eprintln!(
                "{}",
                messages.text_args("usage-bad-language", &[("value", &value)])
            );
            return EXIT_USAGE;
        }
    };

    let parsed = match cli::parse_from(&messages, args) {
        Ok(parsed) => parsed,
        Err(error) => {
            let kind = error.kind();
            let _ = error.print();
            // `--help` dan `--version` bukan kesalahan: pengguna memintanya, dan clap
            // mencetak jawabannya. Hanya argumen yang benar-benar tidak sah yang dihitung
            // sebagai kesalahan pemakaian.
            return match kind {
                clap::error::ErrorKind::DisplayHelp
                | clap::error::ErrorKind::DisplayVersion
                | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => EXIT_OK,
                _ => EXIT_USAGE,
            };
        }
    };

    let reporter = Reporter::new(messages, parsed.quiet, parsed.json);
    execute(&parsed, &reporter)
}

/// Menentukan bahasa dari `--lang`, atau dari pengaturan sistem.
fn resolve_messages(args: &[String]) -> Result<Messages, String> {
    match cli::scan_language(args) {
        Some(value) => match Locale::parse(&value) {
            Some(locale) => Ok(Messages::new(locale)),
            None => Err(value),
        },
        None => Ok(Messages::detected()),
    }
}

/// Menyusun rencana serangan dari argumen.
fn build_plan(args: &Args, messages: &Messages) -> Result<AttackPlan, String> {
    if args.file.is_none() {
        return Err(messages.text("usage-missing-target"));
    }

    let methods = [
        args.wordlist.is_some(),
        args.charset.is_some(),
        args.mask.is_some(),
    ];
    let chosen = methods.iter().filter(|chosen| **chosen).count();
    if chosen == 0 {
        return Err(messages.text("usage-missing-method"));
    }
    if chosen > 1 {
        return Err(messages.text("usage-conflicting-methods"));
    }

    let base = if let Some(wordlist) = args.wordlist.as_ref() {
        AttackPlan::Dictionary {
            wordlist: wordlist.clone(),
        }
    } else if let Some(mask) = args.mask.as_ref() {
        AttackPlan::Mask {
            pattern: mask.clone(),
        }
    } else {
        let charset = parse_charset(args.charset.as_deref().expect("checked above"))
            .map_err(|name| messages.text_args("usage-bad-charset", &[("name", &name)]))?;
        let Some(min_length) = args.min_length else {
            return Err(messages.text("usage-bad-length"));
        };
        let Some(max_length) = args.max_length else {
            return Err(messages.text("usage-bad-length"));
        };
        AttackPlan::BruteForce {
            charset,
            min_length,
            max_length,
        }
    };

    // Aturan membungkus metode yang dipilih, bukan menjadi metode tersendiri. Dengan
    // begitu `--rules` bisa dipakai bersama wordlist maupun mask tanpa menambah cabang
    // di mana pun.
    match args.rules.as_deref() {
        Some(rules) if !rules.trim().is_empty() => Ok(AttackPlan::WithRules {
            base: Box::new(base),
            rules: rules.to_string(),
        }),
        _ => Ok(base),
    }
}

/// Mengurai kelas karakter, menerima `letters` sebagai nama lama.
///
/// Skrip Python menerima `letters` yang berarti gabungan huruf kecil dan huruf besar.
/// Alias itu ditangani di lapisan ini supaya `CharsetSet` hanya punya satu himpunan nama
/// yang sah.
fn parse_charset(spec: &str) -> Result<CharsetSet, String> {
    let expanded = spec
        .split(',')
        .map(|part| match part.trim() {
            "letters" => "lowercase,uppercase",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(",");
    CharsetSet::parse(&expanded).map_err(|error| match error {
        zydra_core::error::CoreError::CharsetParse { name } => name,
        other => other.to_string(),
    })
}

/// Menjalankan rencana yang sudah tersusun.
fn execute(args: &Args, reporter: &Reporter) -> u8 {
    let messages = reporter.messages();

    let plan = match build_plan(args, messages) {
        Ok(plan) => plan,
        Err(text) => {
            eprintln!("{text}");
            return EXIT_USAGE;
        }
    };

    let target = CrackTarget::from_path(args.file.clone().expect("validated above"));

    let mut prepared = match plan.prepare(&AttackLimits::default()) {
        Ok(prepared) => prepared,
        Err(failure) => {
            reporter.failure(&failure);
            return exit_code_for(&failure);
        }
    };

    let sessions = SessionTracker::for_target(&target);
    let mut start_position = 0;

    if args.resume {
        match sessions.lookup() {
            Some(Ok(SessionLookup::Found(state))) => {
                start_position = state.position.min(prepared.total);
                reporter.log(
                    &LogEntry::now(LogLevel::Info, "log-session-resumed")
                        .with_arg("position", format::count(start_position))
                        .with_arg("total", format::count(prepared.total)),
                );
            }
            Some(Ok(SessionLookup::Absent)) => {
                reporter.log(&LogEntry::now(LogLevel::Info, "log-session-absent"));
            }
            // Berkasnya sudah berubah, jadi posisi lama tidak lagi berarti. Melanjutkan
            // akan melewati kandidat yang seharusnya dicoba, dan memulai diam-diam dari
            // awal akan membuat pengguna mengira serangannya dilanjutkan.
            Some(Ok(SessionLookup::Mismatch)) => {
                let failure = CrackFailure::SessionFingerprintMismatch;
                if reporter.is_json() {
                    reporter.json_line(&[
                        ("kind", json::string("failed")),
                        ("code", json::string(failure_code(&failure))),
                    ]);
                }
                reporter.failure(&failure);
                return exit_code_for(&failure);
            }
            // Sesi rusak atau tidak bisa dibaca. Serangan tetap berjalan dari awal, dan
            // sebabnya masuk ke catatan alih-alih menghentikan pekerjaan.
            Some(Err(error)) => {
                reporter.log(
                    &LogEntry::now(LogLevel::Warn, "error-unknown")
                        .with_arg("detail", error.to_string()),
                );
            }
            None => {}
        }
    }

    if start_position > 0 {
        prepared.source.skip(start_position);
    }

    let method_label = messages.text(method_key(&plan));
    let method_value = match &plan {
        AttackPlan::WithRules { rules, .. } => format!("{method_label} + {rules}"),
        _ => method_label,
    };

    reporter.header(&[
        ("cli-target", target.display_name.clone()),
        ("cli-method", method_value),
        ("cli-candidates", prepared.total.to_string()),
    ]);

    let total = prepared.total;
    let cancel = CancelToken::new();
    let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());
    let config = EngineConfig {
        workers: args.threads,
        ..EngineConfig::default()
    };

    // Membuka target sekali sebelum pekerjaan dimulai. Tanpa ini, berkas yang tidak
    // terenkripsi dan backend yang bermasalah baru ketahuan setelah bilah progres sempat
    // muncul, dan pengguna menunggu untuk sesuatu yang memang tidak akan berjalan.
    match factory.create(&target) {
        Ok(tester) if tester.info().encryption == EncryptionKind::None => {
            let failure = CrackFailure::TargetNotEncrypted;
            if reporter.is_json() {
                reporter.json_line(&[
                    ("kind", json::string("failed")),
                    ("code", json::string(failure_code(&failure))),
                ]);
            }
            reporter.failure(&failure);
            return exit_code_for(&failure);
        }
        Ok(_) => {}
        Err(failure) => {
            if reporter.is_json() {
                reporter.json_line(&[
                    ("kind", json::string("failed")),
                    ("code", json::string(failure_code(&failure))),
                ]);
            }
            reporter.failure(&failure);
            return exit_code_for(&failure);
        }
    }

    let started = Instant::now();
    let events = run(prepared.source, factory, target, config, cancel);
    let bar = reporter.progress_bar(total);

    // Posisi awal disimpan juga, supaya serangan yang dihentikan sangat awal tetap punya
    // titik lanjut.
    sessions.save(&plan, start_position);
    let save_interval = Duration::from_secs(SESSION_SAVE_INTERVAL_SECS);
    let mut last_save = Instant::now();

    let mut outcome = Outcome::Unknown;
    while let Ok(event) = events.recv() {
        match event {
            EngineEvent::Started { total } => {
                if reporter.is_json() {
                    reporter.json_line(&[
                        ("kind", json::string("started")),
                        ("total", total.to_string()),
                    ]);
                }
            }
            EngineEvent::Progress(data) => {
                let progress = zydra_core::model::progress::CrackProgress::Trying(data);
                if let Some(bar) = bar.as_ref() {
                    reporter.progress(bar, &progress);
                }
                if reporter.is_json() {
                    reporter.json_line(&[
                        ("kind", json::string("progress")),
                        ("tried", data.tried.to_string()),
                        ("total", data.total.to_string()),
                        ("speed", data.speed_per_second.to_string()),
                    ]);
                }
                if last_save.elapsed() >= save_interval {
                    sessions.save(&plan, absolute_position(start_position, data.tried, total));
                    last_save = Instant::now();
                }
            }
            EngineEvent::Found { password, detail } => {
                outcome = Outcome::Found { password, detail };
                break;
            }
            EngineEvent::Exhausted => {
                outcome = Outcome::Exhausted;
                break;
            }
            EngineEvent::Cancelled => {
                outcome = Outcome::Cancelled;
                break;
            }
            EngineEvent::Failed(failure) => {
                outcome = Outcome::Failed(failure);
                break;
            }
        }
    }

    if let Some(bar) = bar {
        bar.finish_and_clear();
    }
    let elapsed = started.elapsed();

    match outcome {
        Outcome::Found { password, detail } => {
            // Serangan selesai sepenuhnya, jadi tidak ada yang perlu dilanjutkan.
            sessions.clear();
            if reporter.is_json() {
                let mut fields = vec![
                    ("kind", json::string("found")),
                    ("password", json::string(&password)),
                    ("elapsed_seconds", elapsed.as_secs().to_string()),
                ];
                if let Some(detail) = detail.as_ref() {
                    fields.push(("detail", json::string(detail)));
                }
                reporter.json_line(&fields);
            } else {
                reporter.success(&password, detail.as_deref());
                reporter.footer(elapsed);
            }
            EXIT_FOUND
        }
        Outcome::Exhausted => {
            sessions.clear();
            if reporter.is_json() {
                reporter.json_line(&[
                    ("kind", json::string("exhausted")),
                    ("elapsed_seconds", elapsed.as_secs().to_string()),
                ]);
            } else {
                reporter.not_found();
                reporter.footer(elapsed);
            }
            EXIT_NOT_FOUND
        }
        Outcome::Cancelled => {
            reporter.json_line(&[("kind", json::string("cancelled"))]);
            EXIT_NOT_FOUND
        }
        Outcome::Failed(failure) => {
            if reporter.is_json() {
                reporter.json_line(&[
                    ("kind", json::string("failed")),
                    ("code", json::string(failure_code(&failure))),
                ]);
            }
            reporter.failure(&failure);
            exit_code_for(&failure)
        }
        Outcome::Unknown => {
            // Kanal tertutup tanpa peristiwa penutup: worker berhenti tanpa lapor.
            let failure = CrackFailure::NoCandidate;
            if reporter.is_json() {
                reporter.json_line(&[
                    ("kind", json::string("failed")),
                    ("code", json::string("incomplete")),
                ]);
            }
            reporter.failure(&failure);
            EXIT_TARGET
        }
    }
}

/// Posisi absolut dari sebuah kemajuan.
///
/// Mesin menghitung kandidat yang dicoba sejak ia dijalankan, bukan sejak awal ruang
/// pencarian. Setelah serangan dilanjutkan dari posisi tertentu, angka itu harus
/// ditambahkan ke titik awalnya. Tanpa itu posisi yang tersimpan akan mundur setiap kali
/// serangan dilanjutkan lalu dihentikan lagi, dan kemajuannya hilang sedikit demi sedikit.
fn absolute_position(start: u64, tried: u64, total: u64) -> u64 {
    start.saturating_add(tried).min(total)
}

/// Hasil akhir satu kali jalan.
enum Outcome {
    Found {
        password: String,
        detail: Option<String>,
    },
    Exhausted,
    Cancelled,
    Failed(CrackFailure),
    Unknown,
}

/// Kunci terjemahan untuk metode sebuah rencana.
///
/// Rencana dengan aturan memakai metode dasarnya, karena aturan adalah lapisan tambahan di
/// atas metode, bukan metode tersendiri.
fn method_key(plan: &AttackPlan) -> &'static str {
    match plan {
        AttackPlan::Dictionary { .. } => "cli-method-dictionary",
        AttackPlan::BruteForce { .. } => "cli-method-brute-force",
        AttackPlan::Mask { .. } => "cli-method-mask",
        AttackPlan::WithRules { base, .. } => method_key(base),
    }
}

/// Memetakan kegagalan ke kode keluar.
///
/// Pemisahannya disengaja: masalah argumen adalah kesalahan pemakaian, sedangkan masalah
/// berkas dan backend adalah kesalahan lingkungan. Skrip yang memanggil Zydra bisa
/// membedakan keduanya tanpa membaca pesan.
fn exit_code_for(failure: &CrackFailure) -> u8 {
    match failure {
        CrackFailure::FileUnreadable { .. }
        | CrackFailure::BackendUnavailable { .. }
        | CrackFailure::UnsupportedFormat { .. }
        | CrackFailure::TargetNotEncrypted
        | CrackFailure::SessionFingerprintMismatch => EXIT_TARGET,
        CrackFailure::PasswordNotFound | CrackFailure::NoCandidate => EXIT_NOT_FOUND,
        _ => EXIT_USAGE,
    }
}

/// Penanda teknis untuk keluaran terstruktur.
fn failure_code(failure: &CrackFailure) -> &'static str {
    match failure {
        CrackFailure::PasswordNotFound => "password-not-found",
        CrackFailure::UnsupportedFormat { .. } => "unsupported-format",
        CrackFailure::EmptyWordlist => "empty-wordlist",
        CrackFailure::FileUnreadable { .. } => "file-unreadable",
        CrackFailure::NoCharsetSelected => "no-charset",
        CrackFailure::InvalidLengthRange { .. } => "invalid-length-range",
        CrackFailure::InvalidMask { .. } => "invalid-mask",
        CrackFailure::InvalidRule { .. } => "invalid-rule",
        CrackFailure::TooManyCombinations { .. } => "too-many-combinations",
        CrackFailure::NoCandidate => "no-candidate",
        CrackFailure::TargetNotEncrypted => "target-not-encrypted",
        CrackFailure::BackendUnavailable { backend, .. } => match backend {
            Backend::Zip => "backend-unavailable-zip",
            Backend::Rar => "backend-unavailable-rar",
            Backend::Pdf => "backend-unavailable-pdf",
            Backend::Shadow => "backend-unavailable-shadow",
        },
        CrackFailure::SessionFingerprintMismatch => "session-mismatch",
    }
}

/// Jeda antar peristiwa progres yang dipakai mesin.
pub fn default_progress_interval() -> Duration {
    EngineConfig::default().progress_interval
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Argumen kosong, untuk dipakai dengan sintaks pembaruan struktur.
    fn args() -> Args {
        Args {
            file: None,
            wordlist: None,
            charset: None,
            min_length: None,
            max_length: None,
            mask: None,
            rules: None,
            threads: 0,
            lang: None,
            json: false,
            quiet: false,
            resume: false,
        }
    }

    #[test]
    fn letters_is_accepted_as_the_old_name_for_both_cases() {
        assert_eq!(
            parse_charset("letters").unwrap(),
            CharsetSet::LOWERCASE | CharsetSet::UPPERCASE
        );
    }

    #[test]
    fn letters_can_be_combined_with_other_classes() {
        assert_eq!(
            parse_charset("letters,digits").unwrap(),
            CharsetSet::LOWERCASE | CharsetSet::UPPERCASE | CharsetSet::DIGITS
        );
    }

    #[test]
    fn an_unknown_class_is_reported_by_name() {
        assert_eq!(parse_charset("lowercase,nope").unwrap_err(), "nope");
    }

    #[test]
    fn argument_problems_are_usage_errors() {
        assert_eq!(exit_code_for(&CrackFailure::EmptyWordlist), EXIT_USAGE);
        assert_eq!(
            exit_code_for(&CrackFailure::TooManyCombinations { total: 1, limit: 2 }),
            EXIT_USAGE
        );
        assert_eq!(exit_code_for(&CrackFailure::NoCharsetSelected), EXIT_USAGE);
    }

    #[test]
    fn a_bad_mask_or_rule_is_a_usage_error() {
        assert_eq!(
            exit_code_for(&CrackFailure::InvalidMask {
                pattern: "?".into(),
                reason: "x".into()
            }),
            EXIT_USAGE
        );
        assert_eq!(
            exit_code_for(&CrackFailure::InvalidRule {
                rule: "nope".into(),
                reason: "x".into()
            }),
            EXIT_USAGE
        );
    }

    #[test]
    fn file_problems_are_target_errors() {
        assert_eq!(
            exit_code_for(&CrackFailure::FileUnreadable { reason: "x".into() }),
            EXIT_TARGET
        );
        assert_eq!(
            exit_code_for(&CrackFailure::BackendUnavailable {
                backend: Backend::Rar,
                reason: "x".into()
            }),
            EXIT_TARGET
        );
        assert_eq!(
            exit_code_for(&CrackFailure::UnsupportedFormat {
                display_name: "x".into()
            }),
            EXIT_TARGET
        );
    }

    #[test]
    fn a_missing_password_is_not_an_error() {
        assert_eq!(
            exit_code_for(&CrackFailure::PasswordNotFound),
            EXIT_NOT_FOUND
        );
    }

    #[test]
    fn every_failure_has_a_stable_machine_code() {
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
                backend: Backend::Pdf,
                reason: "r".into(),
            },
            CrackFailure::SessionFingerprintMismatch,
        ];
        for failure in &failures {
            let code = failure_code(failure);
            assert!(!code.is_empty());
            assert!(!code.contains(' '), "code must be machine friendly: {code}");
        }
    }

    #[test]
    fn building_a_plan_without_a_target_is_rejected() {
        let messages = Messages::new(Locale::En);
        let args = Args {
            wordlist: Some("w.txt".into()),
            ..args()
        };
        assert_eq!(
            build_plan(&args, &messages).unwrap_err(),
            "Choose a target file."
        );
    }

    #[test]
    fn asking_for_both_methods_at_once_is_rejected() {
        let messages = Messages::new(Locale::En);
        let args = Args {
            file: Some("v.zip".into()),
            wordlist: Some("w.txt".into()),
            charset: Some("digits".into()),
            min_length: Some(1),
            max_length: Some(2),
            ..args()
        };
        assert_eq!(
            build_plan(&args, &messages).unwrap_err(),
            "Choose either a wordlist or a brute force, not both."
        );
    }

    #[test]
    fn asking_for_a_mask_and_a_wordlist_at_once_is_rejected() {
        let messages = Messages::new(Locale::En);
        let args = Args {
            file: Some("v.zip".into()),
            wordlist: Some("w.txt".into()),
            mask: Some("?d".into()),
            ..args()
        };
        assert_eq!(
            build_plan(&args, &messages).unwrap_err(),
            "Choose either a wordlist or a brute force, not both."
        );
    }

    #[test]
    fn a_brute_force_without_lengths_is_rejected() {
        let messages = Messages::new(Locale::En);
        let args = Args {
            file: Some("v.zip".into()),
            charset: Some("digits".into()),
            min_length: Some(1),
            ..args()
        };
        assert_eq!(
            build_plan(&args, &messages).unwrap_err(),
            "Minimum and maximum must be numbers, and minimum must not exceed maximum."
        );
    }

    #[test]
    fn a_mask_becomes_a_mask_plan_without_needing_lengths() {
        let messages = Messages::new(Locale::En);
        let args = Args {
            file: Some("v.zip".into()),
            mask: Some("?l?l?d".into()),
            ..args()
        };
        match build_plan(&args, &messages).unwrap() {
            AttackPlan::Mask { pattern } => assert_eq!(pattern, "?l?l?d"),
            other => panic!("unexpected plan: {other:?}"),
        }
    }

    #[test]
    fn rules_wrap_whichever_method_was_chosen() {
        let messages = Messages::new(Locale::En);
        let args = Args {
            file: Some("v.zip".into()),
            wordlist: Some("w.txt".into()),
            rules: Some("capitalize".into()),
            ..args()
        };
        match build_plan(&args, &messages).unwrap() {
            AttackPlan::WithRules { base, rules } => {
                assert!(matches!(*base, AttackPlan::Dictionary { .. }));
                assert_eq!(rules, "capitalize");
            }
            other => panic!("unexpected plan: {other:?}"),
        }
    }

    #[test]
    fn an_empty_rules_argument_is_ignored() {
        let messages = Messages::new(Locale::En);
        let args = Args {
            file: Some("v.zip".into()),
            mask: Some("?d".into()),
            rules: Some("   ".into()),
            ..args()
        };
        assert!(matches!(
            build_plan(&args, &messages).unwrap(),
            AttackPlan::Mask { .. }
        ));
    }

    #[test]
    fn a_plan_with_rules_reports_the_method_of_its_base() {
        let plan = AttackPlan::WithRules {
            base: Box::new(AttackPlan::Mask {
                pattern: "?d".into(),
            }),
            rules: "leet".into(),
        };
        assert_eq!(method_key(&plan), "cli-method-mask");
    }

    #[test]
    fn an_unknown_language_is_rejected_before_parsing() {
        let args = ["zydra", "--lang", "fr"].map(String::from);
        assert_eq!(resolve_messages(&args).unwrap_err(), "fr");
    }

    #[test]
    fn a_known_language_selects_its_catalogue() {
        let args = ["zydra", "--lang", "id"].map(String::from);
        assert_eq!(resolve_messages(&args).unwrap().locale(), Locale::Id);
    }

    #[test]
    fn a_resumed_position_is_the_start_plus_what_this_run_tried() {
        // Regresi: tanpa penambahan ini, melanjutkan lalu menghentikan lagi akan menulis
        // posisi yang lebih kecil daripada yang dibaca, dan kemajuannya hilang.
        assert_eq!(absolute_position(8_302, 100, 1_000_000), 8_402);
    }

    #[test]
    fn the_absolute_position_never_passes_the_total() {
        assert_eq!(absolute_position(90, 100, 100), 100);
    }

    #[test]
    fn the_absolute_position_does_not_overflow() {
        assert_eq!(absolute_position(u64::MAX - 1, 10, u64::MAX), u64::MAX);
    }
}
