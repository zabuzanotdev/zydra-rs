//! Penguji untuk dokumen PDF.
//!
//! Verifikasi memakai `lopdf`, yang mengimplementasikan standard security handler PDF
//! dalam Rust murni. Versi Python memanggil program luar `qpdf` untuk pekerjaan yang sama,
//! dan program itu harus dipasang lebih dulu oleh pengguna.
//!
//! # Kenapa memuat ulang dokumen untuk setiap kandidat
//!
//! PDF tidak menyediakan cara memeriksa password tanpa memuat dokumennya, karena kunci
//! enkripsinya diturunkan dari password beserta garam dan jumlah iterasi yang tersimpan di
//! dalam berkas. Tidak ada jalan pintas yang lebih murah.

use std::path::{Path, PathBuf};

use lopdf::Document;

use crate::model::failure::{Backend, CrackFailure};
use crate::model::target::TargetKind;
use crate::tester::{EncryptionKind, MatchOutcome, PasswordTester, TesterInfo};

/// Menguji kandidat password terhadap sebuah dokumen PDF.
pub struct PdfTester {
    path: PathBuf,
    info: TesterInfo,
}

impl PdfTester {
    /// Membuka dokumen untuk mengenali keadaannya.
    ///
    /// Pemanggilan ini hanya membaca dan tidak memutuskan apa pun soal password; hasilnya
    /// dipakai untuk mengisi keterangan yang ditampilkan ke pengguna.
    pub fn open(path: &Path) -> Result<PdfTester, CrackFailure> {
        let document = Document::load(path).map_err(|error| CrackFailure::BackendUnavailable {
            backend: Backend::Pdf,
            reason: format!("{}: {error}", path.display()),
        })?;

        // Dokumen yang masih membawa kamus /Encrypt berarti benar-benar terkunci.
        let encryption = if document.is_encrypted() {
            EncryptionKind::PdfStandard
        } else {
            EncryptionKind::None
        };

        Ok(PdfTester {
            path: path.to_path_buf(),
            info: TesterInfo {
                kind: TargetKind::Pdf,
                encryption,
                entries: 1,
            },
        })
    }
}

impl PasswordTester for PdfTester {
    fn try_password(&mut self, password: &str) -> MatchOutcome {
        let document = match Document::load_with_password(&self.path, password) {
            Ok(document) => document,
            Err(_) => return MatchOutcome::NoMatch,
        };

        // Dokumen yang berhasil didekripsi tidak lagi membawa kamus /Encrypt. Memeriksa
        // itu lebih dapat dipercaya daripada mengandalkan keberhasilan pemuatan, karena
        // PDF tanpa enkripsi juga berhasil dimuat dengan password apa pun.
        if document.is_encrypted() {
            return MatchOutcome::NoMatch;
        }

        MatchOutcome::Matched { detail: None }
    }

    fn info(&self) -> &TesterInfo {
        &self.info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Fixture yang di-commit ke repositori.
    ///
    /// Dibuat sekali dengan PyMuPDF memakai AES-256, dan passwordnya sengaja ditulis di
    /// sini supaya tes bisa membuktikan keduanya: yang benar diterima, yang salah ditolak.
    /// Berkasnya 1,5 KB, jadi tidak ada alasan untuk menghasilkannya saat tes berjalan dan
    /// menambah ketergantungan pada alat luar.
    const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/encrypted.pdf");
    const SECRET: &str = "kr4k3n_1987";

    #[test]
    fn the_correct_password_is_found() {
        let mut tester = PdfTester::open(Path::new(FIXTURE)).unwrap();
        assert_eq!(
            tester.try_password(SECRET),
            MatchOutcome::Matched { detail: None }
        );
    }

    #[test]
    fn a_wrong_password_is_rejected() {
        let mut tester = PdfTester::open(Path::new(FIXTURE)).unwrap();
        assert_eq!(tester.try_password("wrong"), MatchOutcome::NoMatch);
    }

    #[test]
    fn an_empty_password_is_rejected_on_an_encrypted_document() {
        let mut tester = PdfTester::open(Path::new(FIXTURE)).unwrap();
        assert_eq!(tester.try_password(""), MatchOutcome::NoMatch);
    }

    #[test]
    fn a_password_sharing_a_prefix_is_rejected() {
        let mut tester = PdfTester::open(Path::new(FIXTURE)).unwrap();
        assert_eq!(tester.try_password("kr4k3n_198"), MatchOutcome::NoMatch);
    }

    #[test]
    fn standard_encryption_is_detected() {
        let tester = PdfTester::open(Path::new(FIXTURE)).unwrap();
        assert_eq!(tester.info().kind, TargetKind::Pdf);
        assert_eq!(tester.info().encryption, EncryptionKind::PdfStandard);
    }

    #[test]
    fn a_file_that_is_not_a_pdf_is_reported_as_unavailable() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"this is definitely not a PDF document")
            .unwrap();
        file.flush().unwrap();
        assert!(matches!(
            PdfTester::open(file.path()),
            Err(CrackFailure::BackendUnavailable { .. })
        ));
    }

    #[test]
    fn a_missing_file_is_reported_as_unavailable() {
        assert!(matches!(
            PdfTester::open(Path::new("no-such-document.pdf")),
            Err(CrackFailure::BackendUnavailable { .. })
        ));
    }
}
