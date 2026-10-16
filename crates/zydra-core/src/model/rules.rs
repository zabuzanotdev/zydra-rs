//! Aturan mangling: variasi yang lazim dipakai orang saat membuat password.
//!
//! Aturan dipakai bersama wordlist. Kamus berisi kata dasar, dan aturan menghasilkan
//! variasi yang sering muncul di dunia nyata: huruf pertama dibesarkan, angka ditambahkan
//! di belakang, huruf diganti angka.
//!
//! # Setiap aturan menghasilkan jumlah keluaran yang tetap
//!
//! Ini bukan detail sepele. Kalau satu aturan menghasilkan jumlah keluaran yang berbeda
//! untuk kata yang berbeda, maka posisi ke-N dalam ruang kandidat tidak lagi bisa dihitung
//! dari N saja. Akibatnya fitur melanjutkan serangan yang terputus kehilangan dasarnya,
//! karena posisi tersimpan tidak bisa diterjemahkan kembali menjadi kata dasar dan aturan.
//!
//! Karena itu `AppendDigits` menghasilkan `max + 1` keluaran untuk kata apa pun, termasuk
//! kata kosong, dan `AppendSymbols` menghasilkan jumlah tetap dari himpunan tetap.

use std::fmt;

use crate::error::CoreError;

/// Himpunan simbol yang ditambahkan aturan `append-symbols`.
const APPENDED_SYMBOLS: [char; 5] = ['!', '@', '#', '$', '%'];

/// Jumlah angka bawaan yang ditambahkan bila pengguna tidak menentukan.
const DEFAULT_DIGIT_MAX: u8 = 9;

/// Peta penggantian untuk aturan `leet`.
const LEET_TABLE: [(char, char); 5] = [('a', '4'), ('e', '3'), ('i', '1'), ('o', '0'), ('s', '5')];

/// Satu aturan mangling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// Kata dasar dikembalikan apa adanya.
    ///
    /// Aturan ini tampak tidak berguna, tetapi tanpanya rangkaian seperti `append-digits`
    /// tidak pernah mencoba kata dasarnya sendiri. Di alat sejenis, aturan kosong ini
    /// ditulis `:`; di sini namanya `none` supaya bisa dibaca tanpa penjelasan.
    Identity,
    /// Huruf pertama dibesarkan, sisanya dikecilkan.
    Capitalize,
    /// Seluruh huruf dibesarkan.
    Upper,
    /// Seluruh huruf dikecilkan.
    Lower,
    /// Urutan karakter dibalik.
    Reverse,
    /// Sebagian huruf diganti angka yang mirip bentuknya.
    Leet,
    /// Angka `0` sampai `max` ditambahkan di belakang.
    AppendDigits {
        /// Angka tertinggi yang dipakai.
        max: u8,
    },
    /// Angka `0` sampai `max` ditambahkan di depan.
    PrependDigits {
        /// Angka tertinggi yang dipakai.
        max: u8,
    },
    /// Simbol lazim ditambahkan di belakang.
    AppendSymbols,
}

impl Rule {
    /// Berapa banyak kandidat yang dihasilkan aturan ini untuk satu kata.
    ///
    /// Jumlahnya harus sama untuk kata apa pun, karena itulah yang membuat posisi dalam
    /// ruang kandidat bisa dihitung secara langsung.
    pub fn output_count(self) -> u64 {
        match self {
            Rule::AppendDigits { max } | Rule::PrependDigits { max } => max as u64 + 1,
            Rule::AppendSymbols => APPENDED_SYMBOLS.len() as u64,
            Rule::Identity
            | Rule::Capitalize
            | Rule::Upper
            | Rule::Lower
            | Rule::Reverse
            | Rule::Leet => 1,
        }
    }

    /// Menerapkan aturan ini pada satu kata.
    ///
    /// Panjang hasilnya selalu sama dengan [`Rule::output_count`].
    pub fn apply(self, word: &str) -> Vec<String> {
        match self {
            Rule::Identity => vec![word.to_string()],
            Rule::Capitalize => vec![capitalize(word)],
            Rule::Upper => vec![word.to_uppercase()],
            Rule::Lower => vec![word.to_lowercase()],
            Rule::Reverse => vec![word.chars().rev().collect()],
            Rule::Leet => vec![to_leet(word)],
            Rule::AppendDigits { max } => (0..=max).map(|digit| format!("{word}{digit}")).collect(),
            Rule::PrependDigits { max } => {
                (0..=max).map(|digit| format!("{digit}{word}")).collect()
            }
            Rule::AppendSymbols => APPENDED_SYMBOLS
                .iter()
                .map(|symbol| format!("{word}{symbol}"))
                .collect(),
        }
    }

