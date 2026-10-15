//! Menyimpan dan memuat kembali posisi sebuah serangan.
//!
//! Serangan brute force yang panjang bisa memakan berjam-jam. Tanpa penyimpanan posisi,
//! menutup aplikasi berarti memulai dari nol, dan itu membuat serangan panjang praktis
//! tidak mungkin dilakukan.
//!
//! # Sidik jari berkas target
//!
//! Posisi yang tersimpan hanya berarti untuk berkas yang persis sama. Kalau berkasnya
//! sudah berubah, melanjutkan dari posisi lama akan melewati kandidat yang seharusnya
//! dicoba. Karena itu setiap sesi menyimpan sidik jari berkasnya: lokasi, ukuran, waktu
//! perubahan, dan jenisnya.
//!
//! # Berkas sesi dinamai dari lokasi target, bukan dari sidik jarinya
//!
//! Kalau namanya diturunkan dari sidik jari, mengubah berkas target akan membuat berkas
//! sesinya tidak ditemukan sama sekali, dan pengguna hanya akan diberi tahu bahwa tidak ada
//! sesi. Dengan nama diturunkan dari lokasi, berkas sesinya tetap ditemukan lalu
//! dibandingkan, sehingga pesannya bisa tepat: berkasnya sudah berubah.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::attack::AttackPlan;
use crate::error::CoreError;
use crate::log::unix_now;
use crate::model::target::{CrackTarget, TargetKind};

/// Versi format berkas sesi.
///
/// Naikkan saat bentuk berkasnya berubah. Sesi dari versi lain ditolak, bukan ditafsirkan
/// dengan aturan yang salah.
pub const SESSION_VERSION: u32 = 1;

/// Ciri-ciri berkas yang membuat sebuah sesi hanya berlaku untuk berkas itu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetFingerprint {
    /// Lokasi berkas.
    pub path: PathBuf,
    /// Ukuran berkas, dalam byte.
    pub size: u64,
    /// Waktu perubahan terakhir, dalam detik sejak epoch Unix.
    pub modified_unix: i64,
    /// Jenis berkas.
    pub kind: TargetKind,
}

impl TargetFingerprint {
    /// Membaca ciri-ciri sebuah target dari sistem berkas.
    pub fn of(target: &CrackTarget) -> Result<TargetFingerprint, CoreError> {
        let metadata = std::fs::metadata(&target.path).map_err(|error| CoreError::Session {
            detail: format!("{}: {error}", target.path.display()),
        })?;

        let modified_unix = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or(0);

        Ok(TargetFingerprint {
            path: target.path.clone(),
            size: metadata.len(),
            modified_unix,
            kind: target.kind,
        })
    }
}

/// Posisi sebuah serangan yang bisa dilanjutkan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionState {
    /// Versi format.
    pub version: u32,
    /// Berkas yang menjadi sasaran.
    pub fingerprint: TargetFingerprint,
    /// Rencana serangan yang sedang dijalankan.
    pub plan: AttackPlan,
    /// Jumlah kandidat yang sudah dicoba.
    pub position: u64,
    /// Waktu penyimpanan terakhir.
    pub updated_at: i64,
}

impl SessionState {
    /// Membuat keadaan sesi dengan versi format saat ini.
    pub fn new(fingerprint: TargetFingerprint, plan: AttackPlan, position: u64) -> SessionState {
        SessionState {
            version: SESSION_VERSION,
            fingerprint,
            plan,
            position,
            updated_at: unix_now(),
        }
    }
}

/// Hasil pencarian sebuah sesi tersimpan.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionLookup {
    /// Ada sesi yang cocok dan bisa dilanjutkan.
    Found(SessionState),
    /// Tidak ada sesi tersimpan untuk berkas ini.
    Absent,
    /// Ada sesi tersimpan, tetapi tidak lagi berlaku untuk berkas ini.
    ///
    /// Bedanya dengan `Absent` penting bagi pengguna: yang satu berarti "belum pernah
    /// dijalankan", yang satu lagi berarti "pernah dijalankan, tetapi berkasnya sudah
    /// berubah".
    Mismatch,
}

