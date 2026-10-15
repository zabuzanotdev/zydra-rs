//! Sumber kandidat berbasis pola mask.
//!
//! Berbeda dari brute force biasa yang memakai satu alfabet untuk semua posisi, di sini
//! setiap posisi punya alfabetnya sendiri. Odometer-nya sama, basisnya yang berbeda per
//! digit.
//!
//! Dua hal yang dipelajari dari kesalahan sebelumnya dan diterapkan di sini sejak awal:
//! kehabisan harus bersifat permanen, dan `skip` harus melompat langsung alih-alih
//! menghasilkan lalu membuang kandidat satu per satu.

use crate::candidate::CandidateSource;
use crate::model::failure::CrackFailure;
use crate::model::mask::MaskPattern;

/// Menghasilkan setiap kombinasi yang diizinkan sebuah pola mask.
#[derive(Debug)]
pub struct MaskSource {
    positions: Vec<Vec<char>>,
    indices: Vec<usize>,
    started: bool,
    done: bool,
    emitted: u64,
    total: u64,
}

impl MaskSource {
    /// Membuat sumber dari pola yang sudah diurai.
    pub fn new(pattern: &MaskPattern) -> Result<MaskSource, CrackFailure> {
        if pattern.positions().is_empty() {
            return Err(CrackFailure::NoCandidate);
        }
        Ok(MaskSource {
            positions: pattern.positions().to_vec(),
            indices: Vec::new(),
            started: false,
            done: false,
            emitted: 0,
            total: pattern.total(),
        })
    }

    /// Memajukan odometer satu langkah. `false` berarti seluruh ruang sudah habis.
    fn advance(&mut self) -> bool {
        let mut position = self.indices.len();
        loop {
            if position == 0 {
                return false;
            }
            position -= 1;
            self.indices[position] += 1;
            if self.indices[position] < self.positions[position].len() {
                return true;
            }
            self.indices[position] = 0;
        }
    }

    /// Menempatkan odometer tepat sebelum kandidat ke-`index`.
    ///
    /// `next_candidate` selalu memajukan lebih dulu, jadi menempatkannya *pada* `index`
    /// akan menghasilkan kandidat berikutnya. Sama seperti sumber brute force, dan dengan
    /// alasan yang sama.
    fn seek(&mut self, index: u64) {
        if index == 0 {
            self.started = false;
            self.done = false;
            self.emitted = 0;
            self.indices.clear();
            return;
        }
        let target = index.min(self.total);
        if target >= self.total {
            self.done = true;
            self.started = true;
            self.emitted = self.total;
            self.indices.clear();
            return;
        }

        // Representasi bilangan campuran: posisi terakhir adalah digit paling cepat
        // berubah, seperti odometer.
        let mut value = target - 1;
        let mut digits = vec![0usize; self.positions.len()];
        let mut position = self.positions.len();
        while position > 0 {
            position -= 1;
            let base = self.positions[position].len() as u64;
            digits[position] = (value % base) as usize;
            value /= base;
        }

        self.indices = digits;
        self.started = true;
        self.done = false;
        self.emitted = target;
    }
}

impl CandidateSource for MaskSource {
    fn next_candidate(&mut self) -> Option<String> {
        if self.done {
            return None;
        }
        if self.started {
            if !self.advance() {
                self.done = true;
                return None;
            }
        } else {
            self.started = true;
            self.indices = vec![0; self.positions.len()];
        }
        self.emitted += 1;
        Some(
            self.indices
                .iter()
                .enumerate()
                .map(|(position, &choice)| self.positions[position][choice])
                .collect(),
        )
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
        self.seek(target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(pattern: &str) -> MaskSource {
        MaskSource::new(&MaskPattern::parse(pattern).unwrap()).unwrap()
    }

    fn drain(source: &mut MaskSource) -> Vec<String> {
        let mut out = Vec::new();
        while let Some(candidate) = source.next_candidate() {
            out.push(candidate);
        }
        out
    }

    #[test]
    fn a_pattern_of_literals_yields_exactly_that_string() {
        assert_eq!(drain(&mut source("abc")), vec!["abc"]);
    }

    #[test]
    fn a_single_class_walks_its_alphabet() {
        assert_eq!(
            drain(&mut source("?d")),
            vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
        );
    }

    #[test]
    fn two_classes_advance_the_last_position_fastest() {
        let candidates = drain(&mut source("?d?d"));
        assert_eq!(candidates.len(), 100);
        assert_eq!(candidates[0], "00");
        assert_eq!(candidates[1], "01");
        assert_eq!(candidates[10], "10");
        assert_eq!(candidates[99], "99");
    }

    #[test]
    fn literals_are_kept_in_place_between_classes() {
        let candidates = drain(&mut source("a?d"));
        assert_eq!(candidates[0], "a0");
        assert_eq!(candidates[9], "a9");
    }

    #[test]
    fn the_total_matches_the_number_of_candidates_yielded() {
        let mut mask = source("?l?d");
        let total = mask.total();
        assert_eq!(total, 26 * 10);
        assert_eq!(drain(&mut mask).len() as u64, total);
    }

    #[test]
    fn position_counts_yielded_candidates() {
        let mut mask = source("?d?d");
        assert_eq!(mask.position(), 0);
        mask.next_candidate();
        mask.next_candidate();
        assert_eq!(mask.position(), 2);
    }

    #[test]
    fn skip_lands_on_the_next_candidate() {
        let mut mask = source("?d?d");
        mask.skip(12);
        assert_eq!(mask.position(), 12);
        assert_eq!(mask.next_candidate().as_deref(), Some("12"));
    }

    #[test]
    fn skipping_the_whole_space_yields_nothing() {
        let mut mask = source("?d?d");
        mask.skip(mask.total());
        assert_eq!(mask.next_candidate(), None);
    }

    #[test]
    fn skipping_past_the_end_yields_nothing_and_does_not_panic() {
        let mut mask = source("?d?d");
        mask.skip(1_000_000);
        assert_eq!(mask.next_candidate(), None);
    }

    #[test]
    fn exhaustion_is_permanent() {
        // Regresi yang sama dengan sumber brute force: sumber yang memulai ulang setelah
        // habis membuat mesin paralel menelusuri ruang yang sama tanpa henti.
        let mut mask = source("?d");
        assert_eq!(drain(&mut mask).len(), 10);
        for _ in 0..5 {
            assert_eq!(mask.next_candidate(), None);
        }
    }

    #[test]
    fn skipping_zero_changes_nothing() {
        let mut mask = source("?d?d");
        mask.skip(0);
        assert_eq!(mask.next_candidate().as_deref(), Some("00"));
    }

    #[test]
    fn every_candidate_has_the_length_of_the_pattern() {
        let mut mask = source("?d?l?u");
        while let Some(candidate) = mask.next_candidate() {
            assert_eq!(candidate.chars().count(), 3);
        }
    }

    #[test]
    fn candidates_are_unique_across_a_full_run() {
        // Odometer dengan basis berbeda per posisi mudah salah di perbatasan antar posisi,
        // dan kesalahannya muncul sebagai kandidat yang berulang.
        let mut mask = source("?d?d?d");
        let mut seen = std::collections::BTreeSet::new();
        while let Some(candidate) = mask.next_candidate() {
            assert!(seen.insert(candidate.clone()), "duplicate: {candidate}");
        }
        assert_eq!(seen.len(), 1_000);
    }
}
