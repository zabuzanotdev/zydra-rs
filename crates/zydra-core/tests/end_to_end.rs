//! Tes end-to-end: rencana serangan, factory, penguji format, dan mesin paralel.
//!
//! Setiap format diuji lewat jalur yang sama persis dengan yang dipakai antarmuka, dari
//! `AttackPlan` sampai `engine::run`. Arsip ZIP dibuat di dalam tes, sedangkan PDF dan RAR
//! memakai fixture yang di-commit beserta keterangan asal-usulnya di `NOTICE`.

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
use zydra_core::model::failure::CrackFailure;
use zydra_core::model::target::CrackTarget;
use zydra_core::tester::factory::DefaultTesterFactory;
use zydra_core::tester::TesterFactory;

/// Password arsip ZIP yang dibuat di dalam tes.
const ZIP_SECRET: &str = "kr4k3n_1987";

/// Password fixture PDF, sama dengan yang dipakai skrip pembuatnya.
const PDF_SECRET: &str = "kr4k3n_1987";

/// Password fixture RAR dari libarchive. Berkas tes di repositori itu menyatakannya:
/// pada arsip solid, seluruh berkas memakai password `password`.
const RAR_SECRET: &str = "password";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Membuat arsip ZIP berenkripsi AES-256 di dalam sebuah direktori.
fn encrypted_zip(directory: &Path, password: &str) -> PathBuf {
    let path = directory.join("vault.zip");
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

fn wordlist(directory: &Path, lines: &[&str]) -> PathBuf {
    let path = directory.join("words.txt");
    let mut file = std::fs::File::create(&path).unwrap();
    for line in lines {
        writeln!(file, "{line}").unwrap();
    }
    path
}

fn config() -> EngineConfig {
    EngineConfig {
        workers: 2,
        batch_size: 4,
        progress_interval: Duration::from_millis(5),
    }
}

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

fn found_password(events: &[EngineEvent]) -> Option<String> {
    events.iter().find_map(|event| match event {
        EngineEvent::Found { password, .. } => Some(password.clone()),
        _ => None,
    })
}

/// Menjalankan serangan wordlist lengkap dan mengembalikan peristiwanya.
fn run_wordlist(target: &Path, words: PathBuf) -> Vec<EngineEvent> {
    let plan = AttackPlan::Dictionary { wordlist: words };
    let prepared = plan.prepare(&AttackLimits::default()).unwrap();
    let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());
    collect(run(
        prepared.source,
        factory,
        CrackTarget::from_path(target),
        config(),
        CancelToken::new(),
    ))
}

#[test]
fn a_wordlist_finds_the_password_of_an_encrypted_zip() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_zip(directory.path(), ZIP_SECRET);
    let words = wordlist(
        directory.path(),
        &["password", "123456", "letmein", ZIP_SECRET],
    );

    let events = run_wordlist(&archive, words);
    assert_eq!(found_password(&events).as_deref(), Some(ZIP_SECRET));
}

#[test]
fn a_wordlist_without_the_zip_password_reports_exhaustion() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_zip(directory.path(), ZIP_SECRET);
    let words = wordlist(directory.path(), &["password", "123456"]);

    let events = run_wordlist(&archive, words);
    assert!(matches!(events.last(), Some(EngineEvent::Exhausted)));
    assert_eq!(found_password(&events), None);
}

#[test]
fn a_brute_force_search_finds_a_two_digit_zip_password() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_zip(directory.path(), "42");

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

    assert_eq!(found_password(&events).as_deref(), Some("42"));
}

#[test]
fn a_mask_search_finds_a_zip_password() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_zip(directory.path(), "z9");

    let plan = AttackPlan::Mask {
        pattern: "?l?d".to_string(),
    };
    let prepared = plan.prepare(&AttackLimits::default()).unwrap();
    assert_eq!(prepared.total, 26 * 10);

    let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());
    let events = collect(run(
        prepared.source,
        factory,
        CrackTarget::from_path(&archive),
        config(),
        CancelToken::new(),
    ));

    assert_eq!(found_password(&events).as_deref(), Some("z9"));
}

#[test]
fn rules_let_a_lowercase_wordlist_find_a_capitalised_password() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_zip(directory.path(), "Admin7");
    let words = wordlist(directory.path(), &["admin", "other"]);

    let plan = AttackPlan::WithRules {
        base: Box::new(AttackPlan::Dictionary { wordlist: words }),
        rules: "capitalize+append-digits:9".to_string(),
    };
    let prepared = plan.prepare(&AttackLimits::default()).unwrap();
    assert_eq!(prepared.total, 2 * 10);

    let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());
    let events = collect(run(
        prepared.source,
        factory,
        CrackTarget::from_path(&archive),
        config(),
        CancelToken::new(),
    ));

    assert_eq!(found_password(&events).as_deref(), Some("Admin7"));
}

#[test]
fn a_wordlist_finds_the_password_of_an_encrypted_pdf() {
    let directory = tempfile::tempdir().unwrap();
    let words = wordlist(
        directory.path(),
        &["password", "123456", "letmein", PDF_SECRET],
    );

    let events = run_wordlist(&fixture("encrypted.pdf"), words);
    assert_eq!(found_password(&events).as_deref(), Some(PDF_SECRET));
}

#[test]
fn a_wordlist_without_the_pdf_password_reports_exhaustion() {
    let directory = tempfile::tempdir().unwrap();
    let words = wordlist(directory.path(), &["password", "123456"]);

    let events = run_wordlist(&fixture("encrypted.pdf"), words);
    assert!(matches!(events.last(), Some(EngineEvent::Exhausted)));
}

#[test]
fn a_wordlist_finds_the_password_of_a_solid_rar4_archive() {
    let directory = tempfile::tempdir().unwrap();
    let words = wordlist(directory.path(), &["letmein", RAR_SECRET]);

    let events = run_wordlist(&fixture("rar4_solid_encrypted.rar"), words);
    assert_eq!(found_password(&events).as_deref(), Some(RAR_SECRET));
}

#[test]
fn a_wordlist_finds_the_password_of_a_rar5_archive() {
    let directory = tempfile::tempdir().unwrap();
    let words = wordlist(directory.path(), &["letmein", RAR_SECRET]);

    let events = run_wordlist(&fixture("rar5_encrypted.rar"), words);
    assert_eq!(found_password(&events).as_deref(), Some(RAR_SECRET));
}

#[test]
fn a_wordlist_without_the_rar_password_reports_exhaustion() {
    let directory = tempfile::tempdir().unwrap();
    // Sengaja tidak memuat `password`, karena itulah password arsip RAR-nya.
    let words = wordlist(directory.path(), &["letmein", "123456"]);

    let events = run_wordlist(&fixture("rar4_solid_encrypted.rar"), words);
    assert!(matches!(events.last(), Some(EngineEvent::Exhausted)));
}

#[test]
fn an_unsupported_target_fails_before_any_work_starts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("notes.txt");
    std::fs::write(&path, b"nothing to see").unwrap();
    let words = wordlist(directory.path(), &["a", "b"]);

    let events = run_wordlist(&path, words);
    assert!(matches!(
        events.last(),
        Some(EngineEvent::Failed(CrackFailure::UnsupportedFormat { .. }))
    ));
}
