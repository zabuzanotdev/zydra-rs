//! Melanjutkan serangan yang terputus.

pub mod store;

use crate::attack::AttackPlan;
use crate::error::CoreError;
use crate::model::target::CrackTarget;
use crate::session::store::{
    FileSessionStore, SessionLookup, SessionState, SessionStore, TargetFingerprint,
};

/// Variabel lingkungan yang menimpa direktori penyimpanan sesi.
pub const SESSION_DIR_VARIABLE: &str = "ZYDRA_SESSION_DIR";

/// Menyimpan dan memuat sesi untuk satu berkas target.
///
/// Pembungkus ini ada supaya antarmuka tidak masing-masing menulis ulang logika yang sama:
/// memilih direktori, membaca sidik jari, dan mengabaikan kegagalan penyimpanan. Kalau CLI
/// dan GUI menyimpannya sendiri-sendiri, cepat atau lambat keduanya akan berbeda pendapat
/// soal apa yang dianggap sesi yang sah.
#[derive(Debug, Clone)]
pub struct SessionTracker {
    store: Option<FileSessionStore>,
    target: Option<CrackTarget>,
}

impl SessionTracker {
    /// Membuat pelacak yang memakai direktori data milik pengguna.
    ///
    /// Variabel lingkungan `ZYDRA_SESSION_DIR` menimpanya. Itu berguna untuk pemasangan
    /// portabel, untuk memisahkan pekerjaan satu sama lain, dan untuk pengujian yang tidak
    /// mungkin menulis ke direktori data pengguna sungguhan. Pemeriksaannya ada di sini,
    /// bukan di tiap antarmuka, supaya CLI dan GUI tidak bisa berbeda perilaku.
    ///
    /// Kalau direktori itu tidak bisa ditentukan, pelacaknya mati dan serangan tetap
    /// berjalan. Kehilangan kemampuan melanjutkan tidak sebanding dengan kehilangan
    /// kemampuan memulihkan password sama sekali.
    pub fn for_target(target: &CrackTarget) -> SessionTracker {
        if let Ok(directory) = std::env::var(SESSION_DIR_VARIABLE) {
            if !directory.trim().is_empty() {
                return SessionTracker::with_store(FileSessionStore::new(directory), target);
            }
        }
        match FileSessionStore::in_user_data_dir() {
            Ok(store) => SessionTracker::with_store(store, target),
            Err(_) => SessionTracker::disabled(),
        }
    }

    /// Membuat pelacak dengan direktori tertentu.
    pub fn with_store(store: FileSessionStore, target: &CrackTarget) -> SessionTracker {
        SessionTracker {
            store: Some(store),
            target: Some(target.clone()),
        }
    }

    /// Pelacak yang tidak menyimpan apa pun.
    pub fn disabled() -> SessionTracker {
        SessionTracker {
            store: None,
            target: None,
        }
    }

    /// Apakah pelacak ini benar-benar menyimpan sesuatu.
    pub fn is_enabled(&self) -> bool {
        self.store.is_some() && self.target.is_some()
    }

    /// Sidik jari berkas target, dibaca dari sistem berkas setiap kali dipanggil.
    ///
    /// Sengaja tidak disimpan saat pelacak dibuat. Pelacak bisa hidup lebih lama daripada
    /// berkasnya, dan sidik jari yang basi akan membuat berkas yang sudah berubah tetap
    /// dianggap cocok - justru keadaan yang paling berbahaya, karena serangan akan
    /// dilanjutkan dari posisi yang tidak lagi berarti.
    fn fingerprint(&self) -> Option<TargetFingerprint> {
        TargetFingerprint::of(self.target.as_ref()?).ok()
    }

    /// Mencari sesi tersimpan. `None` berarti pelacaknya mati.
    pub fn lookup(&self) -> Option<Result<SessionLookup, CoreError>> {
        let store = self.store.as_ref()?;
        let fingerprint = self.fingerprint()?;
        Some(store.load(&fingerprint))
    }

    /// Menyimpan posisi.
    ///
    /// Kegagalan menyimpan tidak pernah dilaporkan sebagai kesalahan ke pemanggil: sesi
    /// hanya alat bantu, dan serangan yang sedang berjalan lebih penting daripada catatan
    /// posisinya.
    pub fn save(&self, plan: &AttackPlan, position: u64) {
        let (Some(store), Some(fingerprint)) = (self.store.as_ref(), self.fingerprint()) else {
            return;
        };
        let state = SessionState::new(fingerprint, plan.clone(), position);
        let _ = store.save(&state);
    }

