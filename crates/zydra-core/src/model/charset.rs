//! Kelas karakter untuk serangan brute force.

use bitflags::bitflags;

use crate::error::CoreError;

/// Huruf kecil.
pub const LOWERCASE_CHARS: &str = "abcdefghijklmnopqrstuvwxyz";

/// Huruf besar.
pub const UPPERCASE_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Angka.
pub const DIGIT_CHARS: &str = "0123456789";

/// Tanda baca.
pub const SYMBOL_CHARS: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// Spasi.
pub const SPACE_CHARS: &str = " ";

bitflags! {
    /// Himpunan kelas karakter. Nama kelas adalah konstanta di dalam tipe ini; tidak ada
    /// enum terpisah untuk konsep yang sama.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CharsetSet: u8 {
        /// Huruf kecil.
        const LOWERCASE = 0b0_0001;
        /// Huruf besar.
        const UPPERCASE = 0b0_0010;
        /// Angka.
        const DIGITS    = 0b0_0100;
        /// Tanda baca.
        const SYMBOLS   = 0b0_1000;
        /// Spasi.
        const SPACE     = 0b1_0000;
    }
}

impl CharsetSet {
    /// Menyusun alfabet dalam urutan tetap: huruf kecil, huruf besar, angka, tanda baca,
    /// spasi.
    pub fn alphabet(self) -> String {
        let mut out = String::new();
        if self.contains(CharsetSet::LOWERCASE) {
            out.push_str(LOWERCASE_CHARS);
        }
        if self.contains(CharsetSet::UPPERCASE) {
            out.push_str(UPPERCASE_CHARS);
        }
        if self.contains(CharsetSet::DIGITS) {
            out.push_str(DIGIT_CHARS);
        }
        if self.contains(CharsetSet::SYMBOLS) {
            out.push_str(SYMBOL_CHARS);
        }
        if self.contains(CharsetSet::SPACE) {
            out.push_str(SPACE_CHARS);
        }
        out
    }

    /// Jumlah kandidat untuk panjang `min` sampai `max`. Menjenuh ke `u64::MAX` saat
    /// melimpah, tidak membungkus.
    pub fn total_combinations(self, min: u8, max: u8) -> u64 {
        let base = self.alphabet().chars().count() as u64;
        if base == 0 || max < min {
            return 0;
        }
        let mut total: u64 = 0;
        for length in min..=max {
            let mut power: u64 = 1;
            for _ in 0..length {
                match power.checked_mul(base) {
                    Some(value) => power = value,
                    None => return u64::MAX,
                }
            }
            match total.checked_add(power) {
                Some(value) => total = value,
                None => return u64::MAX,
            }
        }
        total
    }

    /// Mengurai daftar nama kelas yang dipisah koma. Segmen kosong diabaikan.
    pub fn parse(spec: &str) -> Result<CharsetSet, CoreError> {
        let mut set = CharsetSet::empty();
        for raw in spec.split(',') {
            let name = raw.trim();
            if name.is_empty() {
                continue;
            }
            match name {
                "lowercase" => set |= CharsetSet::LOWERCASE,
                "uppercase" => set |= CharsetSet::UPPERCASE,
                "digits" => set |= CharsetSet::DIGITS,
                "symbols" => set |= CharsetSet::SYMBOLS,
                "space" => set |= CharsetSet::SPACE,
                other => {
                    return Err(CoreError::CharsetParse {
                        name: other.to_string(),
                    })
                }
            }
        }
        if set.is_empty() {
            return Err(CoreError::CharsetParse {
                name: spec.to_string(),
            });
        }
        Ok(set)
    }

    /// Menuliskan himpunan ini kembali sebagai daftar nama kelas.
    ///
    /// Kebalikan dari [`CharsetSet::parse`], dalam urutan tetap yang sama dengan
    /// [`CharsetSet::alphabet`]. Dipakai untuk menampilkan pilihan pengguna dan untuk
    /// menyimpannya ke berkas sesi.
    pub fn to_spec(self) -> String {
        let mut names: Vec<&str> = Vec::new();
        if self.contains(CharsetSet::LOWERCASE) {
            names.push("lowercase");
        }
        if self.contains(CharsetSet::UPPERCASE) {
            names.push("uppercase");
        }
        if self.contains(CharsetSet::DIGITS) {
            names.push("digits");
        }
        if self.contains(CharsetSet::SYMBOLS) {
            names.push("symbols");
        }
        if self.contains(CharsetSet::SPACE) {
            names.push("space");
        }
        names.join(",")
    }
}

/// Serialisasi memakai bentuk teks yang sama dengan baris perintah.
///
/// Bentuk ini dipilih karena bisa dibaca manusia dan sudah stabil: berkas sesi yang
/// menyimpan `"digits,lowercase"` masih bisa dipahami orang, dan bentuknya tidak berubah
/// kalau representasi bit di dalam berubah.
impl serde::Serialize for CharsetSet {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_spec())
    }
}

impl<'de> serde::Deserialize<'de> for CharsetSet {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<CharsetSet, D::Error> {
        use serde::de::Error as _;

