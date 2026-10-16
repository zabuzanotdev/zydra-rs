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
use zydra_core::constants::{DEFAULT_LOG_CAPACITY, SESSION_SAVE_INTERVAL_SECS};
use zydra_core::engine::cancel::CancelToken;
use zydra_core::engine::crack::{run, EngineConfig, EngineEvent};
use zydra_core::log::{LogBuffer, LogEntry, LogLevel};
use zydra_core::model::failure::CrackFailure;
use zydra_core::model::progress::TryingData;
use zydra_core::model::target::CrackTarget;
use zydra_core::session::store::SessionLookup;
use zydra_core::session::SessionTracker;
use zydra_core::tester::factory::DefaultTesterFactory;
use zydra_core::tester::{EncryptionKind, TesterFactory};

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
    /// Jumlah kandidat yang sudah dilewati sebelum run ini dimulai.
    start_position: u64,
    /// Waktu penyimpanan posisi terakhir.
    last_save: Instant,
}

/// Pengendali satu kali pemulihan.
pub struct Controller {
    run: Option<Run>,
    phase: Phase,
    log: LogBuffer,
    target: Option<CrackTarget>,
    plan: Option<AttackPlan>,
    sessions: SessionTracker,
    /// Pelacak sesi yang dipilih pemanggil, mengalahkan direktori data pengguna.
    session_override: Option<SessionTracker>,
    /// Batas byte entri ZIP yang dibaca saat memverifikasi kandidat.
    zip_max_entry_bytes: Option<u64>,
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
            sessions: SessionTracker::disabled(),
            session_override: None,
            zip_max_entry_bytes: None,
        }
    }

    /// Menentukan batas byte entri ZIP yang dibaca saat memverifikasi kandidat.
    pub fn use_zip_max_entry_bytes(&mut self, bytes: u64) {
        self.zip_max_entry_bytes = Some(bytes);
    }

    /// Memakai pelacak sesi tertentu alih-alih direktori data pengguna.
    ///
    /// Dipakai pengujian, yang tidak boleh menulis ke direktori data pengguna sungguhan,
    /// dan oleh antarmuka yang ingin menentukan sendiri lokasi penyimpanannya.
    pub fn use_session_tracker(&mut self, tracker: SessionTracker) {
        self.session_override = Some(tracker);
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
        resume: bool,
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

        // Posisi tersimpan diperiksa sebelum apa pun dijalankan. Berkas yang sudah berubah
        // ditolak, karena melanjutkan dari posisi lama akan melewati kandidat yang
        // seharusnya dicoba, dan memulai diam-diam dari awal akan membuat pengguna mengira
        // serangannya dilanjutkan.
        let mut source = prepared.source;
        let mut start_position = 0;
        self.sessions = match self.session_override.as_ref() {
            Some(tracker) => tracker.clone(),
            None => SessionTracker::for_target(&target),
        };

        if resume {
            match self.sessions.lookup() {
                Some(Ok(SessionLookup::Found(state))) => {
                    start_position = state.position.min(prepared.total);
                    source.skip(start_position);
                    self.log.push(
                        LogEntry::now(LogLevel::Info, "log-session-resumed")
                            .with_arg("position", start_position.to_string())
                            .with_arg("total", prepared.total.to_string()),
                    );
                }
                Some(Ok(SessionLookup::Absent)) => {
                    self.log.record(LogLevel::Info, "log-session-absent");
                }
                Some(Ok(SessionLookup::Mismatch)) => {
                    let failure = CrackFailure::SessionFingerprintMismatch;
                    self.log_failure(&failure);
                    self.phase = Phase::Failed {
                        failure: failure.clone(),
                    };
                    return Err(failure);
                }
                Some(Err(error)) => {
                    self.log.push(
                        LogEntry::now(LogLevel::Warn, "error-unknown")
                            .with_arg("detail", error.to_string()),
                    );
                }
                None => {}
            }
        }

        self.log.push(
            LogEntry::now(LogLevel::Info, "log-target-selected")
                .with_arg("name", target.display_name.clone()),
        );
        let count_key = count_log_key(&plan);
        self.log.push(
            LogEntry::now(LogLevel::Info, count_key).with_arg("count", prepared.total.to_string()),
        );

        let cancel = CancelToken::new();
        let factory: Arc<dyn TesterFactory> = match self.zip_max_entry_bytes {
            Some(bytes) => Arc::new(DefaultTesterFactory::with_zip_max_entry_bytes(bytes)),
            None => Arc::new(DefaultTesterFactory::new()),
        };

        // Pemeriksaan awal: target dibuka sekali sebelum pekerjaan dimulai. Tanpa ini,
        // berkas yang hilang atau backend yang tidak tersedia baru ketahuan setelah
        // antarmuka sempat menampilkan progres, dan pengguna menunggu untuk sesuatu yang
        // memang tidak akan berjalan.
        // Berkas tanpa enkripsi akan berhasil dibuka dengan password apa pun, sehingga
        // kandidat pertama dilaporkan sebagai password yang ditemukan. Itu jawaban yang
        // salah untuk pertanyaan yang tidak pernah ditanyakan.
        //
        // Pemeriksaan ini juga menangkap berkas yang hilang dan backend yang bermasalah
        // sebelum antarmuka sempat menampilkan progres.
        match factory.create(&target) {
            Ok(tester) if tester.info().encryption == EncryptionKind::None => {
                let failure = CrackFailure::TargetNotEncrypted;
                self.log_failure(&failure);
                self.phase = Phase::Failed {
                    failure: failure.clone(),
                };
                return Err(failure);
            }
            Ok(_) => {}
            Err(failure) => {
                self.log_failure(&failure);
                self.phase = Phase::Failed {
                    failure: failure.clone(),
                };
                return Err(failure);
            }
        }

        let events = run(source, factory, target.clone(), config, cancel.clone());

        self.phase = Phase::Running {
            progress: TryingData {
                tried: start_position,
                total: prepared.total,
                speed_per_second: 0,
                elapsed: Duration::ZERO,
                eta: None,
            },
        };
        let now = Instant::now();
        self.run = Some(Run {
            events,
            cancel,
            total: prepared.total,
            started: now,
            start_position,
            last_save: now,
        });
        // Posisi awal disimpan juga, supaya serangan yang dihentikan sangat awal tetap
        // punya titik lanjut.
        self.sessions.save(&plan, start_position);
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

    /// Menambahkan satu catatan dari luar pengendali.
    ///
    /// Dipakai antarmuka untuk mencatat hal yang bukan bagian dari serangan, misalnya hasil
    /// mengekspor catatan. Menaruhnya di catatan yang sama membuat pengguna menemukannya di
    /// tempat yang memang ia lihat, bukan di pemberitahuan terpisah yang bisa terlewat.
    pub fn note(&mut self, level: LogLevel, key: &str, args: &[(&str, &str)]) {
        let mut entry = LogEntry::now(level, key);
        for (name, value) in args {
            entry = entry.with_arg(*name, *value);
        }
        self.log.push(entry);
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
                self.save_progress(data.tried);
                None
            }
            EngineEvent::Found { password, detail } => {
                self.log.record(LogLevel::Success, "result-granted");
                // Serangan selesai sepenuhnya, jadi tidak ada yang perlu dilanjutkan.
                self.sessions.clear();
                Some(Phase::Done { password, detail })
            }
            EngineEvent::Exhausted => {
                self.log.record(LogLevel::Warn, "result-denied");
                self.sessions.clear();
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

    /// Menyimpan posisi secara berkala selama serangan berjalan.
    ///
    /// Mesin menghitung kandidat yang dicoba sejak ia dijalankan, bukan sejak awal ruang
    /// pencarian, jadi titik awalnya harus ditambahkan. Tanpa itu posisi yang tersimpan
    /// akan mundur setiap kali serangan dilanjutkan lalu dihentikan lagi.
    fn save_progress(&mut self, tried: u64) {
        let Some(run) = self.run.as_ref() else {
            return;
        };
        if run.last_save.elapsed() < Duration::from_secs(SESSION_SAVE_INTERVAL_SECS) {
            return;
        }
        let position = run.start_position.saturating_add(tried).min(run.total);

        if let Some(plan) = self.plan.as_ref() {
            self.sessions.save(plan, position);
        }
        if let Some(run) = self.run.as_mut() {
            run.last_save = Instant::now();
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

/// Apakah ada sesi tersimpan yang bisa dilanjutkan untuk sebuah target.
///
/// Dipanggil saat target dipilih, bukan setiap bingkai: pemeriksaannya membaca sistem
/// berkas, dan melakukannya terus-menerus akan membuat antarmuka tersendat tanpa manfaat.
pub fn session_available(target: &CrackTarget) -> bool {
    matches!(
        SessionTracker::for_target(target).lookup(),
        Some(Ok(SessionLookup::Found(_)))
    )
}

/// Kunci log untuk jumlah kandidat sebuah rencana.
///
/// Wordlist memuat sejumlah kandidat yang sudah ada; brute force dan mask menelusuri ruang
/// kombinasi. Rencana dengan aturan mengikuti kata dasarnya, karena aturan hanya
/// melipatgandakan kandidat, bukan mengubah dari mana kandidat itu berasal.
fn count_log_key(plan: &AttackPlan) -> &'static str {
    match plan {
        AttackPlan::Dictionary { .. } => "log-wordlist-loaded",
        AttackPlan::BruteForce { .. } | AttackPlan::Mask { .. } => "log-brute-force-space",
        AttackPlan::WithRules { base, .. } => count_log_key(base),
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
        CrackFailure::TargetNotEncrypted => "target-not-encrypted",
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
                false,
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
                false,
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
                false,
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
                false,
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
            false,
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
            false,
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
                false,
            )
            .unwrap();
        let before = controller.total();
        controller
            .start(
                plan,
                CrackTarget::from_path(&archive),
                &AttackLimits::default(),
                fast_config(),
                false,
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
                false,
            )
            .unwrap();
        run_to_completion(&mut controller);
        controller.reset();

        assert_eq!(controller.phase(), &Phase::Idle);
        assert!(controller.log().is_empty());
        assert!(!controller.is_running());
    }

    #[test]
    fn an_unencrypted_target_is_refused() {
        // Arsip tanpa enkripsi terbuka dengan password apa pun, sehingga tanpa pemeriksaan
        // ini kandidat pertama akan dilaporkan sebagai password yang ditemukan.
        let directory = tempfile::tempdir().unwrap();
        let archive = directory.path().join("plain.zip");
        let file = std::fs::File::create(&archive).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        writer
            .start_file("plain.txt", zip::write::SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut writer, b"no password here").unwrap();
        writer.finish().unwrap();

        let words = wordlist(directory.path(), &["password"]);
        let mut controller = Controller::default();
        let result = controller.start(
            wordlist_plan(&words),
            CrackTarget::from_path(&archive),
            &AttackLimits::default(),
            fast_config(),
            false,
        );

        assert_eq!(result.unwrap_err(), CrackFailure::TargetNotEncrypted);
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

    /// Membuat pelacak sesi yang menulis ke direktori sementara milik tes.
    ///
    /// Pengujian tidak boleh menulis ke direktori data pengguna sungguhan, dan mengubah
    /// variabel lingkungan tidak bisa dipakai di sini karena tes berjalan bersamaan di
    /// dalam satu proses.
    fn tracker_in(directory: &std::path::Path, target: &CrackTarget) -> SessionTracker {
        SessionTracker::with_store(
            zydra_core::session::store::FileSessionStore::new(directory.join("sessions")),
            target,
        )
    }

    fn wordlist_plan(words: &std::path::Path) -> AttackPlan {
        AttackPlan::Dictionary {
            wordlist: words.to_path_buf(),
        }
    }

    #[test]
    fn a_resumed_run_starts_from_the_saved_position() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), SECRET);
        let words = wordlist(directory.path(), &["password", "letmein", SECRET]);
        let target = CrackTarget::from_path(&archive);

        let tracker = tracker_in(directory.path(), &target);
        tracker.save(&wordlist_plan(&words), 2);

        let mut controller = Controller::default();
        controller.use_session_tracker(tracker);
        controller
            .start(
                wordlist_plan(&words),
                target,
                &AttackLimits::default(),
                fast_config(),
                true,
            )
            .unwrap();

        // Kemajuan pertama harus dimulai dari posisi tersimpan, bukan dari nol.
        match controller.phase() {
            Phase::Running { progress } => assert_eq!(progress.tried, 2),
            other => panic!("unexpected phase: {other:?}"),
        }
        assert!(
            controller
                .log()
                .entries()
                .any(|entry| entry.key == "log-session-resumed"),
            "the resume was not logged"
        );

        run_to_completion(&mut controller);
        match controller.phase() {
            Phase::Done { password, .. } => assert_eq!(password, SECRET),
            other => panic!("unexpected phase: {other:?}"),
        }
    }

    #[test]
    fn resuming_a_changed_file_is_refused() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), SECRET);
        let words = wordlist(directory.path(), &["password"]);
        let target = CrackTarget::from_path(&archive);

        let tracker = tracker_in(directory.path(), &target);
        tracker.save(&wordlist_plan(&words), 1);

        // Berkas targetnya berubah setelah sesi disimpan.
        std::fs::write(&archive, b"the file is no longer the same archive").unwrap();

        let mut controller = Controller::default();
        controller.use_session_tracker(tracker);
        let result = controller.start(
            wordlist_plan(&words),
            CrackTarget::from_path(&archive),
            &AttackLimits::default(),
            fast_config(),
            true,
        );

        assert_eq!(
            result.unwrap_err(),
            CrackFailure::SessionFingerprintMismatch
        );
        assert!(!controller.is_running());
    }

    #[test]
    fn resuming_without_a_saved_session_starts_from_the_beginning() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), "42");
        let target = CrackTarget::from_path(&archive);

        let mut controller = Controller::default();
        controller.use_session_tracker(tracker_in(directory.path(), &target));
        controller
            .start(
                AttackPlan::BruteForce {
                    charset: CharsetSet::DIGITS,
                    min_length: 1,
                    max_length: 2,
                },
                target,
                &AttackLimits::default(),
                fast_config(),
                true,
            )
            .unwrap();

        match controller.phase() {
            Phase::Running { progress } => assert_eq!(progress.tried, 0),
            other => panic!("unexpected phase: {other:?}"),
        }
        assert!(
            controller
                .log()
                .entries()
                .any(|entry| entry.key == "log-session-absent"),
            "the missing session was not logged"
        );

        run_to_completion(&mut controller);
    }

    #[test]
    fn a_finished_run_leaves_nothing_to_resume() {
        let directory = tempfile::tempdir().unwrap();
        let archive = encrypted_archive(directory.path(), "42");
        let target = CrackTarget::from_path(&archive);
        let tracker = tracker_in(directory.path(), &target);

        let mut controller = Controller::default();
        controller.use_session_tracker(tracker.clone());
        controller
            .start(
                AttackPlan::BruteForce {
                    charset: CharsetSet::DIGITS,
                    min_length: 1,
                    max_length: 2,
                },
                target,
                &AttackLimits::default(),
                fast_config(),
                false,
            )
            .unwrap();
        run_to_completion(&mut controller);

        assert_eq!(tracker.lookup().unwrap().unwrap(), SessionLookup::Absent);
    }
}
