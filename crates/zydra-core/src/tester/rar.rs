//! Penguji untuk arsip RAR.
//!
//! Memakai crate `unrar`, yang membundel libunrar dari RARLAB dan mengompilasinya dari
//! sumber. Versi Python hanya mendukung RAR4 dan bergantung pada paket `rarfile`; di sini
//! RAR4 dan RAR5 dua-duanya didukung.
//!
//! # Hanya entri terenkripsi pertama yang diuji
//!
//! Sebuah arsip bisa memuat campuran entri terenkripsi dan tidak. Menguji entri yang tidak
//! terenkripsi akan melaporkan kecocokan untuk password apa pun, jadi entri seperti itu
//! dilewati sampai ketemu yang benar-benar terenkripsi.
//!
//! Yang diuji hanya entri terenkripsi pertama. Alasannya teknis: `read` pada crate `unrar`
//! memakan arsipnya, jadi setelah pembacaan gagal tidak ada cara melanjutkan ke entri
//! berikutnya tanpa membuka ulang seluruh arsip. Membuka ulang untuk setiap kandidat akan
//! mengubah serangan menjadi jauh lebih lambat, dan pada arsip nyata seluruh entri memakai
//! password yang sama sehingga entri pertama sudah mewakili.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use unrar::Archive;

use crate::model::failure::{Backend, CrackFailure};
use crate::model::target::TargetKind;
use crate::tester::{EncryptionKind, MatchOutcome, PasswordTester, TesterInfo};

/// Tanda tangan berkas RAR versi 4.
const RAR4_SIGNATURE: [u8; 7] = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00];

/// Tanda tangan berkas RAR versi 5.
const RAR5_SIGNATURE: [u8; 8] = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00];

/// Menguji kandidat password terhadap sebuah arsip RAR.
pub struct RarTester {
    path: PathBuf,
    info: TesterInfo,
}

impl RarTester {
    /// Membuka arsip untuk mengenali versinya dan menghitung entrinya.
    pub fn open(path: &Path) -> Result<RarTester, CrackFailure> {
        let encryption = detect_version(path)?;
        Ok(RarTester {
            path: path.to_path_buf(),
            info: TesterInfo {
                kind: TargetKind::Rar,
                encryption,
                entries: count_entries(path),
            },
        })
    }
}

/// Mengenali versi arsip dari tanda tangannya.
///
/// Ini satu-satunya tempat yang membaca berkas sendiri. Selebihnya diserahkan ke libunrar,
/// yang menangani perbedaan format antara RAR4 dan RAR5.
fn detect_version(path: &Path) -> Result<EncryptionKind, CrackFailure> {
    let mut file = File::open(path).map_err(|error| CrackFailure::FileUnreadable {
        reason: format!("{}: {error}", path.display()),
    })?;

    let mut header = [0u8; 8];
    let read = file
        .read(&mut header)
        .map_err(|error| CrackFailure::FileUnreadable {
            reason: format!("{}: {error}", path.display()),
        })?;

    if read >= RAR5_SIGNATURE.len() && header == RAR5_SIGNATURE {
        return Ok(EncryptionKind::RarCrypto5);
    }
    if read >= RAR4_SIGNATURE.len() && header[..RAR4_SIGNATURE.len()] == RAR4_SIGNATURE {
        return Ok(EncryptionKind::RarCrypto4);
    }

    Err(CrackFailure::BackendUnavailable {
        backend: Backend::Rar,
        reason: "not-a-rar-archive".to_string(),
    })
}

/// Menghitung entri dengan mendaftar arsip tanpa password.
///
/// Daftar gagal pada arsip yang headernya ikut terenkripsi. Jumlah entri hanya keterangan
/// tambahan, jadi kegagalan di sini tidak membuat arsipnya dianggap tidak bisa diuji.
fn count_entries(path: &Path) -> usize {
    match Archive::new(path).open_for_listing() {
        Ok(listing) => listing.filter(Result::is_ok).count(),
        Err(_) => 0,
    }
}

impl PasswordTester for RarTester {
    fn try_password(&mut self, password: &str) -> MatchOutcome {
        let archive = Archive::with_password(&self.path, password);
        let Ok(cursor) = archive.open_for_processing() else {
            return MatchOutcome::NoMatch;
        };

        let mut cursor = cursor;
        loop {
            // Header yang gagal dibaca berarti passwordnya salah pada arsip yang headernya
            // ikut terenkripsi.
            let Ok(header) = cursor.read_header() else {
                return MatchOutcome::NoMatch;
            };
            let Some(next) = header else {
                // Seluruh entri habis dan tidak ada satu pun yang terenkripsi.
                return MatchOutcome::NoMatch;
            };

            if next.entry().is_encrypted() {
                let name = next.entry().filename.to_string_lossy().into_owned();
                return match next.read() {
                    Ok(_) => MatchOutcome::Matched { detail: Some(name) },
                    Err(_) => MatchOutcome::NoMatch,
                };
            }

            // Entri tanpa enkripsi akan cocok dengan password apa pun, jadi dilewati.
            match next.skip() {
                Ok(next_header) => cursor = next_header,
                Err(_) => return MatchOutcome::NoMatch,
            }
        }
    }

