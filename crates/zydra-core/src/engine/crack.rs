//! Menjalankan penelusuran kandidat secara paralel.
//!
//! Modul ini tidak tahu format apa pun dan tidak tahu dari mana kandidat berasal; ia
//! hanya mengenal [`CandidateSource`] dan [`TesterFactory`].

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver};

use crate::candidate::CandidateSource;
use crate::constants::{
    DEFAULT_BATCH_SIZE, DEFAULT_PROGRESS_INTERVAL_MS, DEFAULT_WORKERS, MIN_WORKERS,
};
use crate::engine::cancel::CancelToken;
use crate::engine::stats::SpeedMeter;
use crate::model::failure::CrackFailure;
use crate::model::progress::TryingData;
use crate::model::target::CrackTarget;
use crate::tester::{MatchOutcome, PasswordTester, TesterFactory};

/// Pengaturan satu kali penelusuran.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Jumlah worker. `0` berarti mengikuti jumlah CPU.
    pub workers: usize,
    /// Jumlah kandidat yang diambil satu worker dalam sekali kunci.
    pub batch_size: usize,
    /// Jarak antar peristiwa progres.
    pub progress_interval: Duration,
}

impl Default for EngineConfig {
    fn default() -> EngineConfig {
        EngineConfig {
            workers: DEFAULT_WORKERS,
            batch_size: DEFAULT_BATCH_SIZE,
            progress_interval: Duration::from_millis(DEFAULT_PROGRESS_INTERVAL_MS),
        }
    }
}

/// Peristiwa yang dikirim mesin ke pemanggil.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    /// Penelusuran dimulai. Selalu peristiwa pertama.
    Started {
        /// Jumlah kandidat yang akan dicoba.
        total: u64,
    },
    /// Kemajuan terbaru.
    Progress(TryingData),
    /// Password ditemukan. Peristiwa ini menutup penelusuran.
    Found {
        /// Password yang cocok.
        password: String,
        /// Keterangan tambahan dari backend.
        detail: Option<String>,
    },
    /// Semua kandidat habis tanpa hasil.
    Exhausted,
    /// Penelusuran dihentikan atas permintaan.
    Cancelled,
    /// Penelusuran tidak bisa dijalankan.
    Failed(CrackFailure),
}

/// Menjalankan penelusuran dan segera mengembalikan kanal peristiwa.
///
/// Pekerjaan berjalan di thread terpisah, jadi pemanggil bisa membaca kanal dari loop
/// gambar antarmuka atau loop keluaran terminal tanpa pernah membeku. Pemanggil berhenti
/// membaca setelah peristiwa penutup pertama (`Found`, `Exhausted`, `Cancelled`, atau
/// `Failed`).
pub fn run(
    source: Box<dyn CandidateSource>,
    factory: Arc<dyn TesterFactory>,
    target: CrackTarget,
    config: EngineConfig,
    cancel: CancelToken,
) -> Receiver<EngineEvent> {
    let (sender, receiver) = unbounded();

    let total = source.total();
    if total == 0 {
        let _ = sender.send(EngineEvent::Failed(CrackFailure::NoCandidate));
        return receiver;
    }

    let workers = if config.workers == DEFAULT_WORKERS {
        num_cpus::get()
    } else {
        config.workers
    }
    .max(MIN_WORKERS);
    let batch_size = config.batch_size.max(1);
    let interval = config.progress_interval;

    thread::spawn(move || {
        let _ = sender.send(EngineEvent::Started { total });

        let mut testers: Vec<Box<dyn PasswordTester>> = Vec::with_capacity(workers);
        for _ in 0..workers {
            match factory.create(&target) {
                Ok(tester) => testers.push(tester),
                Err(failure) => {
                    let _ = sender.send(EngineEvent::Failed(failure));
                    return;
                }
            }
        }

        let shared_source = Arc::new(Mutex::new(source));
        let found = Arc::new(AtomicBool::new(false));
        let tried = Arc::new(AtomicU64::new(0));
        let finished = Arc::new(AtomicBool::new(false));

        let mut workers_handles = Vec::with_capacity(workers);
        for mut tester in testers {
            let shared_source = Arc::clone(&shared_source);
            let found = Arc::clone(&found);
            let tried = Arc::clone(&tried);
            let cancel = cancel.clone();
            let sender = sender.clone();
            workers_handles.push(thread::spawn(move || loop {
                if found.load(Ordering::SeqCst) || cancel.is_cancelled() {
                    break;
                }
                let batch = match take_batch(&shared_source, batch_size) {
                    Some(batch) => batch,
                    None => break,
                };
                for password in batch {
                    if found.load(Ordering::SeqCst) || cancel.is_cancelled() {
                        break;
                    }
                    tried.fetch_add(1, Ordering::Relaxed);
                    if let MatchOutcome::Matched { detail } = tester.try_password(&password) {
                        if !found.swap(true, Ordering::SeqCst) {
                            let _ = sender.send(EngineEvent::Found { password, detail });
                        }
                        break;
                    }
                }
            }));
        }

        let progress_handle = {
            let tried = Arc::clone(&tried);
            let found = Arc::clone(&found);
            let finished = Arc::clone(&finished);
            let sender = sender.clone();
            thread::spawn(move || {
                let start = Instant::now();
                let mut meter = SpeedMeter::new(start);
                while !finished.load(Ordering::Relaxed) && !found.load(Ordering::SeqCst) {
                    thread::sleep(interval);
                    if finished.load(Ordering::Relaxed) || found.load(Ordering::SeqCst) {
                        break;
                    }
                    let sample = meter.sample(tried.load(Ordering::Relaxed), total, Instant::now());
                    if sender.send(EngineEvent::Progress(sample)).is_err() {
                        break;
                    }
                }
            })
        };

        for handle in workers_handles {
            let _ = handle.join();
        }
        finished.store(true, Ordering::Relaxed);
        let _ = progress_handle.join();

        if found.load(Ordering::SeqCst) {
            // Peristiwa Found sudah dikirim oleh worker yang menemukannya.
        } else if cancel.is_cancelled() {
            let _ = sender.send(EngineEvent::Cancelled);
        } else {
            let _ = sender.send(EngineEvent::Exhausted);
        }
    });

    receiver
}

