//! Tes end-to-end: rencana serangan, factory, penguji ZIP, dan mesin paralel sekaligus.
//!
//! Arsip uji dibuat di dalam tes memakai crate `zip`, jadi repositori ini tidak perlu
//! menyimpan berkas biner apa pun untuk menguji jalur ZIP.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use zip::write::SimpleFileOptions;
use zip::AesMode;

use zydra_core::attack::{AttackLimits, AttackPlan};
use zydra_core::engine::cancel::CancelToken;
use zydra_core::engine::crack::{run, EngineConfig, EngineEvent};
use zydra_core::model::charset::CharsetSet;
use zydra_core::model::target::CrackTarget;
use zydra_core::tester::factory::DefaultTesterFactory;
use zydra_core::tester::TesterFactory;

const SECRET: &str = "kr4k3n_1987";
const DIGIT_SECRET: &str = "42";

/// Membuat arsip ZIP berenkripsi AES-256 di dalam `directory`.
fn encrypted_archive(directory: &Path, name: &str, password: &str) -> PathBuf {
    let path = directory.join(name);
    let file = std::fs::File::create(&path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().with_aes_encryption(AesMode::Aes256, password);
    writer.start_file("secret.txt", options).unwrap();
    writer
        .write_all(b"isi rahasia yang panjangnya melewati satu buffer baca")
        .unwrap();
    writer.finish().unwrap();
    path
}

fn write_wordlist(directory: &Path, lines: &[&str]) -> PathBuf {
    let path = directory.join("words.txt");
    let mut file = std::fs::File::create(&path).unwrap();
    for line in lines {
        writeln!(file, "{line}").unwrap();
    }
    path
}

/// Membaca peristiwa sampai peristiwa penutup pertama, atau sampai waktu habis.
fn collect(events: crossbeam_channel::Receiver<EngineEvent>) -> Vec<EngineEvent> {
    let mut collected = Vec::new();
    while let Ok(event) = events.recv_timeout(Duration::from_secs(30)) {
        let terminal = matches!(
            event,
            EngineEvent::Found { .. }
                | EngineEvent::Exhausted
                | EngineEvent::Cancelled
                | EngineEvent::Failed(_)
        );
        collected.push(event);
        if terminal {
            break;
        }
    }
    collected
}

fn config() -> EngineConfig {
    EngineConfig {
        workers: 2,
        batch_size: 4,
        progress_interval: Duration::from_millis(5),
    }
}

fn found_password(events: &[EngineEvent]) -> Option<String> {
    events.iter().find_map(|event| match event {
        EngineEvent::Found { password, .. } => Some(password.clone()),
        _ => None,
    })
}

#[test]
fn a_wordlist_finds_the_password_of_an_encrypted_zip() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), "vault.zip", SECRET);
    let wordlist = write_wordlist(
        directory.path(),
        &["password", "123456", "letmein", SECRET, "unused"],
    );

    let plan = AttackPlan::Dictionary { wordlist };
    let prepared = plan.prepare(&AttackLimits::default()).unwrap();
    assert_eq!(prepared.total, 5);

    let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());
    let events = collect(run(
        prepared.source,
        factory,
        CrackTarget::from_path(&archive),
        config(),
        CancelToken::new(),
    ));

    assert_eq!(found_password(&events).as_deref(), Some(SECRET));
}

#[test]
fn a_wordlist_without_the_password_reports_exhaustion() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), "vault.zip", SECRET);
    let wordlist = write_wordlist(directory.path(), &["password", "123456", "letmein"]);

    let plan = AttackPlan::Dictionary { wordlist };
    let prepared = plan.prepare(&AttackLimits::default()).unwrap();
    let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());
    let events = collect(run(
        prepared.source,
        factory,
        CrackTarget::from_path(&archive),
        config(),
        CancelToken::new(),
    ));

    assert!(matches!(events.last(), Some(EngineEvent::Exhausted)));
    assert_eq!(found_password(&events), None);
}

#[test]
fn a_brute_force_search_finds_a_two_digit_password() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), "digits.zip", DIGIT_SECRET);

    let plan = AttackPlan::BruteForce {
        charset: CharsetSet::DIGITS,
        min_length: 1,
        max_length: 2,
    };
    let prepared = plan.prepare(&AttackLimits::default()).unwrap();
    assert_eq!(prepared.total, 10 + 100);

    let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());
    let events = collect(run(
        prepared.source,
        factory,
        CrackTarget::from_path(&archive),
        config(),
        CancelToken::new(),
    ));

    assert_eq!(found_password(&events).as_deref(), Some(DIGIT_SECRET));
}

#[test]
fn an_unsupported_target_fails_before_any_work_starts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("notes.txt");
    std::fs::write(&path, b"nothing to see").unwrap();
    let wordlist = write_wordlist(directory.path(), &["a", "b"]);

    let plan = AttackPlan::Dictionary { wordlist };
    let prepared = plan.prepare(&AttackLimits::default()).unwrap();
    let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());
    let events = collect(run(
        prepared.source,
        factory,
        CrackTarget::from_path(&path),
        config(),
        CancelToken::new(),
    ));

    assert!(matches!(
        events.last(),
        Some(EngineEvent::Failed(
            zydra_core::model::failure::CrackFailure::UnsupportedFormat { .. }
        ))
    ));
}
