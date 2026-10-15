//! Penguji untuk berkas `shadow` Unix.
//!
//! Verifikasi memakai crate `pwhash`, yang mengimplementasikan `crypt(3)` dalam Rust murni
//! dan mengenali sendiri skema dari bentuk hash-nya (`$1$`, `$5$`, `$6$`, `$2y$`, DES).
//!
//! Ini perbaikan nyata atas versi Python: skrip aslinya memakai modul `crypt` yang hanya
//! ada di Unix, sehingga fitur ini mati total di Windows. Di sini ia jalan di ketiga sistem.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::model::failure::{Backend, CrackFailure};
use crate::model::target::TargetKind;
use crate::tester::{EncryptionKind, MatchOutcome, PasswordTester, TesterInfo, UnixCryptScheme};

/// Satu baris `shadow` yang bisa dipecahkan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowEntry {
    /// Nama pengguna.
    pub user: String,
    /// Hash tersimpan, apa adanya, termasuk parameter dan garamnya.
    pub hash: String,
}

/// Menguji kandidat password terhadap seluruh entri sebuah berkas `shadow`.
pub struct ShadowTester {
    entries: Vec<ShadowEntry>,
    info: TesterInfo,
}

impl ShadowTester {
    /// Membaca dan mengurai berkas `shadow`.
    ///
    /// Baris yang tidak bisa diurai dan akun yang terkunci dilewati tanpa membuat seluruh
    /// berkas gagal.
    pub fn open(path: &Path) -> Result<ShadowTester, CrackFailure> {
        let file = File::open(path).map_err(|error| CrackFailure::FileUnreadable {
            reason: format!("{}: {error}", path.display()),
        })?;

        let entries: Vec<ShadowEntry> = BufReader::new(file)
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| parse_line(&line))
            .collect();

        if entries.is_empty() {
            return Err(CrackFailure::BackendUnavailable {
                backend: Backend::Shadow,
                reason: "no-usable-entries".to_string(),
            });
        }

        let encryption = EncryptionKind::UnixCrypt(scheme_of(&entries[0].hash));
        Ok(ShadowTester {
            info: TesterInfo {
                kind: TargetKind::Shadow,
                encryption,
                entries: entries.len(),
            },
            entries,
        })
    }

    /// Entri yang berhasil diurai.
    pub fn entries(&self) -> &[ShadowEntry] {
        &self.entries
    }
}

/// Mengurai satu baris `shadow`. `None` berarti baris itu tidak bisa dipakai.
fn parse_line(line: &str) -> Option<ShadowEntry> {
    let mut parts = line.split(':');
    let user = parts.next()?.trim();
    let hash = parts.next()?.trim();

    if user.is_empty() {
        return None;
    }
    // Akun terkunci atau tanpa password tidak bisa dan tidak perlu dipecahkan.
    if hash.starts_with('*') || hash.starts_with('!') {
        return None;
    }
    // Field seperti nomor hari atau angka lain bukan hash. Menerimanya hanya menambah
    // entri yang pasti tidak akan pernah cocok, dan membuat log jadi berisik.
    if !looks_like_crypt_hash(hash) {
        return None;
    }

    Some(ShadowEntry {
        user: user.to_string(),
        hash: hash.to_string(),
    })
}

/// Apakah field ini berbentuk hash `crypt(3)`: modular (`$id$salt$digest`) atau DES
/// klasik (13 karakter dari alfabet `./0-9A-Za-z`).
fn looks_like_crypt_hash(hash: &str) -> bool {
    if hash.starts_with('$') {
        hash.matches('$').count() >= 3
    } else {
        hash.len() == 13
            && hash
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '/')
    }
}

/// Mengenali skema `crypt(3)` dari bentuk hash-nya.
fn scheme_of(hash: &str) -> UnixCryptScheme {
    if hash.starts_with("$1$") {
        UnixCryptScheme::Md5
    } else if hash.starts_with("$5$") {
        UnixCryptScheme::Sha256
    } else if hash.starts_with("$6$") {
        UnixCryptScheme::Sha512
    } else if hash.starts_with("$2a$") || hash.starts_with("$2b$") || hash.starts_with("$2y$") {
        UnixCryptScheme::Bcrypt
    } else if hash.len() == 13 {
        UnixCryptScheme::Des
    } else {
        UnixCryptScheme::Other(hash.chars().take(3).collect())
    }
}

impl PasswordTester for ShadowTester {
    fn try_password(&mut self, password: &str) -> MatchOutcome {
        for entry in &self.entries {
            if pwhash::unix::verify(password, &entry.hash) {
                return MatchOutcome::Matched {
                    detail: Some(entry.user.clone()),
                };
            }
        }
        MatchOutcome::NoMatch
    }