        let spec = String::deserialize(deserializer)?;
        // Himpunan kosong ditulis sebagai teks kosong, dan `parse` menolaknya karena pada
        // baris perintah himpunan kosong memang kesalahan pengguna. Di sini teks kosong
        // harus diterima supaya hasil serialisasinya bisa dibaca kembali.
        if spec.trim().is_empty() {
            return Ok(CharsetSet::empty());
        }
        CharsetSet::parse(&spec).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercase_alphabet_is_the_26_letters_in_order() {
        assert_eq!(
            CharsetSet::LOWERCASE.alphabet(),
            "abcdefghijklmnopqrstuvwxyz"
        );
    }

    #[test]
    fn combined_alphabet_has_no_duplicate_characters() {
        let alphabet = (CharsetSet::LOWERCASE
            | CharsetSet::UPPERCASE
            | CharsetSet::DIGITS
            | CharsetSet::SYMBOLS
            | CharsetSet::SPACE)
            .alphabet();
        let mut chars: Vec<char> = alphabet.chars().collect();
        let before = chars.len();
        chars.sort_unstable();
        chars.dedup();
        assert_eq!(chars.len(), before);
    }

    #[test]
    fn combinations_for_one_length_is_the_alphabet_size() {
        assert_eq!(CharsetSet::LOWERCASE.total_combinations(1, 1), 26);
    }

    #[test]
    fn combinations_sum_over_every_length() {
        let set = CharsetSet::LOWERCASE | CharsetSet::DIGITS;
        assert_eq!(set.total_combinations(1, 2), 36 + 36 * 36);
    }

    #[test]
    fn combinations_saturate_instead_of_wrapping() {
        let set = CharsetSet::LOWERCASE
            | CharsetSet::UPPERCASE
            | CharsetSet::DIGITS
            | CharsetSet::SYMBOLS
            | CharsetSet::SPACE;
        assert_eq!(set.total_combinations(1, 60), u64::MAX);
    }

    #[test]
    fn combinations_of_an_empty_set_is_zero() {
        assert_eq!(CharsetSet::empty().total_combinations(1, 4), 0);
    }

    #[test]
    fn combinations_of_an_inverted_range_is_zero() {
        assert_eq!(CharsetSet::LOWERCASE.total_combinations(4, 2), 0);
    }

    #[test]
    fn parse_accepts_every_canonical_class_name() {
        let set = CharsetSet::parse("lowercase,uppercase,digits,symbols,space").unwrap();
        assert_eq!(
            set,
            CharsetSet::LOWERCASE
                | CharsetSet::UPPERCASE
                | CharsetSet::DIGITS
                | CharsetSet::SYMBOLS
                | CharsetSet::SPACE
        );
    }

    #[test]
    fn parse_is_order_independent_and_tolerates_whitespace() {
        assert_eq!(
            CharsetSet::parse("digits, lowercase").unwrap(),
            CharsetSet::parse("lowercase,digits").unwrap()
        );
    }

    #[test]
    fn parse_rejects_an_unknown_class_and_names_it() {
        let error = CharsetSet::parse("lowercase,letters").unwrap_err();
        assert!(matches!(error, CoreError::CharsetParse { ref name } if name == "letters"));
    }

    #[test]
    fn parse_rejects_an_empty_specification() {
        assert!(CharsetSet::parse("").is_err());
        assert!(CharsetSet::parse(",,,").is_err());
    }

    #[test]
    fn to_spec_lists_the_classes_in_a_fixed_order() {
        let set = CharsetSet::DIGITS | CharsetSet::LOWERCASE;
        assert_eq!(set.to_spec(), "lowercase,digits");
    }

    #[test]
    fn every_subset_survives_a_round_trip_through_text() {
        // Sifat ini yang membuat berkas sesi bisa dibaca kembali dengan benar. Kalau ada
        // satu himpunan yang tidak bolak-balik dengan utuh, sesinya akan dilanjutkan
        // dengan kelas karakter yang berbeda dari yang dipilih pengguna.
        let classes = [
            CharsetSet::LOWERCASE,
            CharsetSet::UPPERCASE,
            CharsetSet::DIGITS,
            CharsetSet::SYMBOLS,
            CharsetSet::SPACE,
        ];
        for mask in 0u8..32 {
            let mut set = CharsetSet::empty();
            for (index, class) in classes.iter().enumerate() {
                if mask & (1 << index) != 0 {
                    set |= *class;
                }
            }
            let spec = set.to_spec();
            if set.is_empty() {
                assert_eq!(spec, "");
                continue;
            }
            assert_eq!(CharsetSet::parse(&spec).unwrap(), set, "spec was {spec:?}");
        }
    }

    #[test]
    fn an_empty_set_is_written_as_an_empty_string() {
        assert_eq!(CharsetSet::empty().to_spec(), "");
    }

    #[test]
    fn a_charset_serialises_as_readable_text() {
        let set = CharsetSet::LOWERCASE | CharsetSet::DIGITS;
        let encoded = serde_json::to_string(&set).unwrap();
        assert_eq!(encoded, "\"lowercase,digits\"");
        let decoded: CharsetSet = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, set);
    }

    #[test]
    fn an_empty_charset_survives_serialisation_even_though_parsing_it_is_an_error() {
        let encoded = serde_json::to_string(&CharsetSet::empty()).unwrap();
        assert_eq!(encoded, "\"\"");
        let decoded: CharsetSet = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, CharsetSet::empty());
    }

    #[test]
    fn an_unknown_class_in_a_serialised_charset_is_an_error() {
        let result: Result<CharsetSet, _> = serde_json::from_str("\"lowercase,nope\"");
        assert!(result.is_err());
    }
}