    /// Nama yang dipakai di baris perintah.
    pub fn name(self) -> &'static str {
        match self {
            Rule::Identity => "none",
            Rule::Capitalize => "capitalize",
            Rule::Upper => "upper",
            Rule::Lower => "lower",
            Rule::Reverse => "reverse",
            Rule::Leet => "leet",
            Rule::AppendDigits { .. } => "append-digits",
            Rule::PrependDigits { .. } => "prepend-digits",
            Rule::AppendSymbols => "append-symbols",
        }
    }

    /// Mengurai satu nama aturan, dengan parameter opsional setelah `:`.
    fn parse(specification: &str) -> Result<Rule, CoreError> {
        let (name, parameter) = match specification.split_once(':') {
            Some((name, parameter)) => (name.trim(), Some(parameter.trim())),
            None => (specification.trim(), None),
        };

        let digits = |parameter: Option<&str>| -> Result<u8, CoreError> {
            match parameter {
                None => Ok(DEFAULT_DIGIT_MAX),
                Some(value) => value.parse::<u8>().map_err(|_| CoreError::RuleParse {
                    rule: specification.to_string(),
                    reason: "bad-digit-max".to_string(),
                }),
            }
        };

        match name {
            "none" => Ok(Rule::Identity),
            "capitalize" => Ok(Rule::Capitalize),
            "upper" => Ok(Rule::Upper),
            "lower" => Ok(Rule::Lower),
            "reverse" => Ok(Rule::Reverse),
            "leet" => Ok(Rule::Leet),
            "append-digits" => Ok(Rule::AppendDigits {
                max: digits(parameter)?,
            }),
            "prepend-digits" => Ok(Rule::PrependDigits {
                max: digits(parameter)?,
            }),
            "append-symbols" => Ok(Rule::AppendSymbols),
            other => Err(CoreError::RuleParse {
                rule: other.to_string(),
                reason: "unknown-rule".to_string(),
            }),
        }
    }
}

/// Rangkaian aturan yang diterapkan bersamaan.
///
/// Berisi daftar **rantai**. Koma memisahkan rantai yang saling menjadi alternatif, dan
/// tanda tambah merantai aturan di dalam satu rantai:
///
/// ```text
/// none,capitalize,leet,capitalize+append-digits:9
/// ```
///
/// Rantai itu menghasilkan empat kelompok: kata apa adanya, huruf awal dibesarkan, huruf
/// diganti angka, dan huruf awal dibesarkan lalu diberi angka di belakang.
///
/// # Kenapa harus ada rantai
///
/// Tanpa rantai, aturan hanya bisa diterapkan berdampingan pada kata dasar. Rangkaian
/// `capitalize,append-digits` akan menghasilkan `Secret` dan `secret0` sampai `secret9`,
/// tetapi tidak pernah `Secret7` - padahal huruf awal dibesarkan lalu diberi satu angka di
/// belakang justru pola password yang paling sering dipakai orang. Aturan yang tidak bisa
/// menyatakan pola itu tidak banyak gunanya.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuleSet {
    chains: Vec<Vec<Rule>>,
}

impl RuleSet {
    /// Rangkaian aturan yang kosong.
    pub fn empty() -> RuleSet {
        RuleSet { chains: Vec::new() }
    }

    /// Mengurai daftar rantai yang dipisah koma.
    ///
    /// Setiap rantai berisi nama aturan yang dipisah tanda tambah. Segmen kosong diabaikan,
    /// jadi koma berlebih atau tanda tambah di ujung tidak membuat perintah gagal.
    pub fn parse(specification: &str) -> Result<RuleSet, CoreError> {
        let mut chains = Vec::new();
        for chain in specification.split(',') {
            let chain = chain.trim();
            if chain.is_empty() {
                continue;
            }
            let mut rules = Vec::new();
            for part in chain.split('+') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                rules.push(Rule::parse(part)?);
            }
            if !rules.is_empty() {
                chains.push(rules);
            }
        }
        Ok(RuleSet { chains })
    }

    /// Apakah tidak ada aturan sama sekali.
    pub fn is_empty(&self) -> bool {
        self.chains.is_empty()
    }

    /// Jumlah rantai.
    pub fn len(&self) -> usize {
        self.chains.len()
    }

    /// Rantai-rantainya, berurutan.
    pub fn chains(&self) -> &[Vec<Rule>] {
        &self.chains
    }

    /// Berapa banyak kandidat yang dihasilkan rangkaian ini untuk satu kata.
    ///
    /// Selalu minimal satu. Rangkaian kosong menghasilkan kata dasarnya sendiri; rangkaian
    /// berisi menghasilkan jumlah keluaran setiap rantainya, dan kata dasar hanya ikut
    /// kalau ada rantai `none`.
    pub fn outputs_per_word(&self) -> u64 {
        let total: u64 = self.chains.iter().map(|chain| chain_outputs(chain)).sum();
        total.max(1)
    }

    /// Menerapkan seluruh rantai pada satu kata.
    ///
    /// Panjang hasilnya selalu sama dengan [`RuleSet::outputs_per_word`]. Sifat itu yang
    /// membuat posisi dalam ruang kandidat bisa dihitung langsung, dan karenanya melanjutkan
    /// serangan yang terputus mungkin dilakukan.
    pub fn apply(&self, word: &str) -> Vec<String> {
        if self.chains.is_empty() {
            return vec![word.to_string()];
        }
        let mut out = Vec::with_capacity(self.outputs_per_word() as usize);
        for chain in &self.chains {
            out.extend(apply_chain(chain, word));
        }
        out
    }
}

