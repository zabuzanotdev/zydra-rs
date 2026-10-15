//! Jembatan antara antarmuka dan mesin pemecah.
//!
//! Controller memegang satu kali jalan: ia menyiapkan serangan, menjalankan mesin, lalu
//! menyerap peristiwa yang datang. Antarmuka hanya membaca keadaannya dan menggambar.
//!
//! Modul ini sengaja tidak menyentuh egui. Dengan begitu seluruh alur keadaan bisa diuji
//! tanpa membuka jendela, dan itu penting: kesalahan di sini membuat antarmuka membeku
//! atau tombol berhenti yang tidak berfungsi, dua hal yang paling sulit ditemukan lewat
//! pemeriksaan mata.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, TryRecvError};

use zydra_core::attack::{AttackLimits, AttackPlan};
use zydra_core::constants::DEFAULT_LOG_CAPACITY;
use zydra_core::engine::cancel::CancelToken;
use zydra_core::engine::crack::{run, EngineConfig, EngineEvent};
use zydra_core::log::{LogBuffer, LogEntry, LogLevel};
use zydra_core::model::failure::CrackFailure;
use zydra_core::model::progress::TryingData;
use zydra_core::model::target::CrackTarget;
use zydra_core::tester::factory::DefaultTesterFactory;
use zydra_core::tester::TesterFactory;

/// Keadaan satu kali jalan, seperti yang dibaca antarmuka.
#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    /// Belum ada apa-apa.
    Idle,
    /// Serangan berjalan.
    Running {
        /// Kemajuan terakhir.
        progress: TryingData,
    },
    /// Password ditemukan.
    Done {
        /// Password yang cocok.
        password: String,
        /// Keterangan tambahan dari backend.
        detail: Option<String>,
    },
    /// Serangan berhenti tanpa hasil.
    Failed {
        /// Sebabnya.
        failure: CrackFailure,
    },
    /// Serangan dihentikan atas permintaan pengguna.
    Cancelled,
}

impl Phase {
    /// Apakah serangan masih berjalan.
    pub fn is_running(&self) -> bool {
        matches!(self, Phase::Running { .. })
    }

    /// Apakah serangan sudah berakhir, apa pun hasilnya.
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            Phase::Done { .. } | Phase::Failed { .. } | Phase::Cancelled
        )
    }
}

/// Satu kali jalan yang sedang berlangsung.
struct Run {
    events: Receiver<EngineEvent>,
    cancel: CancelToken,
    total: u64,
    started: Instant,
}

/// Pengendali satu kali pemulihan.
pub struct Controller {
    run: Option<Run>,
    phase: Phase,
    log: LogBuffer,
    target: Option<CrackTarget>,
    plan: Option<AttackPlan>,
}

impl Default for Controller {
    fn default() -> Controller {
        Controller::new(DEFAULT_LOG_CAPACITY)
    }
}

impl Controller {
    /// Membuat pengendali dengan kapasitas catatan tertentu.
    pub fn new(log_capacity: usize) -> Controller {
        Controller {
            run: None,
            phase: Phase::Idle,
            log: LogBuffer::new(log_capacity),
            target: None,
            plan: None,
        }
    }

    /// Keadaan sekarang.
    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// Catatan kejadian.
    pub fn log(&self) -> &LogBuffer {
        &self.log
    }

    /// Target yang sedang atau terakhir dipakai.
    pub fn target(&self) -> Option<&CrackTarget> {
        self.target.as_ref()
    }

    /// Rencana yang sedang atau terakhir dipakai.
    pub fn plan(&self) -> Option<&AttackPlan> {
        self.plan.as_ref()
    }

    /// Jumlah kandidat yang akan dicoba, kalau ada yang berjalan.
    pub fn total(&self) -> Option<u64> {
        self.run.as_ref().map(|run| run.total)
    }

    /// Berapa lama serangan berjalan.
    pub fn elapsed(&self) -> Option<Duration> {
        self.run.as_ref().map(|run| run.started.elapsed())
    }

    /// Apakah serangan sedang berjalan.
    pub fn is_running(&self) -> bool {
        self.run.is_some()
    }

