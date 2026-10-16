//! Pengaturan yang bertahan antar sesi.
//!
//! Disimpan sebagai TOML di direktori konfigurasi milik sistem operasinya. Berkas yang rusak
//! atau tidak lengkap tidak membuat aplikasi gagal jalan: nilai yang tidak terbaca diganti
//! nilai bawaan, dan sebabnya dikembalikan supaya pemanggil bisa mencatatnya.
//!
//! # Kenapa nilai bawaannya ada di satu tempat
//!
//! Nilai bawaan yang ditulis di dua tempat akan berbeda cepat atau lambat, dan yang paling
//! membingungkan adalah ketika berkas konfigurasi yang kosong menghasilkan perilaku yang
//! berbeda dari berkas yang tidak ada. Karena itu keduanya melewati jalur yang sama.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::constants::{
    DEFAULT_BATCH_SIZE, DEFAULT_LOG_CAPACITY, DEFAULT_MAX_BRUTE_LENGTH, DEFAULT_MAX_COMBINATIONS,
    DEFAULT_PROGRESS_INTERVAL_MS, DEFAULT_WORKERS, DEFAULT_ZIP_MAX_ENTRY_BYTES,
};
use crate::error::CoreError;

/// Bahasa antarmuka yang dipilih pengguna.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LanguageChoice {
    /// Ikuti pengaturan sistem.
    #[default]
    Auto,
    /// Bahasa Inggris.
    En,
    /// Bahasa Indonesia.
    Id,
}

/// Pengaturan yang bertahan antar sesi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    /// Bahasa antarmuka.
    pub language: LanguageChoice,
    /// Jumlah worker. `0` berarti mengikuti jumlah CPU.
    pub workers: usize,
    /// Batas jumlah kombinasi sebelum serangan ditolak.
    pub max_combinations: u64,
    /// Panjang maksimum yang diizinkan untuk brute force.
    pub max_brute_length: u8,
    /// Jumlah kandidat yang diambil satu worker dalam sekali kunci.
    pub batch_size: usize,
    /// Jarak antar peristiwa progres, dalam milidetik.
    pub progress_interval_ms: u64,
    /// Batas byte entri ZIP yang dibaca saat memverifikasi kandidat.
    pub zip_max_entry_bytes: u64,
    /// Jumlah maksimum entri log yang disimpan di memori.
    pub log_keep_entries: usize,
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig {
            language: LanguageChoice::Auto,
            workers: DEFAULT_WORKERS,
            max_combinations: DEFAULT_MAX_COMBINATIONS,
            max_brute_length: DEFAULT_MAX_BRUTE_LENGTH,
            batch_size: DEFAULT_BATCH_SIZE,
            progress_interval_ms: DEFAULT_PROGRESS_INTERVAL_MS,
            zip_max_entry_bytes: DEFAULT_ZIP_MAX_ENTRY_BYTES,
            log_keep_entries: DEFAULT_LOG_CAPACITY,
        }
    }
}

impl AppConfig {
    /// Menyusun pengaturan dari teks TOML.
    ///
    /// Bidang yang tidak ada memakai nilai bawaan, karena `serde(default)` di tingkat
    /// struktur. Bidang yang ada tetapi tidak masuk akal ditolak, dan pemanggil yang
    /// memutuskan apa yang harus dilakukan.
    pub fn from_toml(text: &str) -> Result<AppConfig, CoreError> {
        let config: AppConfig = toml::from_str(text).map_err(|error| CoreError::ConfigInvalid {
            detail: error.to_string(),
        })?;
        config.validate()?;
        Ok(config)
    }

    /// Menuliskan pengaturan sebagai TOML.
    pub fn to_toml(&self) -> Result<String, CoreError> {
        toml::to_string_pretty(self).map_err(|error| CoreError::ConfigInvalid {
            detail: error.to_string(),
        })
    }

    /// Memeriksa nilai yang bisa membuat serangan tidak masuk akal.
    ///
    /// Nilai yang salah diterima apa adanya akan menghasilkan perilaku yang membingungkan
    /// jauh dari tempat kesalahannya, jadi lebih baik ditolak di sini.
    pub fn validate(&self) -> Result<(), CoreError> {
        let invalid = |detail: &str| CoreError::ConfigInvalid {
            detail: detail.to_string(),
        };

        if self.batch_size == 0 {
            return Err(invalid("batch_size must be at least 1"));
        }
        if self.max_combinations == 0 {
            return Err(invalid("max_combinations must be at least 1"));
        }
        if self.max_brute_length == 0 {
            return Err(invalid("max_brute_length must be at least 1"));
        }
        if self.log_keep_entries == 0 {
            return Err(invalid("log_keep_entries must be at least 1"));
        }
        if self.zip_max_entry_bytes == 0 {
            return Err(invalid("zip_max_entry_bytes must be at least 1"));
        }
        if self.progress_interval_ms == 0 {
            return Err(invalid("progress_interval_ms must be at least 1"));
        }
        Ok(())
    }

