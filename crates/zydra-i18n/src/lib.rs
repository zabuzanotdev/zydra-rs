//! Terjemahan untuk seluruh teks yang dilihat pengguna.
//!
//! # Kenapa crate terpisah
//!
//! `zydra-core` mengembalikan kesalahan sebagai tipe, bukan kalimat. Crate ini yang
//! mengubahnya menjadi teks. Pemisahan itu membuat core bisa diuji tanpa antarmuka, dan
//! membuat satu kalimat hanya punya satu tempat untuk diubah.
//!
//! # Berkas locale disematkan
//!
//! `en.ftl` dan `id.ftl` dimasukkan ke dalam binary dengan `include_str!`. Tidak ada berkas
//! yang bisa hilang saat aplikasi dipindahkan atau dipasang, dan tidak ada kegagalan saat
//! dijalankan hanya karena berkas terjemahan tidak ditemukan.
//!
//! # Kunci yang hilang tidak mematikan aplikasi
//!
//! Pencarian dilakukan ke locale aktif lebih dulu, lalu ke Bahasa Inggris. Kalau keduanya
//! tidak punya kuncinya, kunci itu sendiri yang dikembalikan supaya kekurangan terlihat
//! tanpa membuat program berhenti.

mod failure;
mod log;
mod progress;

pub mod format;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

use zydra_core::log::LogEntry;
use zydra_core::model::failure::CrackFailure;
use zydra_core::model::progress::CrackProgress;

const EN_FTL: &str = include_str!("../locales/en.ftl");
const ID_FTL: &str = include_str!("../locales/id.ftl");

/// Bahasa antarmuka yang didukung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    /// Bahasa Inggris.
    En,
    /// Bahasa Indonesia.
    Id,
}

impl Locale {
    /// Semua bahasa yang didukung, dalam urutan yang dipakai untuk dokumentasi.
    pub const ALL: [Locale; 2] = [Locale::En, Locale::Id];

    /// Kode bahasa yang dipakai di baris perintah dan berkas konfigurasi.
    pub fn code(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Id => "id",
        }
    }

    /// Mengurai kode bahasa. Awalan saja sudah cukup, jadi `id-ID` juga berarti Indonesia.
    pub fn parse(value: &str) -> Option<Locale> {
        let lower = value.trim().to_ascii_lowercase();
        let prefix = lower.split(['-', '_']).next().unwrap_or("");
        match prefix {
            "en" => Some(Locale::En),
            "id" => Some(Locale::Id),
            _ => None,
        }
    }

    /// Menebak bahasa dari pengaturan sistem, dan jatuh ke Bahasa Inggris kalau tidak
    /// dikenali.
    pub fn detect() -> Locale {
        sys_locale::get_locale()
            .as_deref()
            .and_then(Locale::parse)
            .unwrap_or(Locale::En)
    }

    fn language_id(self) -> LanguageIdentifier {
        match self {
            Locale::En => "en".parse().expect("static language tag"),
            Locale::Id => "id".parse().expect("static language tag"),
        }
    }

    fn source(self) -> &'static str {
        match self {
            Locale::En => EN_FTL,
            Locale::Id => ID_FTL,
        }
    }
}

/// Katalog teks yang sedang aktif.
pub struct Messages {
    active: FluentBundle<FluentResource>,
    fallback: FluentBundle<FluentResource>,
    locale: Locale,
}

impl std::fmt::Debug for Messages {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Messages")
            .field("locale", &self.locale)
            .finish()
    }
}

impl Messages {
    /// Membuat katalog untuk sebuah bahasa.
    pub fn new(locale: Locale) -> Messages {
        Messages {
            active: bundle_for(locale),
            fallback: bundle_for(Locale::En),
            locale,
        }
    }

    /// Katalog dengan bahasa yang ditebak dari sistem.
    pub fn detected() -> Messages {
        Messages::new(Locale::detect())
    }

    /// Bahasa yang sedang aktif.
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// Mengganti bahasa. Seluruh teks ikut berubah pada pemanggilan berikutnya.
    pub fn set_locale(&mut self, locale: Locale) {
        if locale == self.locale {
            return;
        }
        self.active = bundle_for(locale);
        self.locale = locale;
    }

    /// Teks untuk sebuah kunci, tanpa argumen.
    pub fn text(&self, key: &str) -> String {
        self.format(key, None).unwrap_or_else(|| key.to_string())
    }

