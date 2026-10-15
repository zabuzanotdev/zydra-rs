//! Pilihan pengguna sebelum serangan dijalankan.
//!
//! Modul ini murni data dan logika: tidak menyentuh egui, tidak menyentuh mesin. Itu
//! disengaja, karena aturan "kapan tombol mulai boleh ditekan" adalah hal yang paling
//! mudah salah dan paling sulit dilihat lewat pemeriksaan mata.

use std::path::PathBuf;
use std::time::Instant;

use zydra_core::attack::AttackPlan;
use zydra_core::candidate::wordlist::WordlistSource;
use zydra_core::candidate::CandidateSource;
use zydra_core::model::charset::CharsetSet;
use zydra_core::model::target::CrackTarget;

/// Metode serangan yang dipilih.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Coba setiap baris wordlist.
    Dictionary,
    /// Coba setiap kombinasi karakter.
    BruteForce,
}

/// Kenapa sebuah rencana belum bisa dijalankan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanProblem {
    /// Belum ada berkas target.
    MissingTarget,
    /// Metode kamus dipilih tetapi belum ada wordlist.
    MissingWordlist,
    /// Tidak ada kelas karakter yang dipilih.
    NoCharset,
    /// Rentang panjang tidak masuk akal.
    BadRange,
}

/// Apa yang sudah dipilih pengguna.
#[derive(Debug, Clone)]
pub struct UiSettings {
    /// Berkas target.
    pub target: Option<CrackTarget>,
    /// Metode serangan.
    pub method: Method,
    /// Berkas wordlist.
    pub wordlist: Option<PathBuf>,
    /// Jumlah baris wordlist, dihitung sekali saat berkasnya dipilih.
    pub wordlist_lines: Option<u64>,
    /// Kelas karakter untuk brute force.
    pub charset: CharsetSet,
    /// Panjang minimum.
    pub min_length: u8,
    /// Panjang maksimum.
    pub max_length: u8,
    /// Apakah panel pilihan sedang terbuka.
    pub sidebar_open: bool,
    /// Waktu terakhir password disalin, untuk menampilkan tanda sebentar saja.
    pub copied_at: Option<Instant>,
}

impl Default for UiSettings {
    fn default() -> UiSettings {
        UiSettings {
            target: None,
            method: Method::Dictionary,
            wordlist: None,
            wordlist_lines: None,
            charset: CharsetSet::LOWERCASE | CharsetSet::DIGITS,
            min_length: 1,
            max_length: 4,
            sidebar_open: true,
            copied_at: None,
        }
    }
}

impl UiSettings {
    /// Menyusun rencana serangan, atau menjelaskan kenapa belum bisa.
    pub fn plan(&self) -> Result<AttackPlan, PlanProblem> {
        let target = self.target.as_ref().ok_or(PlanProblem::MissingTarget)?;
        let _ = target;
        match self.method {
            Method::Dictionary => {
                let wordlist = self.wordlist.clone().ok_or(PlanProblem::MissingWordlist)?;
                Ok(AttackPlan::Dictionary { wordlist })
            }
            Method::BruteForce => {
                if self.charset.is_empty() {
                    return Err(PlanProblem::NoCharset);
                }
                if self.min_length == 0 || self.max_length < self.min_length {
                    return Err(PlanProblem::BadRange);
                }
                Ok(AttackPlan::BruteForce {
                    charset: self.charset,
                    min_length: self.min_length,
                    max_length: self.max_length,
                })
            }
        }
    }

    /// Apakah serangan bisa dimulai sekarang.
    pub fn can_start(&self) -> bool {
        self.plan().is_ok()
    }

    /// Perkiraan jumlah kandidat, kalau bisa diketahui tanpa membaca ulang berkas.
    ///
    /// Untuk wordlist, angkanya datang dari hitungan yang dilakukan sekali saat berkas
    /// dipilih. Membaca ulang seluruh berkas setiap bingkai akan membuat antarmuka
    /// tersendat pada wordlist besar.
    pub fn estimate(&self) -> Option<u64> {
        match self.method {
            Method::Dictionary => self.wordlist_lines,
            Method::BruteForce => {
                if self.charset.is_empty() || self.max_length < self.min_length {
                    return None;
                }
                Some(
                    self.charset
                        .total_combinations(self.min_length, self.max_length),
                )
            }
        }
    }

    /// Menghitung baris wordlist dan menyimpannya.
    ///
    /// Mengembalikan `false` kalau berkasnya tidak bisa dibaca, supaya pemanggil bisa
    /// menampilkan sebabnya alih-alih diam-diam menyimpan angka nol.
    pub fn set_wordlist(&mut self, path: PathBuf) -> bool {
        match WordlistSource::open(&path) {
            Ok(source) => {
                self.wordlist_lines = Some(source.total());
                self.wordlist = Some(path);
                true
            }
            Err(_) => {
                self.wordlist_lines = None;
                false
            }
        }
    }