/// Jumlah keluaran satu rantai: hasil kali keluaran setiap aturannya.
fn chain_outputs(chain: &[Rule]) -> u64 {
    let mut total: u64 = 1;
    for rule in chain {
        total = total.saturating_mul(rule.output_count());
    }
    total
}

/// Menerapkan satu rantai: setiap aturan bekerja atas hasil aturan sebelumnya.
fn apply_chain(chain: &[Rule], word: &str) -> Vec<String> {
    let mut results = vec![word.to_string()];
    for rule in chain {
        let mut next = Vec::with_capacity(results.len() * rule.output_count() as usize);
        for current in &results {
            next.extend(rule.apply(current));
        }
        results = next;
    }
    results
}

impl fmt::Display for RuleSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chains: Vec<String> = self
            .chains
            .iter()
            .map(|chain| {
                chain
                    .iter()
                    .map(|rule| rule.name())
                    .collect::<Vec<&str>>()
                    .join("+")
            })
            .collect();
        formatter.write_str(&chains.join(","))
    }
}

fn capitalize(word: &str) -> String {
    let mut characters = word.chars();
    match characters.next() {
        Some(first) => {
            first.to_uppercase().collect::<String>() + &characters.as_str().to_lowercase()
        }
        None => String::new(),
    }
}