/// Tempat menyimpan sesi.
pub trait SessionStore: Send + Sync {
    /// Menyimpan atau menimpa sesi untuk berkas target.
    fn save(&self, state: &SessionState) -> Result<(), CoreError>;

    /// Mencari sesi untuk sebuah sidik jari.
    fn load(&self, fingerprint: &TargetFingerprint) -> Result<SessionLookup, CoreError>;

    /// Menghapus sesi untuk sebuah sidik jari. Tidak ada sesi juga bukan kesalahan.
    fn clear(&self, fingerprint: &TargetFingerprint) -> Result<(), CoreError>;
}

/// Sesi disimpan sebagai satu berkas JSON per berkas target.
#[derive(Debug, Clone)]
pub struct FileSessionStore {
    directory: PathBuf,
}

impl FileSessionStore {
    /// Menyimpan sesi di direktori tertentu.
    pub fn new(directory: impl Into<PathBuf>) -> FileSessionStore {
        FileSessionStore {
            directory: directory.into(),
        }
    }

    /// Menyimpan sesi di direktori data milik pengguna menurut sistem operasinya.
    ///
    /// Windows memakai `%APPDATA%\Zydra\sessions`, macOS memakai
    /// `~/Library/Application Support/Zydra/sessions`, dan Linux memakai
    /// `~/.local/share/zydra/sessions`.
    pub fn in_user_data_dir() -> Result<FileSessionStore, CoreError> {
        let directories =
            directories::ProjectDirs::from("", "", "Zydra").ok_or_else(|| CoreError::Session {
                detail: "no-user-data-directory".to_string(),
            })?;
        Ok(FileSessionStore::new(
            directories.data_dir().join("sessions"),
        ))
    }

    /// Direktori tempat sesi disimpan.
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Nama berkas sesi untuk sebuah sidik jari.
    ///
    /// Diturunkan dari lokasi berkas target saja, bukan dari seluruh sidik jarinya, supaya
    /// berkas sesi tetap ditemukan ketika berkas targetnya berubah. Nama aslinya ikut
    /// disertakan supaya isi direktori masih bisa dibaca manusia saat menyelidiki masalah.
    fn file_for(&self, fingerprint: &TargetFingerprint) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        fingerprint.path.hash(&mut hasher);
        let digest = hasher.finish();

        let stem = fingerprint
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "target".to_string());
        let stem: String = stem
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || character == '.' || character == '-' {
                    character
                } else {
                    '_'
                }
            })
            .take(40)
            .collect();

        self.directory.join(format!("{stem}-{digest:016x}.json"))
    }
}

impl SessionStore for FileSessionStore {
    fn save(&self, state: &SessionState) -> Result<(), CoreError> {
        std::fs::create_dir_all(&self.directory).map_err(|error| CoreError::Session {
            detail: format!("{}: {error}", self.directory.display()),
        })?;

        let path = self.file_for(&state.fingerprint);
        let encoded = serde_json::to_string_pretty(state).map_err(|error| CoreError::Session {
            detail: format!("encode: {error}"),
        })?;

        std::fs::write(&path, encoded).map_err(|error| CoreError::Session {
            detail: format!("{}: {error}", path.display()),
        })
    }

    fn load(&self, fingerprint: &TargetFingerprint) -> Result<SessionLookup, CoreError> {
        let path = self.file_for(fingerprint);
        let encoded = match std::fs::read_to_string(&path) {
            Ok(encoded) => encoded,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(SessionLookup::Absent)
            }
            Err(error) => {
                return Err(CoreError::Session {
                    detail: format!("{}: {error}", path.display()),
                })
            }
        };

        let state: SessionState =
            serde_json::from_str(&encoded).map_err(|error| CoreError::Session {
                detail: format!("{}: {error}", path.display()),
            })?;

