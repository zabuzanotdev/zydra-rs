//! Sumber kandidat yang menerapkan aturan mangling pada sumber lain.
//!
//! Sumber ini membungkus sumber dasar apa pun - wordlist, brute force, atau mask - dan
//! mengeluarkan variasi dari setiap kandidat dasarnya. Karena aturan menghasilkan jumlah
//! keluaran yang tetap untuk setiap kata, posisi dalam ruang gabungan bisa dihitung
//! langsung: posisi ke-N selalu jatuh pada kata dasar ke-`N / keluaran_per_kata` dengan
//! offset `N % keluaran_per_kata`.
//!
//! Itu bukan kerapian belaka. Tanpa sifat tersebut, posisi yang tersimpan untuk melanjutkan
//! serangan yang terputus tidak bisa diterjemahkan kembali menjadi kata dasar dan aturan.

use crate::candidate::CandidateSource;
use crate::model::rules::RuleSet;

/// Menerapkan aturan mangling pada setiap kandidat sebuah sumber dasar.
pub struct RulesSource {
    base: Box<dyn CandidateSource>,
    rules: RuleSet,
    expansion: Vec<String>,
    expansion_index: usize,
    emitted: u64,
    total: u64,
}

impl std::fmt::Debug for RulesSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RulesSource")
            .field("rules", &self.rules)
            .field("total", &self.total)
            .finish_non_exhaustive()
    }
}

impl RulesSource {
    /// Membungkus sebuah sumber dasar.
    pub fn new(base: Box<dyn CandidateSource>, rules: RuleSet) -> RulesSource {
        let per_word = rules.outputs_per_word();
        let total = base.total().saturating_mul(per_word);
        RulesSource {
            base,
            rules,
            expansion: Vec::new(),
            expansion_index: 0,
            emitted: 0,
            total,
        }
    }

    /// Aturan yang sedang dipakai.
    pub fn rules(&self) -> &RuleSet {
        &self.rules
    }

    /// Mengambil kata dasar berikutnya dan mengembangkannya.
    ///
    /// `false` berarti sumber dasarnya sudah habis.
    fn fill_next(&mut self) -> bool {
        match self.base.next_candidate() {
            Some(word) => {
                self.expansion = self.rules.apply(&word);
                self.expansion_index = 0;
                true
            }
            None => {
                self.expansion.clear();
                self.expansion_index = 0;
                false
            }
        }
    }
}

impl CandidateSource for RulesSource {
    fn next_candidate(&mut self) -> Option<String> {
        loop {
            if self.expansion_index < self.expansion.len() {
                let candidate = self.expansion[self.expansion_index].clone();
                self.expansion_index += 1;
                self.emitted += 1;
                return Some(candidate);
            }
            if !self.fill_next() {
                return None;
            }
        }
    }

    fn total(&self) -> u64 {
        self.total
    }

    fn position(&self) -> u64 {
        self.emitted
    }