    fn info(&self) -> &TesterInfo {
        &self.info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    /// Password yang dipakai libarchive untuk berkas ujinya.
    ///
    /// Berkas `test_read_format_rar_encryption.c` di repositori libarchive menyatakannya:
    /// pada arsip solid atau arsip dengan nama berkas terenkripsi, seluruh berkas memakai
    /// password `password`.
    const SECRET: &str = "password";

    #[test]
    fn the_correct_password_opens_a_solid_rar4_archive() {
        let mut tester = RarTester::open(&fixture("rar4_solid_encrypted.rar")).unwrap();
        assert!(matches!(
            tester.try_password(SECRET),
            MatchOutcome::Matched { .. }
        ));
    }

    #[test]
    fn a_wrong_password_is_rejected_on_a_solid_rar4_archive() {
        let mut tester = RarTester::open(&fixture("rar4_solid_encrypted.rar")).unwrap();
        assert_eq!(tester.try_password("wrong"), MatchOutcome::NoMatch);
    }

    #[test]
    fn an_empty_password_is_rejected() {
        let mut tester = RarTester::open(&fixture("rar4_solid_encrypted.rar")).unwrap();
        assert_eq!(tester.try_password(""), MatchOutcome::NoMatch);
    }

    #[test]
    fn the_correct_password_opens_a_rar5_archive() {
        let mut tester = RarTester::open(&fixture("rar5_encrypted.rar")).unwrap();
        assert!(matches!(
            tester.try_password(SECRET),
            MatchOutcome::Matched { .. }
        ));
    }

    #[test]
    fn a_wrong_password_is_rejected_on_a_rar5_archive() {
        let mut tester = RarTester::open(&fixture("rar5_encrypted.rar")).unwrap();
        assert_eq!(tester.try_password("wrong"), MatchOutcome::NoMatch);
    }

    #[test]
    fn the_matched_entry_is_named_in_the_detail() {
        let mut tester = RarTester::open(&fixture("rar4_solid_encrypted.rar")).unwrap();
        match tester.try_password(SECRET) {
            MatchOutcome::Matched { detail } => {
                let detail = detail.expect("the entry name should be reported");
                assert!(detail.ends_with(".txt"), "unexpected detail: {detail}");
            }
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    #[test]
    fn unencrypted_entries_are_skipped_rather_than_accepted() {
        // Arsip ini memuat campuran: entri pertama tidak terenkripsi, lalu ada yang
        // terenkripsi dengan `password` dan ada yang dengan `password2`. Kalau entri tanpa
        // enkripsi ikut diuji, password apa pun akan dilaporkan cocok.
        let mut tester = RarTester::open(&fixture("rar4_encrypted.rar")).unwrap();
        assert_eq!(tester.try_password("totally wrong"), MatchOutcome::NoMatch);
        assert!(matches!(
            tester.try_password(SECRET),
            MatchOutcome::Matched { .. }
        ));
    }

    #[test]
    fn the_rar4_version_is_detected() {
        let tester = RarTester::open(&fixture("rar4_solid_encrypted.rar")).unwrap();
        assert_eq!(tester.info().kind, TargetKind::Rar);
        assert_eq!(tester.info().encryption, EncryptionKind::RarCrypto4);
    }

    #[test]
    fn the_rar5_version_is_detected() {
        let tester = RarTester::open(&fixture("rar5_encrypted.rar")).unwrap();
        assert_eq!(tester.info().encryption, EncryptionKind::RarCrypto5);
    }

    #[test]
    fn entries_are_counted() {
        let tester = RarTester::open(&fixture("rar4_solid_encrypted.rar")).unwrap();
        assert!(tester.info().entries > 0, "no entries were counted");
    }

    #[test]
    fn a_file_that_is_not_an_archive_is_reported_as_unavailable() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, b"this is definitely not a RAR archive").unwrap();
        std::io::Write::flush(&mut file).unwrap();
        assert!(matches!(
            RarTester::open(file.path()),
            Err(CrackFailure::BackendUnavailable { .. })
        ));
    }

    #[test]
    fn a_missing_file_is_reported_as_unreadable() {
        assert!(matches!(
            RarTester::open(Path::new("no-such-archive.rar")),
            Err(CrackFailure::FileUnreadable { .. })
        ));
    }
}
