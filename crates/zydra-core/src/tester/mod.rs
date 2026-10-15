//! Kontrak untuk menguji satu kandidat password terhadap sebuah target.
//!
//! Mesin pemecah hanya mengenal trait di modul ini. Implementasi nyata untuk tiap format
//! tinggal di modul `zip`, `rar`, `pdf`, dan `shadow`.

pub mod factory;
pub mod shadow;
pub mod zip;

use crate::model::failure::CrackFailure;
use crate::model::target::{CrackTarget, TargetKind};

/// Hasil satu percobaan password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchOutcome {
    /// Kandidat tidak cocok.
    NoMatch,
    /// Kandidat cocok.
    Matched {
        /// Keterangan tambahan, misalnya nama pengguna pada berkas `shadow`.
        detail: Option<String>,
    },
}

/// Skema hash `crypt(3)` pada berkas `shadow`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnixCryptScheme {
    /// `descrypt`.
    Des,
    /// `md5crypt`.
    Md5,
    /// `sha256crypt`.
    Sha256,
    /// `sha512crypt`.
    Sha512,
    /// `bcrypt`.
    Bcrypt,
    /// Skema lain, disimpan apa adanya.
    Other(String),
}

/// Jenis enkripsi yang terdeteksi pada target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionKind {
    /// Berkas tidak terenkripsi.
    None,
    /// ZIP dengan ZipCrypto.
    ZipCrypto,
    /// ZIP dengan AES-256.
    ZipAes256,
    /// RAR4.
    RarCrypto4,
    /// RAR5.
    RarCrypto5,
    /// PDF standard security handler.
    PdfStandard,
    /// Hash `crypt(3)`.
    UnixCrypt(UnixCryptScheme),
}

/// Keterangan tentang target yang sedang diuji.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TesterInfo {
    /// Jenis berkas.
    pub kind: TargetKind,
    /// Enkripsi yang terdeteksi.
    pub encryption: EncryptionKind,
    /// Jumlah entri di dalam arsip, atau `1` untuk berkas tunggal.
    pub entries: usize,
}

/// Menguji satu kandidat password.
///
/// Setiap worker memegang pengujinya sendiri, sehingga `&mut self` aman dipakai tanpa
/// penguncian.
pub trait PasswordTester: Send {
    /// Mencoba satu password.
    fn try_password(&mut self, password: &str) -> MatchOutcome;

    /// Keterangan tentang target.
    fn info(&self) -> &TesterInfo;
}

/// Membuat penguji untuk sebuah target.
///
/// Dipanggil satu kali per worker. Implementasi bertanggung jawab melakukan pekerjaan
/// mahal, seperti membaca header arsip, satu kali saja dan membagikannya ke penguji
/// berikutnya.
pub trait TesterFactory: Send + Sync {
    /// Membuat penguji, atau mengembalikan sebab target tidak bisa diuji.
    fn create(&self, target: &CrackTarget) -> Result<Box<dyn PasswordTester>, CrackFailure>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::target::CrackTarget;

    struct AlwaysMatches {
        info: TesterInfo,
    }

    impl PasswordTester for AlwaysMatches {
        fn try_password(&mut self, _password: &str) -> MatchOutcome {
            MatchOutcome::Matched { detail: None }
        }
        fn info(&self) -> &TesterInfo {
            &self.info
        }
    }

    struct FixedFactory;

    impl TesterFactory for FixedFactory {
        fn create(&self, _target: &CrackTarget) -> Result<Box<dyn PasswordTester>, CrackFailure> {
            Ok(Box::new(AlwaysMatches {
                info: TesterInfo {
                    kind: TargetKind::Zip,
                    encryption: EncryptionKind::ZipCrypto,
                    entries: 1,
                },
            }))
        }
    }

    #[test]
    fn a_tester_reports_its_own_description() {
        let target = CrackTarget::from_path("vault.zip");
        let mut tester = FixedFactory.create(&target).unwrap();
        assert_eq!(tester.info().kind, TargetKind::Zip);
        assert_eq!(tester.info().encryption, EncryptionKind::ZipCrypto);
        assert_eq!(tester.info().entries, 1);
        assert_eq!(
            tester.try_password("anything"),
            MatchOutcome::Matched { detail: None }
        );
    }

    #[test]
    fn a_factory_can_be_shared_across_threads() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FixedFactory>();
    }

    #[test]
    fn a_tester_can_be_moved_to_another_thread() {
        fn assert_send<T: Send>() {}
        assert_send::<Box<dyn PasswordTester>>();
    }
}