    /// Menyiapkan dan menjalankan sebuah serangan.
    ///
    /// Kegagalan penyiapan dikembalikan sebagai nilai, bukan disimpan sebagai keadaan,
    /// karena pemanggil perlu tahu bahwa tidak ada pekerjaan yang dimulai.
    pub fn start(
        &mut self,
        plan: AttackPlan,
        target: CrackTarget,
        limits: &AttackLimits,
        config: EngineConfig,
    ) -> Result<(), CrackFailure> {
        if self.is_running() {
            return Ok(());
        }

        let prepared = match plan.prepare(limits) {
            Ok(prepared) => prepared,
            Err(failure) => {
                self.log_failure(&failure);
                self.phase = Phase::Failed {
                    failure: failure.clone(),
                };
                return Err(failure);
            }
        };

        self.log.push(
            LogEntry::now(LogLevel::Info, "log-target-selected")
                .with_arg("name", target.display_name.clone()),
        );
        let count_key = match plan {
            AttackPlan::Dictionary { .. } => "log-wordlist-loaded",
            AttackPlan::BruteForce { .. } => "log-brute-force-space",
        };
        self.log.push(
            LogEntry::now(LogLevel::Info, count_key).with_arg("count", prepared.total.to_string()),
        );

        let cancel = CancelToken::new();
        let factory: Arc<dyn TesterFactory> = Arc::new(DefaultTesterFactory::new());

        // Pemeriksaan awal: target dibuka sekali sebelum pekerjaan dimulai. Tanpa ini,
        // berkas yang hilang atau backend yang tidak tersedia baru ketahuan setelah
        // antarmuka sempat menampilkan progres, dan pengguna menunggu untuk sesuatu yang
        // memang tidak akan berjalan.
        if let Err(failure) = factory.create(&target) {
            self.log_failure(&failure);
            self.phase = Phase::Failed {
                failure: failure.clone(),
            };
            return Err(failure);
        }

        let events = run(
            prepared.source,
            factory,
            target.clone(),
            config,
            cancel.clone(),
        );

        self.phase = Phase::Running {
            progress: TryingData {
                tried: 0,
                total: prepared.total,
                speed_per_second: 0,
                elapsed: Duration::ZERO,
                eta: None,
            },
        };
        self.run = Some(Run {
            events,
            cancel,
            total: prepared.total,
            started: Instant::now(),
        });
        self.target = Some(target);
        self.plan = Some(plan);
        Ok(())
    }

    /// Menyerap semua peristiwa yang sudah tersedia.
    ///
    /// Dipanggil sekali per bingkai. Tidak memblokir: kalau belum ada peristiwa, ia
    /// kembali dengan segera, sehingga antarmuka tetap menggambar.
    pub fn poll(&mut self) {
        let Some(run) = self.run.as_ref() else {
            return;
        };
        let events = run.events.clone();

        loop {
            match events.try_recv() {
                Ok(event) => {
                    if let Some(terminal) = self.apply(event) {
                        self.finish(terminal);
                        return;
                    }
                }
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    // Kanal tertutup tanpa peristiwa penutup. Ini berarti worker berhenti
                    // tanpa lapor, dan itu harus terlihat, bukan didiamkan.
                    self.log.record(LogLevel::Error, "error-unknown");
                    self.finish(Phase::Failed {
                        failure: CrackFailure::NoCandidate,
                    });
                    return;
                }
            }
        }
    }

    /// Meminta serangan berhenti. Aman dipanggil berkali-kali.
    pub fn stop(&mut self) {
        if let Some(run) = self.run.as_ref() {
            run.cancel.cancel();
        }
    }

    /// Membersihkan keadaan setelah serangan selesai.
    pub fn reset(&mut self) {
        if self.is_running() {
            self.stop();
        }
        self.run = None;
        self.phase = Phase::Idle;
        self.log.clear();
    }

    /// Menerapkan satu peristiwa. Mengembalikan keadaan akhir kalau peristiwa itu menutup
    /// penelusuran.
    fn apply(&mut self, event: EngineEvent) -> Option<Phase> {
        match event {
            EngineEvent::Started { .. } => None,
            EngineEvent::Progress(data) => {
                self.phase = Phase::Running { progress: data };
                None
            }
            EngineEvent::Found { password, detail } => {
                self.log.record(LogLevel::Success, "result-granted");
                Some(Phase::Done { password, detail })
            }
            EngineEvent::Exhausted => {
                self.log.record(LogLevel::Warn, "result-denied");
                Some(Phase::Failed {
                    failure: CrackFailure::PasswordNotFound,
                })
            }
            EngineEvent::Cancelled => {
                self.log.record(LogLevel::Warn, "result-denied");
                Some(Phase::Cancelled)
            }
            EngineEvent::Failed(failure) => {
                self.log_failure(&failure);
                Some(Phase::Failed { failure })
            }
        }
    }

    /// Mencatat kegagalan beserta penanda teknisnya.
    ///
    /// Penanda itu masuk ke argumen log, bukan ke kalimat yang dilihat pengguna: lapisan
    /// terjemahan hanya menampilkan kalimat umum untuk `error-unknown`.
    fn log_failure(&mut self, failure: &CrackFailure) {
        self.log.push(
            LogEntry::now(LogLevel::Error, "error-unknown")
                .with_arg("detail", failure_code(failure).to_string()),
        );
    }

    /// Menutup satu kali jalan.
    fn finish(&mut self, phase: Phase) {
        self.run = None;
        self.phase = phase;
    }
}