/// Mengambil sekumpulan kandidat sekaligus, lalu melepas kunci sebelum mengujinya.
fn take_batch(source: &Arc<Mutex<Box<dyn CandidateSource>>>, size: usize) -> Option<Vec<String>> {
    let mut guard = source.lock().ok()?;
    let mut batch = Vec::with_capacity(size);
    for _ in 0..size {
        match guard.next_candidate() {
            Some(candidate) => batch.push(candidate),
            None => break,
        }
    }
    if batch.is_empty() {
        None
    } else {
        Some(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::bruteforce::BruteForceSource;
    use crate::model::charset::CharsetSet;
    use crate::model::target::TargetKind;
    use crate::tester::{EncryptionKind, MatchOutcome, PasswordTester, TesterFactory, TesterInfo};

    struct FakeTester {
        secret: String,
        delay: Duration,
        info: TesterInfo,
    }

    impl PasswordTester for FakeTester {
        fn try_password(&mut self, password: &str) -> MatchOutcome {
            if !self.delay.is_zero() {
                std::thread::sleep(self.delay);
            }
            if password == self.secret {
                MatchOutcome::Matched { detail: None }
            } else {
                MatchOutcome::NoMatch
            }
        }

        fn info(&self) -> &TesterInfo {
            &self.info
        }
    }

    struct FakeFactory {
        secret: String,
        delay: Duration,
        failure: Option<CrackFailure>,
    }

    impl FakeFactory {
        fn new(secret: &str) -> FakeFactory {
            FakeFactory {
                secret: secret.to_string(),
                delay: Duration::ZERO,
                failure: None,
            }
        }

        fn with_delay(secret: &str, delay: Duration) -> FakeFactory {
            FakeFactory {
                secret: secret.to_string(),
                delay,
                failure: None,
            }
        }

        fn failing(failure: CrackFailure) -> FakeFactory {
            FakeFactory {
                secret: String::new(),
                delay: Duration::ZERO,
                failure: Some(failure),
            }
        }
    }

    impl TesterFactory for FakeFactory {
        fn create(&self, _target: &CrackTarget) -> Result<Box<dyn PasswordTester>, CrackFailure> {
            if let Some(failure) = &self.failure {
                return Err(failure.clone());
            }
            Ok(Box::new(FakeTester {
                secret: self.secret.clone(),
                delay: self.delay,
                info: TesterInfo {
                    kind: TargetKind::Zip,
                    encryption: EncryptionKind::ZipCrypto,
                    entries: 1,
                },
            }))
        }
    }

    struct EmptySource;

    impl CandidateSource for EmptySource {
        fn next_candidate(&mut self) -> Option<String> {
            None
        }
        fn total(&self) -> u64 {
            0
        }
        fn position(&self) -> u64 {
            0
        }
    }

    /// Sumber sintetis dengan jumlah kandidat yang pasti. Dipakai untuk tes yang butuh
    /// pekerjaan berlangsung cukup lama agar progres sempat terkirim, tanpa bergantung
    /// pada ukuran ruang brute force.
    struct CountingSource {
        remaining: u64,
        total: u64,
        position: u64,
    }

    impl CountingSource {
        fn new(total: u64) -> CountingSource {
            CountingSource {
                remaining: total,
                total,
                position: 0,
            }
        }
    }

    impl CandidateSource for CountingSource {
        fn next_candidate(&mut self) -> Option<String> {
            if self.remaining == 0 {
                return None;
            }
            self.remaining -= 1;
            self.position += 1;
            Some(format!("candidate-{}", self.position))
        }
        fn total(&self) -> u64 {
            self.total
        }
        fn position(&self) -> u64 {
            self.position
        }
    }

    fn target() -> CrackTarget {
        CrackTarget::from_path("vault.zip")
    }

    fn config() -> EngineConfig {
        EngineConfig {
            workers: 2,
            batch_size: 8,
            progress_interval: Duration::from_millis(10),
        }
    }

    fn collect(receiver: Receiver<EngineEvent>) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        while let Ok(event) = receiver.recv_timeout(Duration::from_secs(10)) {
            let terminal = matches!(
                event,
                EngineEvent::Found { .. }
                    | EngineEvent::Exhausted
                    | EngineEvent::Cancelled
                    | EngineEvent::Failed(_)
            );
            events.push(event);
            if terminal {
                break;
            }
        }
        events
    }

    fn small_source() -> Box<dyn CandidateSource> {
        Box::new(BruteForceSource::new(CharsetSet::LOWERCASE, 1, 2).unwrap())
    }

    #[test]
    fn started_is_always_the_first_event() {
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::new("zz")),
            target(),
            config(),
            CancelToken::new(),
        ));
        assert!(matches!(events.first(), Some(EngineEvent::Started { .. })));
    }

    #[test]
    fn started_reports_the_total_from_the_source() {
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::new("zz")),
            target(),
            config(),
            CancelToken::new(),
        ));
        assert_eq!(
            events.first(),
            Some(&EngineEvent::Started { total: 26 + 676 })
        );
    }

    #[test]
    fn finds_a_password_that_is_in_the_candidate_space() {
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::new("ba")),
            target(),
            config(),
            CancelToken::new(),
        ));
        let found = events
            .iter()
            .find_map(|event| match event {
                EngineEvent::Found { password, .. } => Some(password.clone()),
                _ => None,
            })
            .expect("no password was found");
        assert_eq!(found, "ba");
    }

    #[test]
    fn reports_exhaustion_when_nothing_matches() {
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::new("not-in-the-space")),
            target(),
            config(),
            CancelToken::new(),
        ));
        assert!(matches!(events.last(), Some(EngineEvent::Exhausted)));
    }

    #[test]
    fn a_cancel_before_starting_reports_cancelled() {
        let cancel = CancelToken::new();
        cancel.cancel();
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::new("ba")),
            target(),
            config(),
            cancel,
        ));
        assert!(matches!(events.last(), Some(EngineEvent::Cancelled)));
        assert!(!events
            .iter()
            .any(|event| matches!(event, EngineEvent::Found { .. })));
    }

    #[test]
    fn an_empty_source_fails_immediately_with_no_candidate() {
        let events = collect(run(
            Box::new(EmptySource),
            Arc::new(FakeFactory::new("ba")),
            target(),
            config(),
            CancelToken::new(),
        ));
        assert_eq!(events, vec![EngineEvent::Failed(CrackFailure::NoCandidate)]);
    }

    #[test]
    fn a_failing_factory_reports_its_failure() {
        let failure = CrackFailure::BackendUnavailable {
            backend: crate::model::failure::Backend::Rar,
            reason: "built without rar support".to_string(),
        };
        let events = collect(run(
            small_source(),
            Arc::new(FakeFactory::failing(failure.clone())),
            target(),
            config(),
            CancelToken::new(),
        ));
        assert_eq!(events.last(), Some(&EngineEvent::Failed(failure)));
    }

    #[test]
    fn progress_events_arrive_while_work_is_running() {
        // 400 kandidat x 1 ms, dibagi 2 worker, berlangsung sekitar 200 ms. Interval
        // progres 10 ms, jadi beberapa peristiwa pasti sempat terkirim.
        let events = collect(run(
            Box::new(CountingSource::new(400)),
            Arc::new(FakeFactory::with_delay(
                "no-match",
                Duration::from_millis(1),
            )),
            target(),
            config(),
            CancelToken::new(),
        ));
        let progress_count = events
            .iter()
            .filter(|event| matches!(event, EngineEvent::Progress(_)))
            .count();
        assert!(progress_count >= 1, "no progress event arrived: {events:?}");
    }

    #[test]
    fn many_workers_still_exhaust_the_whole_space() {
        let mut wide = config();
        wide.workers = 4;
        let events = collect(run(
            Box::new(CountingSource::new(200)),
            Arc::new(FakeFactory::with_delay(
                "no-match",
                Duration::from_micros(200),
            )),
            target(),
            wide,
            CancelToken::new(),
        ));
        assert!(
            matches!(events.last(), Some(EngineEvent::Exhausted)),
            "the run did not finish cleanly: {events:?}"
        );
    }
}
