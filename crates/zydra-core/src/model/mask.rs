//! Pola mask: cara menyatakan bentuk password, bukan sekadar rentang panjang.
//!
//! Serangan brute force biasa mencoba semua kombinasi dari satu himpunan karakter untuk
//! setiap panjang. Itu boros ketika bentuk passwordnya sudah bisa diduga. Mask
//! mempersempit setiap posisi ke kelasnya sendiri, jadi `?l?l?l?l?d?d` hanya mencoba
//! empat huruf kecil diikuti dua angka, bukan seluruh kombinasi enam karakter.
//!
//! Sintaksnya mengikuti kebiasaan yang sudah lazim di alat sejenis:
//!
//! | Token | Arti |
//! |---|---|
//! | `?l` | huruf kecil |
//! | `?u` | huruf besar |
//! | `?d` | angka |
//! | `?s` | tanda baca |
//! | `?a` | gabungan keempatnya |
//! | `??` | karakter `?` itu sendiri |
//!
//! Kelas byte ala hashcat (`?b`) sengaja tidak didukung. Menyediakan alias yang artinya
//! berbeda dari yang diharapkan orang justru lebih berbahaya daripada tidak ada.

use std::fmt;

use crate::error::CoreError;
use crate::model::charset::{DIGIT_CHARS, LOWERCASE_CHARS, SYMBOL_CHARS, UPPERCASE_CHARS};

/// Satu pola mask yang sudah diurai.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskPattern {
    source: String,
    positions: Vec<Vec<char>>,
}

impl MaskPattern {
    /// Mengurai sebuah pola.
    ///
    /// Karakter biasa menjadi posisi dengan satu kemungkinan. Sebuah `?` di akhir pola
    /// adalah kesalahan, bukan karakter literal, karena itu hampir selalu berarti tokennya
    /// terpotong.
    pub fn parse(pattern: &str) -> Result<MaskPattern, CoreError> {
        let mut positions: Vec<Vec<char>> = Vec::new();
        let mut characters = pattern.chars();

        while let Some(character) = characters.next() {
            if character != '?' {
                positions.push(vec![character]);
                continue;
            }
            let Some(token) = characters.next() else {
                return Err(CoreError::MaskParse {
                    pattern: pattern.to_string(),
                    reason: "trailing-question-mark".to_string(),
                });
            };
            match token {
                'l' => positions.push(LOWERCASE_CHARS.chars().collect()),
                'u' => positions.push(UPPERCASE_CHARS.chars().collect()),
                'd' => positions.push(DIGIT_CHARS.chars().collect()),
                's' => positions.push(SYMBOL_CHARS.chars().collect()),
                'a' => positions.push(all_printable().collect()),
                '?' => positions.push(vec!['?']),
                other => {
                    return Err(CoreError::MaskParse {
                        pattern: pattern.to_string(),
                        reason: format!("unknown-token-{other}"),
                    })
                }
            }
        }

        if positions.is_empty() {
            return Err(CoreError::MaskParse {
                pattern: pattern.to_string(),
                reason: "empty-pattern".to_string(),
            });
        }

        Ok(MaskPattern {
            source: pattern.to_string(),
            positions,
        })
    }

    /// Pola aslinya, apa adanya.
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// Alfabet untuk setiap posisi.
    pub fn positions(&self) -> &[Vec<char>] {
        &self.positions
    }

    /// Panjang password yang dihasilkan pola ini.
    pub fn length(&self) -> usize {
        self.positions.len()
    }

    /// Jumlah kandidat. Menjenuh ke `u64::MAX` saat melimpah, tidak membungkus.
    pub fn total(&self) -> u64 {
        let mut total: u64 = 1;
        for position in &self.positions {
            let size = position.len() as u64;
            match total.checked_mul(size) {
                Some(value) => total = value,
                None => return u64::MAX,
            }
        }
        total
    }
}

impl fmt::Display for MaskPattern {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.source)
    }
}

