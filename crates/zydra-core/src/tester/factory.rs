//! Pemilihan penguji berdasarkan jenis berkas target.
//!
//! Mesin paralel memanggil [`TesterFactory::create`] satu kali per worker, jadi pekerjaan
//! mahal seperti membaca header arsip terjadi di sini, bukan di dalam loop kandidat.

use crate::constants::DEFAULT_ZIP_MAX_ENTRY_BYTES;
use crate::model::failure::CrackFailure;
use crate::model::target::{CrackTarget, TargetKind};
use crate::tester::pdf::PdfTester;
use crate::tester::rar::RarTester;
use crate::tester::shadow::ShadowTester;
use crate::tester::zip::ZipTester;
use crate::tester::{PasswordTester, TesterFactory};

/// Factory yang memetakan setiap jenis target ke pengujinya.
pub struct DefaultTesterFactory {
    zip_max_entry_bytes: u64,
}

impl DefaultTesterFactory {
    /// Factory dengan batas baca ZIP bawaan.
    pub fn new() -> DefaultTesterFactory {
        DefaultTesterFactory {
            zip_max_entry_bytes: DEFAULT_ZIP_MAX_ENTRY_BYTES,
        }
    }

    /// Factory dengan batas baca ZIP yang bisa diatur. Dipakai pengujian dan konfigurasi.
    pub fn with_zip_max_entry_bytes(zip_max_entry_bytes: u64) -> DefaultTesterFactory {
        DefaultTesterFactory {
            zip_max_entry_bytes,
        }
    }
}

impl Default for DefaultTesterFactory {
    fn default() -> DefaultTesterFactory {
        DefaultTesterFactory::new()
    }
}

impl TesterFactory for DefaultTesterFactory {
    fn create(&self, target: &CrackTarget) -> Result<Box<dyn PasswordTester>, CrackFailure> {
        match target.kind {
            TargetKind::Zip => Ok(Box::new(ZipTester::open_with_limit(
                &target.path,
                self.zip_max_entry_bytes,
            )?)),
            TargetKind::Shadow => Ok(Box::new(ShadowTester::open(&target.path)?)),
            TargetKind::Pdf => Ok(Box::new(PdfTester::open(&target.path)?)),
            TargetKind::Rar => Ok(Box::new(RarTester::open(&target.path)?)),
            TargetKind::Unsupported => Err(CrackFailure::UnsupportedFormat {
                display_name: target.display_name.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::failure::Backend;
    use crate::tester::EncryptionKind;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn zip_target() -> (tempfile::NamedTempFile, CrackTarget) {
        let file = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
        let mut writer = zip::ZipWriter::new(file.reopen().unwrap());
        writer
            .start_file("plain.txt", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"content").unwrap();
        writer.finish().unwrap();
        let target = CrackTarget::from_path(file.path());
        (file, target)
    }

    #[test]
    fn the_helper_target_is_recognised_as_a_zip() {
        let (_file, target) = zip_target();
        assert_eq!(target.kind, TargetKind::Zip);
    }

    #[test]
    fn a_zip_target_produces_a_zip_tester() {
        let (_file, target) = zip_target();
        let tester = DefaultTesterFactory::new().create(&target).unwrap();
        assert_eq!(tester.info().kind, TargetKind::Zip);
    }

    #[test]
    fn a_shadow_target_produces_a_shadow_tester() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let hash = pwhash::sha512_crypt::hash_with("$6$abcdefgh$", "secret").unwrap();
        writeln!(file, "root:{hash}:19000:0:99999:7:::").unwrap();
        file.flush().unwrap();

        let target = CrackTarget {
            kind: TargetKind::Shadow,
            path: file.path().to_path_buf(),
            display_name: "shadow".to_string(),
        };
        let tester = DefaultTesterFactory::new().create(&target).unwrap();
        assert_eq!(tester.info().kind, TargetKind::Shadow);
    }

    #[test]
    fn a_rar_target_produces_a_rar_tester() {
        let fixture = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/rar4_solid_encrypted.rar"
        );
        let target = CrackTarget::from_path(fixture);
        let tester = DefaultTesterFactory::new().create(&target).unwrap();
        assert_eq!(tester.info().kind, TargetKind::Rar);
        assert_eq!(tester.info().encryption, EncryptionKind::RarCrypto4);
    }

    #[test]
    fn a_rar_file_that_is_not_an_archive_is_reported_as_unavailable() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("fake.rar");
        std::fs::write(&path, b"not really a rar archive").unwrap();
        let target = CrackTarget::from_path(&path);
        match DefaultTesterFactory::new().create(&target) {
            Err(CrackFailure::BackendUnavailable { backend, .. }) => {
                assert_eq!(backend, Backend::Rar)
            }
            Err(other) => panic!("unexpected failure: {other:?}"),
            Ok(_) => panic!("expected a failure for a non-RAR file"),
        }
    }

    #[test]
    fn a_pdf_target_produces_a_pdf_tester() {
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/encrypted.pdf");
        let target = CrackTarget::from_path(fixture);
        let tester = DefaultTesterFactory::new().create(&target).unwrap();
        assert_eq!(tester.info().kind, TargetKind::Pdf);
        assert_eq!(tester.info().encryption, EncryptionKind::PdfStandard);
    }

    #[test]
    fn an_unsupported_target_names_itself_in_the_failure() {
        let target = CrackTarget {
            kind: TargetKind::Unsupported,
            path: "notes.txt".into(),
            display_name: "notes.txt".to_string(),
        };
        match DefaultTesterFactory::new().create(&target) {
            Err(CrackFailure::UnsupportedFormat { display_name }) => {
                assert_eq!(display_name, "notes.txt")
            }
            Err(other) => panic!("unexpected failure: {other:?}"),
            Ok(_) => panic!("expected a failure for an unsupported target"),
        }
    }

    #[test]
    fn a_missing_zip_file_surfaces_the_unreadable_failure() {
        let target = CrackTarget::from_path("no-such-archive.zip");
        assert!(matches!(
            DefaultTesterFactory::new().create(&target),
            Err(CrackFailure::FileUnreadable { .. })
        ));
    }
}