    fn info(&self) -> &TesterInfo {
        &self.info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const SECRET: &str = "correct horse battery staple";

    /// Membuat berkas `shadow` sintetis berisi hash asli untuk `SECRET`.
    fn shadow_file(entries: &[(&str, String)]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        for (user, hash) in entries {
            writeln!(file, "{user}:{hash}:19000:0:99999:7:::").unwrap();
        }
        file.flush().unwrap();
        file
    }

    fn sha512_hash() -> String {
        pwhash::sha512_crypt::hash_with("$6$abcdefgh$", SECRET).unwrap()
    }

    /// Membuat hash md5crypt untuk diuji, bukan untuk disarankan.
    ///
    /// `pwhash` menandai fungsi ini deprecated karena md5crypt tidak layak dipakai untuk
    /// password baru. Kita tetap membutuhkannya di sini karena berkas `shadow` lama di
    /// dunia nyata memuat hash md5crypt, dan pemulihannya justru harus bisa menanganinya.
    #[allow(deprecated)]
    fn md5_hash() -> String {
        pwhash::md5_crypt::hash_with("$1$abcdefgh$", SECRET).unwrap()
    }

    #[test]
    fn a_password_matching_the_stored_hash_is_found_and_names_the_user() {
        let file = shadow_file(&[("root", sha512_hash())]);
        let mut tester = ShadowTester::open(file.path()).unwrap();
        assert_eq!(
            tester.try_password(SECRET),
            MatchOutcome::Matched {
                detail: Some("root".to_string())
            }
        );
    }

    #[test]
    fn a_wrong_password_is_rejected() {
        let file = shadow_file(&[("root", sha512_hash())]);
        let mut tester = ShadowTester::open(file.path()).unwrap();
        assert_eq!(tester.try_password("wrong"), MatchOutcome::NoMatch);
    }

    #[test]
    fn md5_crypt_hashes_are_supported() {
        let file = shadow_file(&[("alice", md5_hash())]);
        let mut tester = ShadowTester::open(file.path()).unwrap();
        assert_eq!(
            tester.try_password(SECRET),
            MatchOutcome::Matched {
                detail: Some("alice".to_string())
            }
        );
    }

    #[test]
    fn every_entry_is_tried_not_only_the_first() {
        let file = shadow_file(&[("root", sha512_hash()), ("alice", sha512_hash())]);
        let mut tester = ShadowTester::open(file.path()).unwrap();
        assert_eq!(tester.entries().len(), 2);
        assert!(matches!(
            tester.try_password(SECRET),
            MatchOutcome::Matched { .. }
        ));
    }

    #[test]
    fn locked_accounts_are_skipped() {
        let file = shadow_file(&[
            ("root", "*".to_string()),
            ("daemon", "!".to_string()),
            ("bin", "!!".to_string()),
            ("alice", sha512_hash()),
        ]);
        let tester = ShadowTester::open(file.path()).unwrap();
        assert_eq!(tester.entries().len(), 1);
        assert_eq!(tester.entries()[0].user, "alice");
    }

    #[test]
    fn malformed_lines_are_skipped_without_failing_the_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "this line has no colon").unwrap();
        writeln!(file, ":{}:19000:0:99999:7:::", sha512_hash()).unwrap();
        writeln!(file, "empty:19000:0:99999:7:::").unwrap();
        writeln!(file, "alice:{}:19000:0:99999:7:::", sha512_hash()).unwrap();
        file.flush().unwrap();

        let tester = ShadowTester::open(file.path()).unwrap();
        assert_eq!(tester.entries().len(), 1);
        assert_eq!(tester.entries()[0].user, "alice");
    }

    #[test]
    fn a_file_without_usable_entries_is_reported_as_unavailable() {
        let file = shadow_file(&[("root", "*".to_string())]);
        assert!(matches!(
            ShadowTester::open(file.path()),
            Err(CrackFailure::BackendUnavailable {
                backend: Backend::Shadow,
                ..
            })
        ));
    }

    #[test]
    fn a_missing_file_is_reported_as_unreadable() {
        assert!(matches!(
            ShadowTester::open(Path::new("no-such-shadow-file")),
            Err(CrackFailure::FileUnreadable { .. })
        ));
    }

    #[test]
    fn the_crypt_scheme_is_recognised_from_the_hash() {
        assert_eq!(scheme_of(&sha512_hash()), UnixCryptScheme::Sha512);
        assert_eq!(scheme_of(&md5_hash()), UnixCryptScheme::Md5);
        assert_eq!(scheme_of("$5$salt$abc"), UnixCryptScheme::Sha256);
        assert_eq!(scheme_of("$2y$05$abc"), UnixCryptScheme::Bcrypt);
        assert_eq!(scheme_of("abcdefghijklm"), UnixCryptScheme::Des);
    }

    #[test]
    fn a_numeric_field_is_not_mistaken_for_a_hash() {
        assert!(!looks_like_crypt_hash("19000"));
        assert!(!looks_like_crypt_hash("0"));
        assert!(!looks_like_crypt_hash(""));
        assert!(!looks_like_crypt_hash("$6$abcdefgh"));
    }

    #[test]
    fn real_hashes_are_recognised() {
        assert!(looks_like_crypt_hash(&sha512_hash()));
        assert!(looks_like_crypt_hash(&md5_hash()));
        assert!(looks_like_crypt_hash("abcdefghijklm"));
        assert!(!looks_like_crypt_hash("abcdefghijkl")); // 12 karakter, terlalu pendek
        assert!(!looks_like_crypt_hash("abcdefghijklmn")); // 14 karakter, terlalu panjang
        assert!(!looks_like_crypt_hash("abcdefghijkl!"));
    }

    #[test]
    fn a_shadow_file_with_only_numeric_fields_is_reported_as_unavailable() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "empty:19000:0:99999:7:::").unwrap();
        writeln!(file, "root:0:0:99999:7:::").unwrap();
        file.flush().unwrap();
        assert!(matches!(
            ShadowTester::open(file.path()),
            Err(CrackFailure::BackendUnavailable { .. })
        ));
    }

    #[test]
    fn the_reported_scheme_matches_the_file() {
        let file = shadow_file(&[("root", sha512_hash())]);
        let tester = ShadowTester::open(file.path()).unwrap();
        assert_eq!(
            tester.info().encryption,
            EncryptionKind::UnixCrypt(UnixCryptScheme::Sha512)
        );
    }
}
