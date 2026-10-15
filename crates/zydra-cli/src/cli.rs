//! Definisi argumen baris perintah.
//!
//! Nama argumen yang lama dari skrip Python (`-f`, `-d`, `-b`, `-m`, `-x`) dipertahankan
//! supaya perintah yang sudah ditulis orang tidak rusak. Teks bantuannya sendiri diambil
//! dari katalog terjemahan, bukan dari atribut di sini.

use std::path::PathBuf;

use clap::{ArgMatches, Command, CommandFactory, FromArgMatches, Parser};
use zydra_i18n::Messages;

/// Argumen yang diterima `zydra`.
#[derive(Debug, Parser)]
#[command(name = "zydra", version)]
pub struct Args {
    /// Target file.
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Wordlist file.
    #[arg(short = 'd', long = "wordlist", value_name = "WORDLIST")]
    pub wordlist: Option<PathBuf>,

    /// Character classes.
    #[arg(short = 'b', long = "charset", value_name = "CLASSES")]
    pub charset: Option<String>,

    /// Minimum length.
    #[arg(short = 'm', long = "min-length", value_name = "N")]
    pub min_length: Option<u8>,

    /// Maximum length.
    #[arg(short = 'x', long = "max-length", value_name = "N")]
    pub max_length: Option<u8>,

    /// Mask pattern.
    #[arg(long = "mask", value_name = "PATTERN")]
    pub mask: Option<String>,

    /// Mangling rules.
    #[arg(long = "rules", value_name = "LIST")]
    pub rules: Option<String>,

    /// Worker threads.
    #[arg(long = "threads", value_name = "N", default_value_t = 0)]
    pub threads: usize,

    /// Interface language.
    #[arg(long = "lang", value_name = "CODE")]
    pub lang: Option<String>,

    /// Machine readable output.
    #[arg(long = "json")]
    pub json: bool,

    /// Plain output.
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Continue a previous run.
    #[arg(long = "resume")]
    pub resume: bool,
}

/// Pasangan nama argumen dan kunci bantuan yang dipakai.
const HELP_KEYS: [(&str, &str); 12] = [
    ("file", "cli-help-file"),
    ("wordlist", "cli-help-wordlist"),
    ("charset", "cli-help-charset"),
    ("min_length", "cli-help-min-length"),
    ("max_length", "cli-help-max-length"),
    ("mask", "cli-help-mask"),
    ("rules", "cli-help-rules"),
    ("threads", "cli-help-threads"),
    ("lang", "cli-help-lang"),
    ("json", "cli-help-json"),
    ("quiet", "cli-help-quiet"),
    ("resume", "cli-help-resume"),
];

/// Membangun perintah clap dengan seluruh teks dari katalog terjemahan.
///
/// Tata letak bantuan ditulis sendiri karena clap membawa kata bawaannya sendiri
/// ("Usage:", "Options:", "Print help") yang semuanya terlihat pengguna.
pub fn command(messages: &Messages) -> Command {
    let template = format!(
        "{{about}}\n\n{} {{usage}}\n\n{}\n{{options}}",
        messages.text("cli-heading-usage"),
        messages.text("cli-heading-options")
    );

    let mut command = Args::command()
        .about(messages.text("cli-about"))
        .help_template(template);
    for (id, key) in HELP_KEYS {
        command = command.mut_arg(id, |arg| arg.help(messages.text(key)));
    }
    // Arg bantuan dan versi bawaan clap baru ada setelah perintah dibangun, jadi
    // deskripsinya baru bisa diganti di titik ini. `build` bekerja di tempat dan tidak
    // mengembalikan nilai.
    let mut command = command;
    command.build();
    command
        .mut_arg("help", |arg| arg.help(messages.text("cli-help-help")))
        .mut_arg("version", |arg| arg.help(messages.text("cli-help-version")))
}

/// Mengurai argumen dengan teks bantuan yang sudah diterjemahkan.
pub fn parse_from(messages: &Messages, args: &[String]) -> Result<Args, clap::Error> {
    let matches: ArgMatches = command(messages).try_get_matches_from(args)?;
    Args::from_arg_matches(&matches)
}