    /// Mengembalikan seluruh nilai ke bawaan.
    pub fn reset(&mut self) {
        *self = AppConfig::default();
    }
}

/// Membaca dan menulis pengaturan pada sebuah direktori.
///
/// Direktori dipisah dari `AppConfig` supaya pengujian bisa memakai direktori sementara, dan
/// supaya lokasi penyimpanan bisa ditentukan antarmuka tanpa mengubah bentuk pengaturannya.
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    /// Menyimpan pengaturan di berkas tertentu.
    pub fn at(path: impl Into<PathBuf>) -> ConfigStore {
        ConfigStore { path: path.into() }
    }

    /// Menyimpan pengaturan di direktori konfigurasi milik pengguna.
    ///
    /// Variabel lingkungan `ZYDRA_CONFIG_DIR` menimpanya, sama seperti direktori sesi. Itu
    /// berguna untuk pemasangan portabel dan untuk pengujian.
    pub fn in_user_config_dir() -> Result<ConfigStore, CoreError> {
        if let Ok(directory) = std::env::var(CONFIG_DIR_VARIABLE) {
            if !directory.trim().is_empty() {
                return Ok(ConfigStore::at(PathBuf::from(directory).join(FILE_NAME)));
            }
        }
        let directories = directories::ProjectDirs::from("", "", "Zydra").ok_or_else(|| {
            CoreError::ConfigInvalid {
                detail: "no-user-config-directory".to_string(),
            }
        })?;
        Ok(ConfigStore::at(directories.config_dir().join(FILE_NAME)))
    }

    /// Lokasi berkasnya.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Memuat pengaturan.
    ///
    /// Berkas yang tidak ada berarti bawaan, dan itu bukan kesalahan. Berkas yang rusak juga
    /// menghasilkan bawaan, tetapi sebabnya dikembalikan supaya bisa dicatat: pengguna yang
    /// mengubah berkasnya secara manual perlu tahu bahwa perubahannya diabaikan.
    pub fn load(&self) -> (AppConfig, Option<CoreError>) {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return (AppConfig::default(), None)
            }
            Err(error) => {
                return (
                    AppConfig::default(),
                    Some(CoreError::ConfigInvalid {
                        detail: format!("{}: {error}", self.path.display()),
                    }),
                )
            }
        };

        match AppConfig::from_toml(&text) {
            Ok(config) => (config, None),
            Err(error) => (AppConfig::default(), Some(error)),
        }
    }

    /// Menyimpan pengaturan, membuat direktorinya kalau perlu.
    pub fn save(&self, config: &AppConfig) -> Result<(), CoreError> {
        config.validate()?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| CoreError::ConfigInvalid {
                detail: format!("{}: {error}", parent.display()),
            })?;
        }
        let text = config.to_toml()?;
        std::fs::write(&self.path, text).map_err(|error| CoreError::ConfigInvalid {
            detail: format!("{}: {error}", self.path.display()),
        })
    }
}

/// Variabel lingkungan yang menimpa direktori konfigurasi.
pub const CONFIG_DIR_VARIABLE: &str = "ZYDRA_CONFIG_DIR";