fn to_leet(word: &str) -> String {
    word.chars()
        .map(|character| {
            let lower = character.to_ascii_lowercase();
            match LEET_TABLE.iter().find(|(from, _)| *from == lower) {
                Some((_, to)) => *to,
                None => character,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalize_only_touches_the_first_letter() {
        assert_eq!(Rule::Capitalize.apply("hello"), vec!["Hello"]);
        assert_eq!(Rule::Capitalize.apply("HELLO"), vec!["Hello"]);
    }

    #[test]
    fn capitalize_handles_an_empty_word() {
        assert_eq!(Rule::Capitalize.apply(""), vec![""]);
    }

    #[test]
    fn case_rules_work_on_multibyte_text() {
        assert_eq!(Rule::Upper.apply("café"), vec!["CAFÉ"]);
        assert_eq!(Rule::Reverse.apply("abc"), vec!["cba"]);
    }

    #[test]
    fn leet_replaces_the_expected_letters() {
        assert_eq!(Rule::Leet.apply("password"), vec!["p455w0rd"]);
    }

    #[test]
    fn leet_leaves_letters_without_a_lookalike_alone() {
        assert_eq!(Rule::Leet.apply("xyz"), vec!["xyz"]);
    }

    #[test]
    fn append_digits_produces_one_output_per_digit() {
        let outputs = Rule::AppendDigits { max: 3 }.apply("ab");
        assert_eq!(outputs, vec!["ab0", "ab1", "ab2", "ab3"]);
    }

    #[test]
    fn prepend_digits_puts_the_digit_first() {
        let outputs = Rule::PrependDigits { max: 1 }.apply("ab");
        assert_eq!(outputs, vec!["0ab", "1ab"]);
    }

    #[test]
    fn append_symbols_uses_a_fixed_set() {
        let outputs = Rule::AppendSymbols.apply("ab");
        assert_eq!(outputs.len(), APPENDED_SYMBOLS.len());
        assert_eq!(outputs[0], "ab!");
    }

    #[test]
    fn every_rule_produces_exactly_the_number_of_outputs_it_promises() {
        // Ini yang membuat posisi dalam ruang kandidat bisa dihitung langsung, dan
        // karenanya melanjutkan serangan yang terputus mungkin dilakukan.
        let rules = [
            Rule::Identity,
            Rule::Capitalize,
            Rule::Upper,
            Rule::Lower,
            Rule::Reverse,
            Rule::Leet,
            Rule::AppendDigits { max: 4 },
            Rule::PrependDigits { max: 4 },
            Rule::AppendSymbols,
        ];
        for rule in rules {
            for word in ["", "a", "password", "kata-panjang-sekali"] {
                assert_eq!(
                    rule.apply(word).len() as u64,
                    rule.output_count(),
                    "rule {} lied about its output count for {word:?}",
                    rule.name()
                );
            }
        }
    }

    #[test]
    fn the_identity_rule_keeps_the_base_word() {
        assert_eq!(Rule::Identity.apply("admin"), vec!["admin"]);
    }

    #[test]
    fn a_rule_set_without_the_identity_rule_never_tries_the_base_word() {
        // Perilaku ini disengaja dan mengikuti kebiasaan alat sejenis, jadi diuji
        // langsung supaya tidak ada yang mengira ini kekurangan.
        let rules = RuleSet::parse("append-digits:1").unwrap();
        assert_eq!(rules.apply("admin"), vec!["admin0", "admin1"]);
    }

    #[test]
    fn adding_the_identity_rule_puts_the_base_word_back() {
        let rules = RuleSet::parse("none,append-digits:1").unwrap();
        assert_eq!(rules.apply("admin"), vec!["admin", "admin0", "admin1"]);
    }

    #[test]
    fn an_empty_rule_set_returns_the_word_unchanged() {
        let rules = RuleSet::empty();
        assert!(rules.is_empty());
        assert_eq!(rules.outputs_per_word(), 1);
        assert_eq!(rules.apply("admin"), vec!["admin"]);
    }

    #[test]
    fn a_rule_set_concatenates_the_outputs_of_its_rules() {
        let rules = RuleSet::parse("capitalize,append-digits:1").unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules.outputs_per_word(), 1 + 2);
        assert_eq!(rules.apply("admin"), vec!["Admin", "admin0", "admin1"]);
    }

    #[test]
    fn a_chain_applies_its_rules_one_after_another() {
        // Inilah pola password yang paling sering dipakai orang: huruf awal dibesarkan,
        // lalu satu angka ditambahkan di belakang.
        let rules = RuleSet::parse("capitalize+append-digits:1").unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules.outputs_per_word(), 2);
        assert_eq!(rules.apply("admin"), vec!["Admin0", "Admin1"]);
    }

    #[test]
    fn chains_and_alternatives_can_be_mixed() {
        let rules = RuleSet::parse("none,capitalize,capitalize+append-digits:1").unwrap();
        assert_eq!(rules.len(), 3);
        assert_eq!(rules.outputs_per_word(), 1 + 1 + 2);
        assert_eq!(
            rules.apply("admin"),
            vec!["admin", "Admin", "Admin0", "Admin1"]
        );
    }

    #[test]
    fn a_chain_output_count_matches_what_it_actually_produces() {
        let rules = RuleSet::parse("capitalize+leet+append-symbols").unwrap();
        // Huruf awal dibesarkan, huruf diganti angka, lalu simbol ditambahkan.
        for word in ["", "x", "secret"] {
            assert_eq!(
                rules.apply(word).len() as u64,
                rules.outputs_per_word(),
                "output count lied for {word:?}"
            );
        }
    }

    #[test]
    fn a_longer_chain_multiplies_its_outputs() {
        let rules = RuleSet::parse("none+append-digits:1+append-symbols").unwrap();
        // `none` menghasilkan satu keluaran, satu angka menghasilkan dua, lima simbol
        // menghasilkan lima. Hasil kalinya sepuluh.
        assert_eq!(rules.outputs_per_word(), 2 * 5);
    }

    #[test]
    fn a_trailing_plus_is_ignored() {
        let rules = RuleSet::parse("capitalize+").unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules.apply("admin"), vec!["Admin"]);
    }

    #[test]
    fn a_rule_set_output_count_matches_what_it_actually_produces() {
        let rules = RuleSet::parse("leet,append-symbols,prepend-digits:2").unwrap();
        for word in ["", "x", "secret"] {
            assert_eq!(
                rules.apply(word).len() as u64,
                rules.outputs_per_word(),
                "output count lied for {word:?}"
            );
        }
    }

    #[test]
    fn digits_default_to_nine_when_no_parameter_is_given() {
        let rules = RuleSet::parse("append-digits").unwrap();
        assert_eq!(rules.outputs_per_word(), 10);
    }

    #[test]
    fn an_unknown_rule_is_rejected_and_named() {
        let error = RuleSet::parse("capitalize,nope").unwrap_err();
        assert!(matches!(error, CoreError::RuleParse { ref rule, .. } if rule == "nope"));
    }

    #[test]
    fn a_non_numeric_digit_parameter_is_rejected() {
        let error = RuleSet::parse("append-digits:abc").unwrap_err();
        assert!(
            matches!(error, CoreError::RuleParse { ref reason, .. } if reason == "bad-digit-max")
        );
    }

    #[test]
    fn empty_segments_are_ignored() {
        let rules = RuleSet::parse("capitalize,,upper,").unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn a_rule_set_prints_back_as_a_list_of_chains() {
        let rules = RuleSet::parse("capitalize,leet+append-digits:2").unwrap();
        assert_eq!(rules.to_string(), "capitalize,leet+append-digits");
    }
}