/// Seluruh karakter yang bisa dicetak, tanpa duplikat, dalam urutan tetap.
fn all_printable() -> impl Iterator<Item = char> {
    let mut seen = [false; 128];
    let mut collected = Vec::new();
    for source in [LOWERCASE_CHARS, UPPERCASE_CHARS, DIGIT_CHARS, SYMBOL_CHARS] {
        for character in source.chars() {
            let index = character as usize;
            if index < seen.len() && !seen[index] {
                seen[index] = true;
                collected.push(character);
            }
        }
    }
    collected.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pattern_of_literal_characters_has_one_candidate() {
        let mask = MaskPattern::parse("abc").unwrap();
        assert_eq!(mask.length(), 3);
        assert_eq!(mask.total(), 1);
    }

    #[test]
    fn classes_expand_to_their_alphabets() {
        let mask = MaskPattern::parse("?d").unwrap();
        assert_eq!(mask.positions()[0].len(), 10);
        assert_eq!(mask.total(), 10);
    }

    #[test]
    fn the_total_is_the_product_of_every_position() {
        let mask = MaskPattern::parse("?l?l?d?d").unwrap();
        assert_eq!(mask.length(), 4);
        assert_eq!(mask.total(), 26 * 26 * 10 * 10);
    }

    #[test]
    fn a_mixed_pattern_combines_literals_and_classes() {
        let mask = MaskPattern::parse("admin?d?d").unwrap();
        assert_eq!(mask.length(), 7);
        assert_eq!(mask.total(), 100);
        assert_eq!(mask.positions()[0], vec!['a']);
        assert_eq!(mask.positions()[5].len(), 10);
    }

    #[test]
    fn a_double_question_mark_is_a_literal_question_mark() {
        let mask = MaskPattern::parse("a??b").unwrap();
        assert_eq!(mask.length(), 3);
        assert_eq!(mask.total(), 1);
        assert_eq!(mask.positions()[1], vec!['?']);
    }

    #[test]
    fn the_all_class_holds_every_printable_character_exactly_once() {
        let mask = MaskPattern::parse("?a").unwrap();
        let alphabet = &mask.positions()[0];
        let mut sorted = alphabet.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), alphabet.len(), "the class has duplicates");
        // Huruf kecil, huruf besar, angka, dan tanda baca.
        assert_eq!(alphabet.len(), 26 + 26 + 10 + SYMBOL_CHARS.chars().count());
    }

    #[test]
    fn a_trailing_question_mark_is_rejected() {
        let error = MaskPattern::parse("ab?").unwrap_err();
        assert!(
            matches!(error, CoreError::MaskParse { ref reason, .. } if reason == "trailing-question-mark")
        );
    }

    #[test]
    fn an_unknown_token_is_rejected_and_named() {
        let error = MaskPattern::parse("?z").unwrap_err();
        assert!(
            matches!(error, CoreError::MaskParse { ref reason, .. } if reason == "unknown-token-z")
        );
    }

    #[test]
    fn an_empty_pattern_is_rejected() {
        let error = MaskPattern::parse("").unwrap_err();
        assert!(
            matches!(error, CoreError::MaskParse { ref reason, .. } if reason == "empty-pattern")
        );
    }

    #[test]
    fn an_enormous_pattern_saturates_instead_of_wrapping() {
        let mask = MaskPattern::parse(&"?a".repeat(40)).unwrap();
        assert_eq!(mask.total(), u64::MAX);
    }

    #[test]
    fn the_original_pattern_is_kept_for_display() {
        let mask = MaskPattern::parse("?l?d").unwrap();
        assert_eq!(mask.as_str(), "?l?d");
        assert_eq!(mask.to_string(), "?l?d");
    }

    #[test]
    fn positions_are_ordered_left_to_right() {
        let mask = MaskPattern::parse("?dA?l").unwrap();
        assert_eq!(mask.positions()[0].len(), 10);
        assert_eq!(mask.positions()[1], vec!['A']);
        assert_eq!(mask.positions()[2].len(), 26);
    }
}
