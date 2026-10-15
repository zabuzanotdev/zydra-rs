//! Sumber kandidat hasil penelusuran seluruh kombinasi karakter.

use crate::candidate::CandidateSource;
use crate::model::charset::CharsetSet;
use crate::model::failure::CrackFailure;

/// Menghasilkan setiap kombinasi dari sebuah alfabet untuk panjang `min_length` sampai
/// `max_length`, tanpa menyimpan seluruh ruang ke memori.
#[derive(Debug)]
pub struct BruteForceSource {
    alphabet: Vec<char>,
    min_length: u8,
    max_length: u8,
    indices: Vec<usize>,
    current_length: u8,
    started: bool,
    emitted: u64,
    total: u64,
}

impl BruteForceSource {
    /// Membuat sumber dari sebuah himpunan kelas karakter.
    pub fn new(
        charset: CharsetSet,
        min_length: u8,
        max_length: u8,
    ) -> Result<BruteForceSource, CrackFailure> {
        Self::from_alphabet(charset.alphabet().chars().collect(), min_length, max_length)
    }

    /// Membuat sumber dari alfabet yang sudah jadi. Dipakai oleh pengujian dan oleh
    /// sumber mask.
    pub fn from_alphabet(
        alphabet: Vec<char>,
        min_length: u8,
        max_length: u8,
    ) -> Result<BruteForceSource, CrackFailure> {
        if alphabet.is_empty() {
            return Err(CrackFailure::NoCharsetSelected);
        }
        if min_length == 0 || max_length < min_length {
            return Err(CrackFailure::InvalidLengthRange {
                min: min_length,
                max: max_length,
                allowed_max: max_length,
            });
        }
        let base = alphabet.len() as u64;
        let mut total: u64 = 0;
        for length in min_length..=max_length {
            let mut power: u64 = 1;
            for _ in 0..length {
                match power.checked_mul(base) {
                    Some(value) => power = value,
                    None => {
                        total = u64::MAX;
                        break;
                    }
                }
            }
            match total.checked_add(power) {
                Some(value) => total = value,
                None => {
                    total = u64::MAX;
                    break;
                }
            }
        }
        Ok(BruteForceSource {
            alphabet,
            min_length,
            max_length,
            indices: Vec::new(),
            current_length: min_length,
            started: false,
            emitted: 0,
            total,
        })
    }

    /// Memajukan odometer satu langkah. `false` berarti seluruh ruang sudah habis.
    fn advance(&mut self) -> bool {
        let mut position = self.indices.len();
        loop {
            if position == 0 {
                if self.current_length >= self.max_length {
                    return false;
                }
                self.current_length += 1;
                self.indices = vec![0; self.current_length as usize];
                return true;
            }
            position -= 1;
            self.indices[position] += 1;
            if self.indices[position] < self.alphabet.len() {
                return true;
            }
            self.indices[position] = 0;
        }
    }

    /// Menempatkan odometer langsung sebelum kandidat ke-`index`, sehingga panggilan
    /// `next_candidate` berikutnya menghasilkan kandidat ke-`index`.
    fn seek(&mut self, index: u64) {
        if index == 0 {
            self.started = false;
            self.emitted = 0;
            self.indices.clear();
            self.current_length = self.min_length;
            return;
        }
        let base = self.alphabet.len() as u64;
        let target = index.min(self.total);
        let mut remaining = target - 1;
        let mut length = self.min_length;
        loop {
            let block = base.saturating_pow(length as u32);
            if remaining < block || length >= self.max_length {
                let mut digits = vec![0usize; length as usize];
                let mut value = remaining;
                let mut position = length as usize;
                while position > 0 {
                    position -= 1;
                    digits[position] = (value % base) as usize;
                    value /= base;
                }
                self.current_length = length;
                self.indices = digits;
                self.started = true;
                self.emitted = target;
                return;
            }
            remaining -= block;
            length += 1;
        }
    }
}

impl CandidateSource for BruteForceSource {
    fn next_candidate(&mut self) -> Option<String> {
        if self.started {
            if !self.advance() {
                return None;
            }
        } else {
            self.started = true;
            self.current_length = self.min_length;
            self.indices = vec![0; self.min_length as usize];
        }
        self.emitted += 1;
        Some(
            self.indices
                .iter()
                .map(|&index| self.alphabet[index])
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

    fn drain(source: &mut BruteForceSource) -> Vec<String> {
        let mut out = Vec::new();
        while let Some(candidate) = source.next_candidate() {
            out.push(candidate);
        }
        out
    }

    fn source(min: u8, max: u8) -> BruteForceSource {
        BruteForceSource::from_alphabet(vec!['a', 'b'], min, max).unwrap()
    }

    #[test]
    fn single_length_walks_the_alphabet() {
        assert_eq!(drain(&mut source(1, 1)), vec!["a", "b"]);
    }

    #[test]
    fn longer_lengths_follow_shorter_ones() {
        assert_eq!(
            drain(&mut source(1, 2)),
            vec!["a", "b", "aa", "ab", "ba", "bb"]
        );
    }

    #[test]
    fn total_matches_the_number_of_candidates_yielded() {
        let mut brute = source(1, 3);
        let total = brute.total();
        assert_eq!(total, drain(&mut brute).len() as u64);
    }

    #[test]
    fn total_uses_the_charset_arithmetic() {
        let brute = BruteForceSource::new(CharsetSet::LOWERCASE, 1, 2).unwrap();
        assert_eq!(brute.total(), 26 + 26 * 26);
    }

    #[test]
    fn position_counts_yielded_candidates() {
        let mut brute = source(1, 2);
        assert_eq!(brute.position(), 0);
        brute.next_candidate();
        brute.next_candidate();
        assert_eq!(brute.position(), 2);
    }

    #[test]
    fn skip_lands_on_the_next_candidate() {
        let mut brute = source(1, 2);
        brute.skip(2);
        assert_eq!(brute.position(), 2);
        assert_eq!(brute.next_candidate().as_deref(), Some("aa"));
    }

    #[test]
    fn skip_one_moves_a_single_step() {
        let mut brute = source(1, 2);
        brute.skip(1);
        assert_eq!(brute.next_candidate().as_deref(), Some("b"));
    }

    #[test]
    fn skipping_the_whole_space_yields_nothing() {
        let mut brute = source(1, 2);
        brute.skip(6);
        assert_eq!(brute.next_candidate(), None);
    }

    #[test]
    fn skipping_past_the_end_yields_nothing_and_does_not_panic() {
        let mut brute = source(1, 2);
        brute.skip(1_000);
        assert_eq!(brute.next_candidate(), None);
    }

    #[test]
    fn skipping_zero_changes_nothing() {
        let mut brute = source(1, 2);
        brute.skip(0);
        assert_eq!(brute.next_candidate().as_deref(), Some("a"));
    }

    #[test]
    fn an_empty_charset_is_rejected() {
        assert_eq!(
            BruteForceSource::new(CharsetSet::empty(), 1, 2).unwrap_err(),
            CrackFailure::NoCharsetSelected
        );
    }

    #[test]
    fn a_zero_minimum_is_rejected() {
        assert!(matches!(
            BruteForceSource::new(CharsetSet::LOWERCASE, 0, 2).unwrap_err(),
            CrackFailure::InvalidLengthRange { .. }
        ));
    }

    #[test]
    fn a_maximum_below_the_minimum_is_rejected() {
        assert!(matches!(
            BruteForceSource::new(CharsetSet::LOWERCASE, 3, 2).unwrap_err(),
            CrackFailure::InvalidLengthRange { .. }
        ));
    }
}