    /// Menandai bahwa password baru saja disalin.
    pub fn mark_copied(&mut self) {
        self.copied_at = Some(Instant::now());
    }

    /// Apakah tanda "tersalin" masih pantas ditampilkan.
    pub fn copied_recently(&self) -> bool {
        self.copied_at
            .map(|at| at.elapsed().as_secs_f32() < 2.0)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn target() -> CrackTarget {
        CrackTarget::from_path("vault.zip")
    }

    #[test]
    fn a_fresh_settings_cannot_start_because_no_target_is_chosen() {
        let settings = UiSettings::default();
        assert_eq!(settings.plan().unwrap_err(), PlanProblem::MissingTarget);
        assert!(!settings.can_start());
    }

    #[test]
    fn the_dictionary_method_needs_a_wordlist() {
        let settings = UiSettings {
            target: Some(target()),
            ..UiSettings::default()
        };
        assert_eq!(settings.plan().unwrap_err(), PlanProblem::MissingWordlist);
    }

    #[test]
    fn a_complete_dictionary_setup_produces_a_dictionary_plan() {
        let settings = UiSettings {
            target: Some(target()),
            wordlist: Some(PathBuf::from("words.txt")),
            ..UiSettings::default()
        };
        assert!(matches!(
            settings.plan().unwrap(),
            AttackPlan::Dictionary { .. }
        ));
        assert!(settings.can_start());
    }

    #[test]
    fn brute_force_needs_at_least_one_character_class() {
        let settings = UiSettings {
            target: Some(target()),
            method: Method::BruteForce,
            charset: CharsetSet::empty(),
            ..UiSettings::default()
        };
        assert_eq!(settings.plan().unwrap_err(), PlanProblem::NoCharset);
    }

    #[test]
    fn brute_force_rejects_an_inverted_length_range() {
        let settings = UiSettings {
            target: Some(target()),
            method: Method::BruteForce,
            min_length: 5,
            max_length: 2,
            ..UiSettings::default()
        };
        assert_eq!(settings.plan().unwrap_err(), PlanProblem::BadRange);
    }

    #[test]
    fn brute_force_rejects_a_zero_minimum() {
        let settings = UiSettings {
            target: Some(target()),
            method: Method::BruteForce,
            min_length: 0,
            max_length: 2,
            ..UiSettings::default()
        };
        assert_eq!(settings.plan().unwrap_err(), PlanProblem::BadRange);
    }

    #[test]
    fn a_complete_brute_force_setup_produces_a_brute_force_plan() {
        let settings = UiSettings {
            target: Some(target()),
            method: Method::BruteForce,
            charset: CharsetSet::DIGITS,
            min_length: 2,
            max_length: 3,
            ..UiSettings::default()
        };
        match settings.plan().unwrap() {
            AttackPlan::BruteForce {
                charset,
                min_length,
                max_length,
            } => {
                assert_eq!(charset, CharsetSet::DIGITS);
                assert_eq!(min_length, 2);
                assert_eq!(max_length, 3);
            }
            other => panic!("unexpected plan: {other:?}"),
        }
    }

    #[test]
    fn the_brute_force_estimate_matches_the_charset_arithmetic() {
        let settings = UiSettings {
            method: Method::BruteForce,
            charset: CharsetSet::DIGITS,
            min_length: 1,
            max_length: 2,
            ..UiSettings::default()
        };
        assert_eq!(settings.estimate(), Some(10 + 100));
    }

    #[test]
    fn the_dictionary_estimate_comes_from_the_counted_lines() {
        let mut settings = UiSettings::default();
        assert_eq!(settings.estimate(), None);

        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "alpha").unwrap();
        writeln!(file, "bravo").unwrap();
        file.flush().unwrap();

        assert!(settings.set_wordlist(file.path().to_path_buf()));
        assert_eq!(settings.estimate(), Some(2));
    }

    #[test]
    fn an_unreadable_wordlist_is_reported_and_clears_the_estimate() {
        let mut settings = UiSettings::default();
        assert!(!settings.set_wordlist(PathBuf::from("no-such-wordlist.txt")));
        assert_eq!(settings.estimate(), None);
        assert!(settings.wordlist.is_none());
    }

    #[test]
    fn the_estimate_is_absent_when_the_setup_is_incomplete() {
        let settings = UiSettings {
            method: Method::BruteForce,
            charset: CharsetSet::empty(),
            ..UiSettings::default()
        };
        assert_eq!(settings.estimate(), None);
    }

    #[test]
    fn the_copied_notice_expires() {
        let mut settings = UiSettings::default();
        assert!(!settings.copied_recently());
        settings.mark_copied();
        assert!(settings.copied_recently());
    }

    #[test]
    fn the_default_character_set_is_letters_and_digits() {
        let settings = UiSettings::default();
        assert_eq!(settings.charset, CharsetSet::LOWERCASE | CharsetSet::DIGITS);
    }
}
