//! Rencana serangan: memvalidasi pilihan pengguna dan menyiapkan sumber kandidat.
//!
//! Modul ini adalah satu-satunya tempat yang tahu cara menerjemahkan pilihan pengguna
//! menjadi [`CandidateSource`]. CLI dan GUI memakainya supaya aturan validasi tidak
//! ditulis dua kali dan tidak bisa saling berbeda.

use std::path::{Path, PathBuf};

use crate::candidate::bruteforce::BruteForceSource;
use crate::candidate::wordlist::WordlistSource;
use crate::candidate::CandidateSource;
use crate::constants::{DEFAULT_MAX_BRUTE_LENGTH, DEFAULT_MAX_COMBINATIONS};
use crate::error::CoreError;
use crate::model::charset::CharsetSet;
use crate::model::failure::CrackFailure;

/// Batas yang berlaku untuk sebuah serangan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackLimits {
    /// Jumlah kandidat maksimum sebelum serangan ditolak.
    pub max_combinations: u64,
    /// Panjang maksimum yang diizinkan untuk brute force.
    pub max_brute_length: u8,
}

impl Default for AttackLimits {
    fn default() -> AttackLimits {
        AttackLimits {
            max_combinations: DEFAULT_MAX_COMBINATIONS,
            max_brute_length: DEFAULT_MAX_BRUTE_LENGTH,
        }
    }
}

/// Apa yang akan dicoba.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttackPlan {
    /// Coba setiap baris sebuah wordlist.
    Dictionary {
        /// Berkas wordlist.
        wordlist: PathBuf,
    },
    /// Coba setiap kombinasi dari himpunan kelas karakter.
    BruteForce {
        /// Kelas karakter yang dipakai.
        charset: CharsetSet,
        /// Panjang minimum.
        min_length: u8,
        /// Panjang maksimum.
        max_length: u8,
    },
}

/// Rencana yang sudah divalidasi, beserta sumber kandidatnya.
pub struct PreparedAttack {
    /// Jumlah kandidat yang akan dicoba.
    pub total: u64,
    /// Sumber kandidat, siap diberikan ke [`crate::engine::crack::run`].
    pub source: Box<dyn CandidateSource>,
}

impl std::fmt::Debug for PreparedAttack {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedAttack")
            .field("total", &self.total)
            .finish_non_exhaustive()
    }
}

impl AttackPlan {
    /// Memvalidasi rencana lalu menyiapkan sumber kandidatnya.
    ///
    /// Validasi dan penyiapan digabung dalam satu langkah dengan dua alasan: wordlist
    /// hanya dihitung sekali, dan tidak ada jalan bagi pemanggil untuk menjalankan
    /// rencana yang belum divalidasi.
    pub fn prepare(&self, limits: &AttackLimits) -> Result<PreparedAttack, CrackFailure> {
        match self {
            AttackPlan::Dictionary { wordlist } => {
                let source =
                    WordlistSource::open(wordlist).map_err(|error| unreadable(wordlist, &error))?;
                let total = source.total();
                if total == 0 {
                    return Err(CrackFailure::EmptyWordlist);
                }
                if total > limits.max_combinations {
                    return Err(CrackFailure::TooManyCombinations {
                        total,
                        limit: limits.max_combinations,
                    });
                }
                Ok(PreparedAttack {
                    total,
                    source: Box::new(source),
                })
            }
            AttackPlan::BruteForce {
                charset,
                min_length,
                max_length,
            } => {
                if charset.is_empty() {
                    return Err(CrackFailure::NoCharsetSelected);
                }
                if *min_length == 0
                    || max_length < min_length
                    || *max_length > limits.max_brute_length
                {
                    return Err(CrackFailure::InvalidLengthRange {
                        min: *min_length,
                        max: *max_length,
                        allowed_max: limits.max_brute_length,
                    });
                }
                let total = charset.total_combinations(*min_length, *max_length);
                if total > limits.max_combinations {
                    return Err(CrackFailure::TooManyCombinations {
                        total,
                        limit: limits.max_combinations,
                    });
                }
                let source = BruteForceSource::new(*charset, *min_length, *max_length)?;
                Ok(PreparedAttack {
                    total,
                    source: Box::new(source),
                })
            }
        }
    }
}

