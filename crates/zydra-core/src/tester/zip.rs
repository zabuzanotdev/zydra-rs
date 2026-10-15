//! Penguji untuk arsip ZIP.
//!
//! # Kenapa entri dibaca sampai habis
//!
//! ZipCrypto hanya menyediakan 1 dari 256 bit untuk memvalidasi password, sehingga
//! password yang salah bisa diterima saat header dibuka. Dokumentasi crate `zip`
//! menyatakan hal ini secara eksplisit: "This function sometimes accepts wrong password."
//!
//! Karena itu memeriksa beberapa kilobyte pertama tidak membuktikan apa pun. Satu-satunya
//! cara memastikan adalah membaca entri sampai habis dan bersandar pada CRC32 yang
//! diperiksa crate `zip` di akhir aliran. AES-256 tidak punya masalah ini karena punya
//! verifikasi HMAC, tetapi jalur kodenya tetap sama.

use std::fs::File;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use crate::constants::DEFAULT_ZIP_MAX_ENTRY_BYTES;
use crate::model::failure::{Backend, CrackFailure};
use crate::model::target::TargetKind;
use crate::tester::{EncryptionKind, MatchOutcome, PasswordTester, TesterInfo};

/// Penanda bahwa verifikasi berhenti di batas baca, bukan karena entri selesai.
pub const TRUNCATED_DETAIL: &str = "verification-truncated";

const READ_BUFFER_BYTES: usize = 8 * 1024;

/// Menguji kandidat password terhadap sebuah arsip ZIP.
pub struct ZipTester {
    path: PathBuf,
    entry_index: usize,
    max_entry_bytes: u64,
    info: TesterInfo,
}

impl ZipTester {
    /// Membuka arsip, memilih entri berkas pertama, dan mengenali jenis enkripsinya.
    pub fn open(path: &Path) -> Result<ZipTester, CrackFailure> {
        ZipTester::open_with_limit(path, DEFAULT_ZIP_MAX_ENTRY_BYTES)
    }

    /// Sama seperti [`ZipTester::open`] tetapi dengan batas baca yang bisa diatur.
    pub fn open_with_limit(path: &Path, max_entry_bytes: u64) -> Result<ZipTester, CrackFailure> {
        let file = File::open(path).map_err(|error| CrackFailure::FileUnreadable {
            reason: format!("{}: {error}", path.display()),
        })?;
        let mut archive =
            ZipArchive::new(file).map_err(|error| CrackFailure::BackendUnavailable {
                backend: Backend::Zip,
                reason: format!("{}: {error}", path.display()),
            })?;

        let entries = archive.len();
        let entry_index = first_file_index(&archive).ok_or(CrackFailure::BackendUnavailable {
            backend: Backend::Zip,
            reason: "no-file-entries".to_string(),
        })?;
        let encryption = detect_encryption(&mut archive, entry_index);

        Ok(ZipTester {
            path: path.to_path_buf(),
            entry_index,
            max_entry_bytes,
            info: TesterInfo {
                kind: TargetKind::Zip,
                encryption,
                entries,
            },
        })
    }
}

/// Indeks entri pertama yang bukan direktori.
fn first_file_index<R: Read + Seek>(archive: &ZipArchive<R>) -> Option<usize> {
    (0..archive.len()).find(|&index| {
        archive
            .name_for_index(index)
            .and_then(Result::ok)
            .map(|name| !name.ends_with('/'))
            .unwrap_or(false)
    })
}

/// Menebak jenis enkripsi entri.
///
/// AES dikenali dari adanya bahan verifikasi AES. Kalau bukan AES tetapi entri tidak bisa
/// dibuka tanpa password, arsipnya memakai ZipCrypto. Selain itu arsipnya tidak terenkripsi.
fn detect_encryption<R: Read + Seek>(archive: &mut ZipArchive<R>, index: usize) -> EncryptionKind {
    if matches!(
        archive.get_aes_verification_key_and_salt(index),
        Ok(Some(_))
    ) {
        return EncryptionKind::ZipAes256;
    }
    match archive.by_index(index) {
        Ok(_) => EncryptionKind::None,
        Err(_) => EncryptionKind::ZipCrypto,
    }
}

impl PasswordTester for ZipTester {
    fn try_password(&mut self, password: &str) -> MatchOutcome {
        // Arsip dibuka ulang untuk setiap kandidat. Setiap worker memegang pengujinya
        // sendiri, jadi tidak ada state bersama yang perlu dikunci.
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(_) => return MatchOutcome::NoMatch,
        };
        let mut archive = match ZipArchive::new(file) {
            Ok(archive) => archive,
            Err(_) => return MatchOutcome::NoMatch,
        };
        let mut entry = match archive.by_index_decrypt(self.entry_index, password.as_bytes()) {
            Ok(entry) => entry,
            Err(_) => return MatchOutcome::NoMatch,
        };