        // Versi yang berbeda berarti bentuk berkasnya sudah berubah, dan menafsirkannya
        // dengan aturan sekarang hanya akan menghasilkan posisi yang salah.
        if state.version != SESSION_VERSION || state.fingerprint != *fingerprint {
            return Ok(SessionLookup::Mismatch);
        }

        Ok(SessionLookup::Found(state))
    }

    fn clear(&self, fingerprint: &TargetFingerprint) -> Result<(), CoreError> {
        let path = self.file_for(fingerprint);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(CoreError::Session {
                detail: format!("{}: {error}", path.display()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn target(directory: &Path, name: &str, contents: &[u8]) -> CrackTarget {
        let path = directory.join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(contents).unwrap();
        file.flush().unwrap();
        CrackTarget::from_path(&path)
    }

    fn store(directory: &Path) -> FileSessionStore {
        FileSessionStore::new(directory.join("sessions"))
    }

    fn plan() -> AttackPlan {
        AttackPlan::BruteForce {
            charset: crate::model::charset::CharsetSet::DIGITS,
            min_length: 1,
            max_length: 4,
        }
    }

    #[test]
    fn a_saved_session_can_be_loaded_back() {
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let fingerprint =
            TargetFingerprint::of(&target(directory.path(), "v.zip", b"data")).unwrap();
        let state = SessionState::new(fingerprint.clone(), plan(), 12_345);

        store.save(&state).unwrap();

        match store.load(&fingerprint).unwrap() {
            SessionLookup::Found(loaded) => {
                assert_eq!(loaded.position, 12_345);
                assert_eq!(loaded.plan, plan());
                assert_eq!(loaded.version, SESSION_VERSION);
                assert_eq!(loaded.fingerprint, fingerprint);
            }
            other => panic!("unexpected lookup: {other:?}"),
        }
    }

    #[test]
    fn nothing_saved_means_no_session() {
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let fingerprint =
            TargetFingerprint::of(&target(directory.path(), "v.zip", b"data")).unwrap();
        assert_eq!(store.load(&fingerprint).unwrap(), SessionLookup::Absent);
    }

    #[test]
    fn a_changed_target_file_makes_the_session_stale() {
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let target = target(directory.path(), "v.zip", b"original contents");
        let fingerprint = TargetFingerprint::of(&target).unwrap();
        store
            .save(&SessionState::new(fingerprint, plan(), 500))
            .unwrap();

        // Berkas targetnya berubah setelah sesi disimpan.
        std::fs::write(&target.path, b"contents that are longer than before").unwrap();
        let changed = TargetFingerprint::of(&target).unwrap();

        assert_eq!(store.load(&changed).unwrap(), SessionLookup::Mismatch);
    }

    #[test]
    fn a_session_from_another_format_version_is_stale() {
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let fingerprint =
            TargetFingerprint::of(&target(directory.path(), "v.zip", b"data")).unwrap();

        let mut state = SessionState::new(fingerprint.clone(), plan(), 10);
        state.version = SESSION_VERSION + 1;
        store.save(&state).unwrap();

        assert_eq!(store.load(&fingerprint).unwrap(), SessionLookup::Mismatch);
    }

    #[test]
    fn a_corrupt_session_file_is_reported_as_an_error() {
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let fingerprint =
            TargetFingerprint::of(&target(directory.path(), "v.zip", b"data")).unwrap();

        std::fs::create_dir_all(store.directory()).unwrap();
        std::fs::write(store.file_for(&fingerprint), b"{ this is not json").unwrap();

        assert!(matches!(
            store.load(&fingerprint),
            Err(CoreError::Session { .. })
        ));
    }

    #[test]
    fn clearing_removes_the_session() {
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let fingerprint =
            TargetFingerprint::of(&target(directory.path(), "v.zip", b"data")).unwrap();
        store
            .save(&SessionState::new(fingerprint.clone(), plan(), 7))
            .unwrap();

        store.clear(&fingerprint).unwrap();
        assert_eq!(store.load(&fingerprint).unwrap(), SessionLookup::Absent);
    }

    #[test]
    fn clearing_a_session_that_does_not_exist_is_not_an_error() {
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let fingerprint =
            TargetFingerprint::of(&target(directory.path(), "v.zip", b"data")).unwrap();
        assert!(store.clear(&fingerprint).is_ok());
    }

    #[test]
    fn different_targets_get_different_session_files() {
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let first = TargetFingerprint::of(&target(directory.path(), "a.zip", b"one")).unwrap();
        let second = TargetFingerprint::of(&target(directory.path(), "b.zip", b"two")).unwrap();

        store
            .save(&SessionState::new(first.clone(), plan(), 1))
            .unwrap();
        store
            .save(&SessionState::new(second.clone(), plan(), 2))
            .unwrap();

        match store.load(&first).unwrap() {
            SessionLookup::Found(state) => assert_eq!(state.position, 1),
            other => panic!("unexpected lookup: {other:?}"),
        }
        match store.load(&second).unwrap() {
            SessionLookup::Found(state) => assert_eq!(state.position, 2),
            other => panic!("unexpected lookup: {other:?}"),
        }
    }

    #[test]
    fn every_plan_shape_survives_a_round_trip() {
        // Bentuk berkas sesi bergantung pada bentuk AttackPlan, jadi setiap variannya diuji
        // supaya penambahan varian baru tidak diam-diam merusak sesi yang sudah ada.
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let fingerprint =
            TargetFingerprint::of(&target(directory.path(), "v.zip", b"data")).unwrap();

        let plans = [
            AttackPlan::Dictionary {
                wordlist: PathBuf::from("words.txt"),
            },
            AttackPlan::BruteForce {
                charset: crate::model::charset::CharsetSet::LOWERCASE
                    | crate::model::charset::CharsetSet::DIGITS,
                min_length: 2,
                max_length: 6,
            },
            AttackPlan::Mask {
                pattern: "?l?l?d?d".to_string(),
            },
            AttackPlan::WithRules {
                base: Box::new(AttackPlan::Mask {
                    pattern: "?d".to_string(),
                }),
                rules: "none,capitalize+append-digits:4".to_string(),
            },
        ];

        for plan in plans {
            store
                .save(&SessionState::new(fingerprint.clone(), plan.clone(), 3))
                .unwrap();
            match store.load(&fingerprint).unwrap() {
                SessionLookup::Found(state) => assert_eq!(state.plan, plan),
                other => panic!("unexpected lookup for {plan:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn the_fingerprint_records_the_size_and_the_kind() {
        let directory = tempfile::tempdir().unwrap();
        let target = target(directory.path(), "v.zip", b"1234567890");
        let fingerprint = TargetFingerprint::of(&target).unwrap();
        assert_eq!(fingerprint.size, 10);
        assert_eq!(fingerprint.kind, TargetKind::Zip);
        assert!(fingerprint.modified_unix > 0);
    }

    #[test]
    fn a_missing_target_has_no_fingerprint() {
        let target = CrackTarget::from_path("no-such-file-anywhere.zip");
        assert!(matches!(
            TargetFingerprint::of(&target),
            Err(CoreError::Session { .. })
        ));
    }

    #[test]
    fn the_session_file_name_is_readable_and_mentions_the_target() {
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let fingerprint =
            TargetFingerprint::of(&target(directory.path(), "vault.zip", b"x")).unwrap();
        let name = store.file_for(&fingerprint);
        let name = name.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with("vault.zip-"), "unexpected name: {name}");
        assert!(name.ends_with(".json"), "unexpected name: {name}");
    }

    #[test]
    fn a_target_name_with_odd_characters_still_produces_a_safe_file_name() {
        let directory = tempfile::tempdir().unwrap();
        let store = store(directory.path());
        let target = target(directory.path(), "weird name.rar", b"x");
        let fingerprint = TargetFingerprint::of(&target).unwrap();
        let name = store.file_for(&fingerprint);
        let name = name.file_name().unwrap().to_string_lossy().into_owned();
        assert!(
            name.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_'),
            "unsafe file name: {name}"
        );
    }
}