    fn skip(&mut self, count: u64) {
        if count == 0 {
            return;
        }
        let target = self.emitted.saturating_add(count);
        let per_word = self.rules.outputs_per_word();
        let word_index = target / per_word;
        let offset = (target % per_word) as usize;

        // Lompati kata dasar yang tidak perlu dikembangkan sama sekali.
        let consumed = self.base.position();
        if word_index > consumed {
            self.base.skip(word_index - consumed);
        }

        self.expansion.clear();
        self.expansion_index = 0;
        self.emitted = target;

        if offset > 0 {
            // Kata dasar berikutnya harus dikembangkan untuk membuang sebagian awalnya.
            if self.fill_next() {
                self.expansion_index = offset.min(self.expansion.len());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::bruteforce::BruteForceSource;
    use crate::candidate::wordlist::WordlistSource;
    use crate::model::charset::CharsetSet;
    use std::io::Write;

    fn wordlist(words: &[&str]) -> WordlistSource {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        for word in words {
            writeln!(file, "{word}").unwrap();
        }
        file.flush().unwrap();
        // Berkas harus tetap hidup selama sumbernya dipakai, jadi isinya disalin ke
        // berkas baru yang tidak dihapus saat fungsi ini kembali.
        let path = file.into_temp_path().keep().unwrap();
        WordlistSource::open(&path).unwrap()
    }

    fn drain(source: &mut RulesSource) -> Vec<String> {
        let mut out = Vec::new();
        while let Some(candidate) = source.next_candidate() {
            out.push(candidate);
        }
        out
    }

    #[test]
    fn an_empty_rule_set_passes_the_base_candidates_through() {
        let mut source =
            RulesSource::new(Box::new(wordlist(&["alpha", "bravo"])), RuleSet::empty());
        assert_eq!(source.total(), 2);
        assert_eq!(drain(&mut source), vec!["alpha", "bravo"]);
    }

    #[test]
    fn rules_are_applied_to_every_base_candidate() {
        let mut source = RulesSource::new(
            Box::new(wordlist(&["alpha", "bravo"])),
            RuleSet::parse("capitalize").unwrap(),
        );
        assert_eq!(source.total(), 2);
        assert_eq!(drain(&mut source), vec!["Alpha", "Bravo"]);
    }

    #[test]
    fn a_rule_with_several_outputs_multiplies_the_total() {
        let mut source = RulesSource::new(
            Box::new(wordlist(&["a", "b"])),
            RuleSet::parse("append-digits:2").unwrap(),
        );
        assert_eq!(source.total(), 6);
        assert_eq!(drain(&mut source), vec!["a0", "a1", "a2", "b0", "b1", "b2"]);
    }

    #[test]
    fn the_total_matches_the_number_of_candidates_yielded() {
        let mut source = RulesSource::new(
            Box::new(wordlist(&["one", "two", "three"])),
            RuleSet::parse("none,leet,append-symbols").unwrap(),
        );
        let total = source.total();
        assert_eq!(drain(&mut source).len() as u64, total);
    }

    #[test]
    fn skip_lands_on_the_next_candidate() {
        // Urutannya a0, a1, a2, b0, b1, b2, jadi posisi ke-4 adalah b1.
        let mut source = RulesSource::new(
            Box::new(wordlist(&["a", "b"])),
            RuleSet::parse("append-digits:2").unwrap(),
        );
        source.skip(4);
        assert_eq!(source.position(), 4);
        assert_eq!(source.next_candidate().as_deref(), Some("b1"));
    }

    #[test]
    fn skip_inside_a_word_does_not_advance_the_base_word_twice() {
        // Offset yang tidak nol membuat kata dasar berikutnya harus dikembangkan lebih
        // dulu. Kalau sumber dasarnya ikut dimajukan dua kali, hasilnya akan melompat.
        let mut source = RulesSource::new(
            Box::new(wordlist(&["a", "b", "c"])),
            RuleSet::parse("append-digits:2").unwrap(),
        );
        source.skip(1);
        assert_eq!(source.next_candidate().as_deref(), Some("a1"));
        assert_eq!(source.next_candidate().as_deref(), Some("a2"));
        assert_eq!(source.next_candidate().as_deref(), Some("b0"));
    }

    #[test]
    fn skipping_a_whole_word_boundary_lands_correctly() {
        let mut source = RulesSource::new(
            Box::new(wordlist(&["a", "b", "c"])),
            RuleSet::parse("append-digits:2").unwrap(),
        );
        source.skip(3);
        assert_eq!(source.next_candidate().as_deref(), Some("b0"));
    }

    #[test]
    fn skipping_zero_changes_nothing() {
        let mut source = RulesSource::new(
            Box::new(wordlist(&["a"])),
            RuleSet::parse("append-digits:2").unwrap(),
        );
        source.skip(0);
        assert_eq!(source.next_candidate().as_deref(), Some("a0"));
    }

    #[test]
    fn skipping_the_whole_space_yields_nothing() {
        let mut source = RulesSource::new(
            Box::new(wordlist(&["a", "b"])),
            RuleSet::parse("append-digits:2").unwrap(),
        );
        source.skip(source.total());
        assert_eq!(source.next_candidate(), None);
    }

    #[test]
    fn skipping_past_the_end_yields_nothing_and_does_not_panic() {
        let mut source = RulesSource::new(
            Box::new(wordlist(&["a"])),
            RuleSet::parse("append-digits:2").unwrap(),
        );
        source.skip(100_000);
        assert_eq!(source.next_candidate(), None);
    }

    #[test]
    fn exhaustion_is_permanent() {
        let mut source = RulesSource::new(
            Box::new(wordlist(&["a"])),
            RuleSet::parse("capitalize").unwrap(),
        );
        assert_eq!(drain(&mut source).len(), 1);
        for _ in 0..5 {
            assert_eq!(source.next_candidate(), None);
        }
    }

    #[test]
    fn skip_and_sequential_walks_agree() {
        // Perbandingan langsung antara dua cara mencapai posisi yang sama. Kalau `skip`
        // dan penelusuran biasa berbeda pendapat, melanjutkan serangan yang terputus akan
        // melewatkan kandidat atau mengulanginya.
        let rules = RuleSet::parse("none,capitalize,append-digits:1").unwrap();
        let per_word = rules.outputs_per_word();
        let words = ["alpha", "bravo", "charlie"];

        for skip_count in 0..(words.len() as u64 * per_word) {
            let mut sequential = RulesSource::new(Box::new(wordlist(&words)), rules.clone());
            let mut expected = Vec::new();
            for _ in 0..skip_count {
                expected.push(sequential.next_candidate());
            }
            let next_expected = sequential.next_candidate();

            let mut skipped = RulesSource::new(Box::new(wordlist(&words)), rules.clone());
            skipped.skip(skip_count);
            assert_eq!(
                skipped.next_candidate(),
                next_expected,
                "skip({skip_count}) disagreed with walking there"
            );
        }
    }

    #[test]
    fn wrapping_a_brute_force_source_works_too() {
        let base = BruteForceSource::new(CharsetSet::DIGITS, 1, 2).unwrap();
        let mut source =
            RulesSource::new(Box::new(base), RuleSet::parse("prepend-digits:0").unwrap());
        // Aturan itu menghasilkan satu keluaran per kata, jadi totalnya sama dengan sumber
        // dasarnya: 10 satu-digit ditambah 100 dua-digit.
        assert_eq!(source.total(), 10 + 100);
        assert_eq!(source.next_candidate().as_deref(), Some("00"));
    }
}