/// Penanda teknis kegagalan, untuk catatan.
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
        CrackFailure::BackendUnavailable { .. } => "backend-unavailable",
        CrackFailure::SessionFingerprintMismatch => "session-mismatch",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use zip::write::SimpleFileOptions;
    use zip::AesMode;
    use zydra_core::model::charset::CharsetSet;

    const SECRET: &str = "kr4k3n_1987";

    fn encrypted_archive(directory: &std::path::Path, password: &str) -> PathBuf {
        let path = directory.join("vault.zip");
        let file = std::fs::File::create(&path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default().with_aes_encryption(AesMode::Aes256, password);
        writer.start_file("secret.txt", options).unwrap();
        writer
            .write_all(b"isi rahasia yang panjangnya melewati satu buffer")
            .unwrap();
        writer.finish().unwrap();
        path
    }

    fn wordlist(directory: &std::path::Path, lines: &[&str]) -> PathBuf {
        let path = directory.join("words.txt");
        let mut file = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(file, "{line}").unwrap();
        }
        path
    }

    fn fast_config() -> EngineConfig {
        EngineConfig {
            workers: 2,
            batch_size: 4,
            progress_interval: Duration::from_millis(5),
        }
    }

    /// Menjalankan controller sampai selesai, dengan batas waktu supaya tes tidak bisa
    /// menggantung selamanya.
    fn run_to_completion(controller: &mut Controller) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while controller.is_running() {
            controller.poll();
            if Instant::now() > deadline {
                panic!("the controller never finished");
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn a_fresh_controller_is_idle_and_empty() {
        let controller = Controller::default();
        assert_eq!(controller.phase(), &Phase::Idle);
        assert!(!controller.is_running());
        assert!(controller.log().is_empty());
        assert_eq!(controller.total(), None);
    }

    #[test]
    fn a_wordlist_finds_the_password_and_ends_in_done() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), SECRET);
        let words = wordlist(directory.path(), &["password", "letmein", SECRET]);

        let mut controller = Controller::default();
        controller
            .start(
                AttackPlan::Dictionary { wordlist: words },
                CrackTarget::from_path(&archive),
                &AttackLimits::default(),
                fast_config(),
            )
            .unwrap();
        assert!(controller.is_running());
        run_to_completion(&mut controller);

        match controller.phase() {
            Phase::Done { password, .. } => assert_eq!(password, SECRET),
            other => panic!("unexpected phase: {other:?}"),
        }
        assert!(!controller.is_running());
        assert!(controller.log().len() >= 3, "expected a few log entries");
    }

    #[test]
    fn a_wordlist_without_the_password_ends_in_failure() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), SECRET);
        let words = wordlist(directory.path(), &["password", "letmein"]);

        let mut controller = Controller::default();
        controller
            .start(
                AttackPlan::Dictionary { wordlist: words },
                CrackTarget::from_path(&archive),
                &AttackLimits::default(),
                fast_config(),
            )
            .unwrap();
        run_to_completion(&mut controller);

        assert_eq!(
            controller.phase(),
            &Phase::Failed {
                failure: CrackFailure::PasswordNotFound
            }
        );
    }

    #[test]
    fn a_brute_force_search_ends_in_done() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), "42");

        let mut controller = Controller::default();
        controller
            .start(
                AttackPlan::BruteForce {
                    charset: CharsetSet::DIGITS,
                    min_length: 1,
                    max_length: 2,
                },
                CrackTarget::from_path(&archive),
                &AttackLimits::default(),
                fast_config(),
            )
            .unwrap();
        run_to_completion(&mut controller);

        match controller.phase() {
            Phase::Done { password, .. } => assert_eq!(password, "42"),
            other => panic!("unexpected phase: {other:?}"),
        }
    }

    #[test]
    fn stopping_mid_run_ends_in_cancelled() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), SECRET);
        let words = wordlist(directory.path(), &["password", "letmein"]);

        let mut controller = Controller::default();
        controller
            .start(
                AttackPlan::Dictionary { wordlist: words },
                CrackTarget::from_path(&archive),
                &AttackLimits::default(),
                fast_config(),
            )
            .unwrap();
        controller.stop();
        run_to_completion(&mut controller);

        assert_eq!(controller.phase(), &Phase::Cancelled);
    }

    #[test]
    fn a_target_that_cannot_be_opened_fails_before_any_work_starts() {
        let directory = tempfile::tempdir().unwrap();
        let words = wordlist(directory.path(), &["a"]);
        let missing = directory.path().join("gone.zip");

        let mut controller = Controller::default();
        let result = controller.start(
            AttackPlan::Dictionary { wordlist: words },
            CrackTarget::from_path(&missing),
            &AttackLimits::default(),
            fast_config(),
        );

        assert!(matches!(result, Err(CrackFailure::FileUnreadable { .. })));
        assert!(!controller.is_running());
        assert!(matches!(controller.phase(), Phase::Failed { .. }));
    }

    #[test]
    fn an_empty_wordlist_is_rejected_before_any_work_starts() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), SECRET);
        let words = wordlist(directory.path(), &["", "", ""]);

        let mut controller = Controller::default();
        let result = controller.start(
            AttackPlan::Dictionary { wordlist: words },
            CrackTarget::from_path(&archive),
            &AttackLimits::default(),
            fast_config(),
        );

        assert_eq!(result.unwrap_err(), CrackFailure::EmptyWordlist);
        assert!(!controller.is_running());
    }

    #[test]
    fn starting_twice_while_running_changes_nothing() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), SECRET);
        let words = wordlist(directory.path(), &["password"]);

        let mut controller = Controller::default();
        let plan = AttackPlan::Dictionary {
            wordlist: words.clone(),
        };
        controller
            .start(
                plan.clone(),
                CrackTarget::from_path(&archive),
                &AttackLimits::default(),
                fast_config(),
            )
            .unwrap();
        let before = controller.total();
        controller
            .start(
                plan,
                CrackTarget::from_path(&archive),
                &AttackLimits::default(),
                fast_config(),
            )
            .unwrap();
        assert_eq!(controller.total(), before);

        controller.stop();
        run_to_completion(&mut controller);
    }

    #[test]
    fn polling_without_a_run_does_nothing() {
        let mut controller = Controller::default();
        controller.poll();
        assert_eq!(controller.phase(), &Phase::Idle);
    }

    #[test]
    fn reset_returns_the_controller_to_idle() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), SECRET);
        let words = wordlist(directory.path(), &[SECRET]);

        let mut controller = Controller::default();
        controller
            .start(
                AttackPlan::Dictionary { wordlist: words },
                CrackTarget::from_path(&archive),
                &AttackLimits::default(),
                fast_config(),
            )
            .unwrap();
        run_to_completion(&mut controller);
        controller.reset();

        assert_eq!(controller.phase(), &Phase::Idle);
        assert!(controller.log().is_empty());
        assert!(!controller.is_running());
    }

    #[test]
    fn every_failure_has_a_log_code() {
        let failures = [
            CrackFailure::PasswordNotFound,
            CrackFailure::EmptyWordlist,
            CrackFailure::NoCandidate,
            CrackFailure::SessionFingerprintMismatch,
        ];
        for failure in &failures {
            assert!(!failure_code(failure).is_empty());
        }
    }
}