        let mut buffer = [0u8; READ_BUFFER_BYTES];
        let mut read_total: u64 = 0;
        loop {
            match entry.read(&mut buffer) {
                // Sampai di sini berarti crate `zip` sudah memeriksa CRC32 entri dan
                // tidak menemukan ketidakcocokan.
                Ok(0) => return MatchOutcome::Matched { detail: None },
                Ok(bytes) => {
                    read_total += bytes as u64;
                    if read_total >= self.max_entry_bytes {
                        return MatchOutcome::Matched {
                            detail: Some(TRUNCATED_DETAIL.to_string()),
                        };
                    }
                }
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
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::AesMode;

    const SECRET: &str = "correct horse battery staple";
    const CONTENT: &[u8] = b"isi rahasia yang cukup panjang untuk melewati beberapa buffer";

    /// Membuat arsip ZIP berenkripsi AES-256 di berkas sementara.
    fn encrypted_archive() -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut writer = zip::ZipWriter::new(file.reopen().unwrap());
        let options = SimpleFileOptions::default().with_aes_encryption(AesMode::Aes256, SECRET);
        writer.start_file("secret.txt", options).unwrap();
        writer.write_all(CONTENT).unwrap();
        writer.finish().unwrap();
        file
    }

    #[test]
    fn the_correct_password_is_found() {
        let archive = encrypted_archive();
        let mut tester = ZipTester::open(archive.path()).unwrap();
        assert_eq!(
            tester.try_password(SECRET),
            MatchOutcome::Matched { detail: None }
        );
    }

    #[test]
    fn a_wrong_password_is_rejected() {
        let archive = encrypted_archive();
        let mut tester = ZipTester::open(archive.path()).unwrap();
        assert_eq!(
            tester.try_password("not the password"),
            MatchOutcome::NoMatch
        );
    }

    #[test]
    fn a_wrong_password_that_shares_a_prefix_is_still_rejected() {
        let archive = encrypted_archive();
        let mut tester = ZipTester::open(archive.path()).unwrap();
        assert_eq!(
            tester.try_password("correct horse battery stapl"),
            MatchOutcome::NoMatch
        );
    }

    #[test]
    fn aes_encryption_is_detected() {
        let archive = encrypted_archive();
        let tester = ZipTester::open(archive.path()).unwrap();
        assert_eq!(tester.info().kind, TargetKind::Zip);
        assert_eq!(tester.info().encryption, EncryptionKind::ZipAes256);
        assert_eq!(tester.info().entries, 1);
    }

    #[test]
    fn an_unencrypted_archive_is_recognised_as_such() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut writer = zip::ZipWriter::new(file.reopen().unwrap());
        writer
            .start_file("plain.txt", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"no secret here").unwrap();
        writer.finish().unwrap();

        let tester = ZipTester::open(file.path()).unwrap();
        assert_eq!(tester.info().encryption, EncryptionKind::None);
        // Bahkan tanpa enkripsi, password kosong membuka entri dan isinya terbaca.
        let mut tester = tester;
        assert!(matches!(
            tester.try_password(""),
            MatchOutcome::Matched { .. }
        ));
    }

    #[test]
    fn a_directory_entry_is_skipped_when_choosing_what_to_read() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut writer = zip::ZipWriter::new(file.reopen().unwrap());
        writer
            .add_directory("folder/", SimpleFileOptions::default())
            .unwrap();
        let options = SimpleFileOptions::default().with_aes_encryption(AesMode::Aes256, SECRET);
        writer.start_file("folder/secret.txt", options).unwrap();
        writer.write_all(CONTENT).unwrap();
        writer.finish().unwrap();

        let mut tester = ZipTester::open(file.path()).unwrap();
        assert_eq!(tester.info().entries, 2);
        assert_eq!(
            tester.try_password(SECRET),
            MatchOutcome::Matched { detail: None }
        );
    }

    #[test]
    fn a_truncated_verification_is_reported_as_such() {
        let archive = encrypted_archive();
        let mut tester = ZipTester::open_with_limit(archive.path(), 8).unwrap();
        assert_eq!(
            tester.try_password(SECRET),
            MatchOutcome::Matched {
                detail: Some(TRUNCATED_DETAIL.to_string())
            }
        );
    }

    #[test]
    fn a_file_that_is_not_an_archive_is_reported_as_unavailable() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"this is definitely not a zip archive")
            .unwrap();
        file.flush().unwrap();
        assert!(matches!(
            ZipTester::open(file.path()),
            Err(CrackFailure::BackendUnavailable { .. })
        ));
    }

    #[test]
    fn a_missing_file_is_reported_as_unreadable() {
        assert!(matches!(
            ZipTester::open(Path::new("no-such-archive.zip")),
            Err(CrackFailure::FileUnreadable { .. })
        ));
    }
}