/// Menyisir argumen mentah untuk `--lang`, sebelum clap dijalankan.
///
/// Tanpa langkah ini, `--help` akan selalu tampil dalam bahasa Inggris: teks bantuan clap
/// disusun saat perintah dibangun, jadi bahasanya harus diketahui lebih dulu.
pub fn scan_language(args: &[String]) -> Option<String> {
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix("--lang=") {
            return Some(value.to_string());
        }
        if arg == "--lang" {
            return args.get(index + 1).cloned();
        }
        index += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use zydra_i18n::Locale;

    #[test]
    fn the_python_flag_letters_still_work() {
        let messages = Messages::new(Locale::En);
        let args = parse_from(
            &messages,
            &[
                "zydra".to_string(),
                "-f".into(),
                "vault.zip".into(),
                "-d".into(),
                "words.txt".into(),
            ],
        )
        .unwrap();
        assert_eq!(args.file.unwrap().to_string_lossy(), "vault.zip");
        assert_eq!(args.wordlist.unwrap().to_string_lossy(), "words.txt");
    }

    #[test]
    fn brute_force_flags_are_parsed() {
        let messages = Messages::new(Locale::En);
        let args = parse_from(
            &messages,
            &[
                "zydra".to_string(),
                "-f".into(),
                "v.zip".into(),
                "-b".into(),
                "digits".into(),
                "-m".into(),
                "1".into(),
                "-x".into(),
                "4".into(),
            ],
        )
        .unwrap();
        assert_eq!(args.charset.as_deref(), Some("digits"));
        assert_eq!(args.min_length, Some(1));
        assert_eq!(args.max_length, Some(4));
    }

    #[test]
    fn long_names_are_accepted_too() {
        let messages = Messages::new(Locale::En);
        let args = parse_from(
            &messages,
            &[
                "zydra".to_string(),
                "--file".into(),
                "v.zip".into(),
                "--wordlist".into(),
                "w.txt".into(),
                "--threads".into(),
                "3".into(),
            ],
        )
        .unwrap();
        assert_eq!(args.threads, 3);
    }

    #[test]
    fn help_text_follows_the_catalogue() {
        let english = command(&Messages::new(Locale::En))
            .render_help()
            .to_string();
        let indonesian = command(&Messages::new(Locale::Id))
            .render_help()
            .to_string();

        assert!(english.contains("Recover the password"), "{english}");
        assert!(indonesian.contains("Memulihkan password"), "{indonesian}");
    }

    #[test]
    fn even_the_help_headings_are_translated() {
        let indonesian = command(&Messages::new(Locale::Id))
            .render_help()
            .to_string();
        assert!(indonesian.contains("Penggunaan:"), "{indonesian}");
        assert!(indonesian.contains("Pilihan:"), "{indonesian}");
        assert!(indonesian.contains("Tampilkan bantuan ini"), "{indonesian}");
        // Tidak boleh ada kata bawaan clap yang bocor ke layar.
        assert!(!indonesian.contains("Usage:"), "{indonesian}");
        assert!(!indonesian.contains("Options:"), "{indonesian}");
        assert!(!indonesian.contains("Print help"), "{indonesian}");
    }

    #[test]
    fn the_language_can_be_scanned_before_parsing() {
        let args = ["zydra", "--lang", "id", "-f", "x"].map(String::from);
        assert_eq!(scan_language(&args).as_deref(), Some("id"));
        let args = ["zydra", "--lang=id"].map(String::from);
        assert_eq!(scan_language(&args).as_deref(), Some("id"));
        let args = ["zydra", "-f", "x"].map(String::from);
        assert_eq!(scan_language(&args), None);
        let args = ["zydra", "--lang"].map(String::from);
        assert_eq!(scan_language(&args), None);
    }

    #[test]
    fn an_unknown_flag_is_reported_as_a_usage_error() {
        let messages = Messages::new(Locale::En);
        let error = parse_from(&messages, &["zydra".to_string(), "--nope".into()]).unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }
}