/// Nama berkas pengaturan.
pub const FILE_NAME: &str = "config.toml";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_match_the_constants() {
        let config = AppConfig::default();
        assert_eq!(config.workers, DEFAULT_WORKERS);
        assert_eq!(config.max_combinations, DEFAULT_MAX_COMBINATIONS);
        assert_eq!(config.max_brute_length, DEFAULT_MAX_BRUTE_LENGTH);
        assert_eq!(config.batch_size, DEFAULT_BATCH_SIZE);
        assert_eq!(config.log_keep_entries, DEFAULT_LOG_CAPACITY);
    }

    #[test]
    fn a_configuration_survives_a_round_trip() {
        let config = AppConfig {
            language: LanguageChoice::Id,
            workers: 6,
            max_combinations: 500,
            max_brute_length: 5,
            ..AppConfig::default()
        };
        let text = config.to_toml().unwrap();
        assert_eq!(AppConfig::from_toml(&text).unwrap(), config);
    }

    #[test]
    fn missing_fields_fall_back_to_their_defaults() {
        // Ini yang membuat berkas konfigurasi lama tetap bisa dibaca setelah bidang baru
        // ditambahkan.
        let config = AppConfig::from_toml("workers = 3\n").unwrap();
        assert_eq!(config.workers, 3);
        assert_eq!(config.max_combinations, DEFAULT_MAX_COMBINATIONS);
        assert_eq!(config.language, LanguageChoice::Auto);
    }

    #[test]
    fn an_empty_file_is_the_same_as_no_file() {
        // Kalau keduanya berbeda, pengguna yang menyentuh berkasnya akan mendapat perilaku
        // yang berbeda tanpa sebab yang jelas.
        assert_eq!(AppConfig::from_toml("").unwrap(), AppConfig::default());
    }

    #[test]
    fn the_language_is_written_as_a_lowercase_word() {
        let text = AppConfig {
            language: LanguageChoice::Id,
            ..AppConfig::default()
        }
        .to_toml()
        .unwrap();
        assert!(text.contains("language = \"id\""), "{text}");
    }

    #[test]
    fn an_unknown_language_is_rejected() {
        assert!(AppConfig::from_toml("language = \"fr\"").is_err());
    }

    #[test]
    fn an_unknown_field_is_rejected_instead_of_silently_ignored() {
        // Salah ketik nama bidang akan membuat pengguna mengira perubahannya berlaku.
        assert!(AppConfig::from_toml("worker = 4").is_err());
    }

    #[test]
    fn malformed_toml_is_rejected() {
        assert!(AppConfig::from_toml("workers = ").is_err());
    }

    #[test]
    fn values_that_would_break_a_run_are_rejected() {
        for text in [
            "batch_size = 0",
            "max_combinations = 0",
            "max_brute_length = 0",
            "log_keep_entries = 0",
            "zip_max_entry_bytes = 0",
            "progress_interval_ms = 0",
        ] {
            assert!(
                AppConfig::from_toml(text).is_err(),
                "{text} should have been rejected"
            );
        }
    }

    #[test]
    fn resetting_returns_every_value_to_its_default() {
        let mut config = AppConfig {
            workers: 9,
            max_brute_length: 8,
            language: LanguageChoice::En,
            ..AppConfig::default()
        };
        config.reset();
        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn a_missing_file_loads_the_defaults_without_complaint() {
        let directory = tempfile::tempdir().unwrap();
        let store = ConfigStore::at(directory.path().join("config.toml"));
        let (config, problem) = store.load();
        assert_eq!(config, AppConfig::default());
        assert!(problem.is_none(), "a missing file is not a problem");
    }

    #[test]
    fn a_corrupt_file_loads_the_defaults_and_says_why() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        std::fs::write(&path, b"this is not toml at all = = =").unwrap();

        let store = ConfigStore::at(&path);
        let (config, problem) = store.load();
        assert_eq!(config, AppConfig::default());
        assert!(problem.is_some(), "a corrupt file should be reported");
    }

    #[test]
    fn saving_creates_the_directory_and_the_file_can_be_read_back() {
        let directory = tempfile::tempdir().unwrap();
        let store = ConfigStore::at(directory.path().join("nested").join("config.toml"));
        let config = AppConfig {
            workers: 4,
            ..AppConfig::default()
        };

        store.save(&config).unwrap();

        let (loaded, problem) = store.load();
        assert!(problem.is_none());
        assert_eq!(loaded, config);
    }

    #[test]
    fn saving_rejects_a_configuration_that_would_break_a_run() {
        let directory = tempfile::tempdir().unwrap();
        let store = ConfigStore::at(directory.path().join("config.toml"));
        let broken = AppConfig {
            batch_size: 0,
            ..AppConfig::default()
        };
        assert!(store.save(&broken).is_err());
    }

    #[test]
    fn the_saved_file_is_readable_by_a_person() {
        let directory = tempfile::tempdir().unwrap();
        let store = ConfigStore::at(directory.path().join("config.toml"));
        store.save(&AppConfig::default()).unwrap();
        let text = std::fs::read_to_string(store.path()).unwrap();
        assert!(text.contains("workers"), "{text}");
        assert!(text.contains("max_combinations"), "{text}");
    }
}