/// Menerjemahkan kegagalan membaca berkas menjadi sebab yang bisa ditampilkan.
///
/// `reason` bersifat teknis dan ditujukan untuk catatan; lapisan antarmuka menampilkan
/// kalimat umum dari katalog terjemahan, bukan teks ini.
fn unreadable(path: &Path, error: &CoreError) -> CrackFailure {
    CrackFailure::FileUnreadable {
        reason: format!("{}: {error}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn wordlist(contents: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(contents).unwrap();
        file.flush().unwrap();
        file
    }

    fn plan_for(file: &tempfile::NamedTempFile) -> AttackPlan {
        AttackPlan::Dictionary {
            wordlist: file.path().to_path_buf(),
        }
    }

    #[test]
    fn a_dictionary_plan_counts_only_real_candidates() {
        let file = wordlist(b"alpha\n\nbravo\ncharlie\r\n");
        let prepared = plan_for(&file).prepare(&AttackLimits::default()).unwrap();
        assert_eq!(prepared.total, 3);
        assert_eq!(prepared.source.total(), 3);
    }

    #[test]
    fn a_dictionary_plan_on_a_missing_file_is_unreadable() {
        let plan = AttackPlan::Dictionary {
            wordlist: PathBuf::from("no-such-wordlist-here.txt"),
        };
        assert!(matches!(
            plan.prepare(&AttackLimits::default()),
            Err(CrackFailure::FileUnreadable { .. })
        ));
    }

    #[test]
    fn a_wordlist_without_candidates_is_rejected() {
        let file = wordlist(b"\n\n\n");
        assert_eq!(
            plan_for(&file)
                .prepare(&AttackLimits::default())
                .unwrap_err(),
            CrackFailure::EmptyWordlist
        );
    }

    #[test]
    fn a_wordlist_beyond_the_combination_limit_is_rejected() {
        let file = wordlist(b"one\ntwo\nthree\n");
        let limits = AttackLimits {
            max_combinations: 2,
            ..AttackLimits::default()
        };
        assert_eq!(
            plan_for(&file).prepare(&limits).unwrap_err(),
            CrackFailure::TooManyCombinations { total: 3, limit: 2 }
        );
    }

    #[test]
    fn a_brute_force_plan_reports_the_charset_arithmetic() {
        let plan = AttackPlan::BruteForce {
            charset: CharsetSet::LOWERCASE,
            min_length: 1,
            max_length: 2,
        };
        let prepared = plan.prepare(&AttackLimits::default()).unwrap();
        assert_eq!(prepared.total, 26 + 26 * 26);
        assert_eq!(prepared.source.total(), 26 + 26 * 26);
    }

    #[test]
    fn a_brute_force_plan_without_a_charset_is_rejected() {
        let plan = AttackPlan::BruteForce {
            charset: CharsetSet::empty(),
            min_length: 1,
            max_length: 2,
        };
        assert_eq!(
            plan.prepare(&AttackLimits::default()).unwrap_err(),
            CrackFailure::NoCharsetSelected
        );
    }

    #[test]
    fn a_zero_minimum_length_is_rejected() {
        let plan = AttackPlan::BruteForce {
            charset: CharsetSet::LOWERCASE,
            min_length: 0,
            max_length: 2,
        };
        assert!(matches!(
            plan.prepare(&AttackLimits::default()).unwrap_err(),
            CrackFailure::InvalidLengthRange { .. }
        ));
    }

    #[test]
    fn a_maximum_beyond_the_configured_limit_is_rejected_and_reports_the_limit() {
        let plan = AttackPlan::BruteForce {
            charset: CharsetSet::LOWERCASE,
            min_length: 1,
            max_length: 8,
        };
        let limits = AttackLimits {
            max_combinations: u64::MAX,
            max_brute_length: 6,
        };
        match plan.prepare(&limits).unwrap_err() {
            CrackFailure::InvalidLengthRange { allowed_max, .. } => assert_eq!(allowed_max, 6),
            other => panic!("unexpected failure: {other:?}"),
        }
    }

    #[test]
    fn a_brute_force_plan_beyond_the_combination_limit_is_rejected() {
        let plan = AttackPlan::BruteForce {
            charset: CharsetSet::LOWERCASE | CharsetSet::UPPERCASE | CharsetSet::DIGITS,
            min_length: 1,
            max_length: 6,
        };
        let limits = AttackLimits {
            max_combinations: 1_000,
            max_brute_length: 6,
        };
        assert!(matches!(
            plan.prepare(&limits).unwrap_err(),
            CrackFailure::TooManyCombinations { limit: 1_000, .. }
        ));
    }

    #[test]
    fn a_prepared_dictionary_source_yields_the_wordlist_in_order() {
        let file = wordlist(b"alpha\nbravo\n");
        let mut prepared = plan_for(&file).prepare(&AttackLimits::default()).unwrap();
        assert_eq!(prepared.source.next_candidate().as_deref(), Some("alpha"));
        assert_eq!(prepared.source.next_candidate().as_deref(), Some("bravo"));
        assert_eq!(prepared.source.next_candidate(), None);
    }
}