    /// Teks untuk sebuah kunci, dengan argumen.
    pub fn text_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, *value);
        }
        self.format(key, Some(&fluent_args))
            .unwrap_or_else(|| key.to_string())
    }

    /// Kalimat untuk sebuah kegagalan pemulihan.
    pub fn failure(&self, failure: &CrackFailure) -> String {
        let (key, args) = failure::describe(failure);
        let borrowed: Vec<(&str, &str)> = args
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        self.text_args(key, &borrowed)
    }

    /// Kalimat ringkas untuk sebuah langkah kemajuan.
    pub fn progress(&self, progress: &CrackProgress) -> String {
        progress::describe(self, progress)
    }

    /// Satu baris log lengkap: waktu, tingkat, lalu teksnya.
    pub fn log_line(&self, entry: &LogEntry) -> String {
        log::describe(self, entry)
    }

    /// Menyisir locale aktif lalu Bahasa Inggris.
    fn format(&self, key: &str, args: Option<&FluentArgs>) -> Option<String> {
        for bundle in [&self.active, &self.fallback] {
            let Some(message) = bundle.get_message(key) else {
                continue;
            };
            let Some(pattern) = message.value() else {
                continue;
            };
            let mut errors = Vec::new();
            let value = bundle.format_pattern(pattern, args, &mut errors);
            return Some(value.into_owned());
        }
        None
    }
}

/// Menyiapkan bundle untuk satu bahasa.
///
/// Berkas locale yang salah ketik tidak membuat aplikasi panik: `try_new` tetap
/// mengembalikan sumber dayanya bersama daftar kesalahan, dan pesan yang rusak itu saja
/// yang jatuh ke Bahasa Inggris.
fn bundle_for(locale: Locale) -> FluentBundle<FluentResource> {
    let mut bundle = FluentBundle::new(vec![locale.language_id()]);
    // Tanpa ini, fluent membungkus nilai yang disisipkan dengan tanda isolasi Unicode yang
    // tidak terlihat di terminal tetapi ikut tersalin ke mana-mana.
    bundle.set_use_isolating(false);
    let resource = FluentResource::try_new(locale.source().to_string())
        .unwrap_or_else(|(resource, _errors)| resource);
    bundle.add_resource_overriding(resource);
    bundle
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn keys(source: &str) -> BTreeSet<String> {
        source
            .lines()
            .filter_map(|line| line.split_once('=').map(|(key, _)| key.trim().to_string()))
            .filter(|key| !key.is_empty() && !key.starts_with('#'))
            .collect()
    }

    #[test]
    fn both_locales_define_exactly_the_same_keys() {
        // Kunci yang hanya ada di satu bahasa akan tampil sebagai kunci mentah bagi
        // pengguna bahasa itu, jadi ketidakcocokan harus ketahuan di sini.
        assert_eq!(keys(EN_FTL), keys(ID_FTL));
    }

    #[test]
    fn no_key_is_left_empty() {
        for source in [EN_FTL, ID_FTL] {
            for line in source.lines() {
                let Some((key, value)) = line.split_once('=') else {
                    continue;
                };
                if key.trim().is_empty() || key.trim().starts_with('#') {
                    continue;
                }
                assert!(!value.trim().is_empty(), "key {} has no text", key.trim());
            }
        }
    }

    #[test]
    fn english_text_is_resolved() {
        let messages = Messages::new(Locale::En);
        assert_eq!(messages.text("result-granted"), "Password found");
    }

    #[test]
    fn indonesian_text_is_resolved() {
        let messages = Messages::new(Locale::Id);
        assert_eq!(messages.text("result-granted"), "Password ditemukan");
    }

    #[test]
    fn arguments_are_substituted() {
        let messages = Messages::new(Locale::En);
        let text = messages.text_args("error-unsupported-format", &[("name", "notes.txt")]);
        assert_eq!(text, "Unsupported file type: notes.txt");
    }

    #[test]
    fn a_missing_key_falls_back_to_english_then_to_the_key_itself() {
        let messages = Messages::new(Locale::Id);
        // Kunci ini ada di kedua bahasa; yang diuji adalah perilaku saat tidak ada.
        assert_eq!(
            messages.text("no-such-key-anywhere"),
            "no-such-key-anywhere"
        );
    }

    #[test]
    fn changing_the_locale_changes_the_text() {
        let mut messages = Messages::new(Locale::En);
        assert_eq!(messages.locale(), Locale::En);
        messages.set_locale(Locale::Id);
        assert_eq!(messages.locale(), Locale::Id);
        assert_eq!(messages.text("result-denied"), "Password tidak ditemukan");
    }

    #[test]
    fn locale_codes_round_trip() {
        for locale in Locale::ALL {
            assert_eq!(Locale::parse(locale.code()), Some(locale));
        }
    }

    #[test]
    fn locale_parsing_accepts_regions_and_separators() {
        assert_eq!(Locale::parse("id-ID"), Some(Locale::Id));
        assert_eq!(Locale::parse("en_US"), Some(Locale::En));
        assert_eq!(Locale::parse("ID"), Some(Locale::Id));
        assert_eq!(Locale::parse("fr"), None);
        assert_eq!(Locale::parse(""), None);
    }

    #[test]
    fn detection_always_returns_a_supported_locale() {
        let detected = Locale::detect();
        assert!(Locale::ALL.contains(&detected));
    }
}