    /// Menghapus sesi, dipakai setelah serangan selesai sepenuhnya.
    pub fn clear(&self) {
        let (Some(store), Some(fingerprint)) = (self.store.as_ref(), self.fingerprint()) else {
            return;
        };
        let _ = store.clear(&fingerprint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;

    fn target(directory: &Path) -> CrackTarget {
        let path = directory.join("v.zip");
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(b"contents").unwrap();
        file.flush().unwrap();
        CrackTarget::from_path(&path)
    }

    fn plan() -> AttackPlan {
        AttackPlan::Mask {
            pattern: "?d?d".to_string(),
        }
    }

    #[test]
    fn a_disabled_tracker_stores_nothing_and_reports_nothing() {
        let tracker = SessionTracker::disabled();
        assert!(!tracker.is_enabled());
        assert!(tracker.lookup().is_none());
        // Keduanya tidak boleh panik.
        tracker.save(&plan(), 10);
        tracker.clear();
    }

    #[test]
    fn a_tracker_round_trips_a_position() {
        let directory = tempfile::tempdir().unwrap();
        let target = target(directory.path());
        let tracker = SessionTracker::with_store(
            FileSessionStore::new(directory.path().join("sessions")),
            &target,
        );

        assert!(tracker.is_enabled());
        assert_eq!(tracker.lookup().unwrap().unwrap(), SessionLookup::Absent);

        tracker.save(&plan(), 42);
        match tracker.lookup().unwrap().unwrap() {
            SessionLookup::Found(state) => {
                assert_eq!(state.position, 42);
                assert_eq!(state.plan, plan());
            }
            other => panic!("unexpected lookup: {other:?}"),
        }

        tracker.clear();
        assert_eq!(tracker.lookup().unwrap().unwrap(), SessionLookup::Absent);
    }

    #[test]
    fn saving_repeatedly_keeps_the_latest_position() {
        let directory = tempfile::tempdir().unwrap();
        let target = target(directory.path());
        let tracker = SessionTracker::with_store(
            FileSessionStore::new(directory.path().join("sessions")),
            &target,
        );

        tracker.save(&plan(), 1);
        tracker.save(&plan(), 2);
        tracker.save(&plan(), 3);

        match tracker.lookup().unwrap().unwrap() {
            SessionLookup::Found(state) => assert_eq!(state.position, 3),
            other => panic!("unexpected lookup: {other:?}"),
        }
    }

    #[test]
    fn a_tracker_notices_that_the_target_changed_after_it_was_created() {
        // Sidik jari dibaca ulang setiap kali, bukan disimpan saat pelacak dibuat. Kalau
        // disimpan, pelacak yang hidup lebih lama daripada berkasnya akan menganggap berkas
        // yang sudah berubah masih cocok, dan serangan dilanjutkan dari posisi yang tidak
        // lagi berarti.
        let directory = tempfile::tempdir().unwrap();
        let target = target(directory.path());
        let tracker = SessionTracker::with_store(
            FileSessionStore::new(directory.path().join("sessions")),
            &target,
        );
        tracker.save(&plan(), 5);

        std::fs::write(&target.path, b"different contents now").unwrap();

        assert_eq!(tracker.lookup().unwrap().unwrap(), SessionLookup::Mismatch);
    }

    #[test]
    fn clearing_after_the_target_changed_still_removes_the_old_session() {
        // Berkas sesi dinamai dari lokasi target, bukan dari sidik jarinya, sehingga
        // pembersihan tetap menemukannya walaupun berkasnya sudah berubah.
        let directory = tempfile::tempdir().unwrap();
        let target = target(directory.path());
        let tracker = SessionTracker::with_store(
            FileSessionStore::new(directory.path().join("sessions")),
            &target,
        );
        tracker.save(&plan(), 5);

        std::fs::write(&target.path, b"different contents now").unwrap();
        tracker.clear();

        assert_eq!(tracker.lookup().unwrap().unwrap(), SessionLookup::Absent);
    }
}
