//! Tes end-to-end untuk PDF: rencana serangan, factory, penguji PDF, dan mesin paralel.
//!
//! Fixture-nya dibuat oleh `tests/fixtures/make_encrypted_pdf.py` dan di-commit ke
//! repositori. Skripnya disertakan supaya asal-usul berkas biner itu bisa diperiksa dan
//! fixture-nya bisa dibuat ulang, bukan sekadar berkas tanpa keterangan.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use zydra_core::attack::{AttackLimits, AttackPlan};
use zydra_core::engine::cancel::CancelToken;
use zydra_core::engine::crack::{run, EngineConfig, EngineEvent};
use zydra_core::model::target::CrackTarget;
use zydra_core::tester::factory::DefaultTesterFactory;
use zydra_core::tester::TesterFactory;

const SECRET: &str = "kr4k3n_1987";

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("encrypted.pdf")
}

fn write_wordlist(directory: &Path, lines: &[&str]) -> PathBuf {
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

#[test]
fn a_wordlist_finds_the_password_of_an_encrypted_pdf() {
    let directory = tempfile::tempdir().unwrap();
    let wordlist = write_wordlist(directory.path(), &["password", "123456", "letmein", SECRET]);

    let plan = AttackPlan::Dictionary { wordlist };
    let prepared = plan.prepare(&AttackLimits::default()).unwrap();
    assert_eq!(prepared.total, 4);

    let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());
    let events = collect(run(
        prepared.source,
        factory,
        CrackTarget::from_path(fixture()),
        config(),
        CancelToken::new(),
    ));

    assert_eq!(found_password(&events).as_deref(), Some(SECRET));
}

#[test]
fn a_wordlist_without_the_password_reports_exhaustion() {
    let directory = tempfile::tempdir().unwrap();
    let wordlist = write_wordlist(directory.path(), &["password", "123456"]);

    let plan = AttackPlan::Dictionary { wordlist };
    let prepared = plan.prepare(&AttackLimits::default()).unwrap();
    let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());
    let events = collect(run(
        prepared.source,
        factory,
        CrackTarget::from_path(fixture()),
        config(),
        CancelToken::new(),
    ));

    assert!(matches!(events.last(), Some(EngineEvent::Exhausted)));
    assert_eq!(found_password(&events), None);
}
