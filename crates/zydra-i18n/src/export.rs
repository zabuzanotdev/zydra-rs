//! Menulis catatan aktivitas ke sebuah berkas.
//!
//! Tinggal di sini, bukan di masing-masing antarmuka, karena render satu entri menjadi satu
//! baris sudah ada di sini juga. Kalau CLI dan GUI menyusun berkasnya sendiri-sendiri,
//! cepat atau lambat keduanya akan menghasilkan format yang berbeda.
//!
//! Isinya ditulis dalam bahasa yang sedang aktif. Entri log menyimpan kunci terjemahan,
//! bukan kalimat jadi, sehingga berkas yang sama bisa diekspor dalam bahasa apa pun tanpa
//! kehilangan apa pun.

use std::io::Write;
use std::path::Path;

use zydra_core::log::{unix_now, LogBuffer};

use crate::log::format_time;
use crate::Messages;

/// Menulis seluruh catatan ke sebuah berkas, satu entri per baris.
///
/// Berkas ditimpa kalau sudah ada. Mengembalikan kesalahan I/O apa adanya supaya pemanggil
/// bisa menampilkan sebabnya alih-alih berpura-pura berhasil.
pub fn write_log(log: &LogBuffer, messages: &Messages, path: &Path) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(render(log, messages).as_bytes())
}

/// Menyusun isi berkas sebagai teks, tanpa menyentuh sistem berkas.
///
/// Dipisah dari penulisannya supaya isinya bisa diperiksa tanpa membuat berkas.
pub fn render(log: &LogBuffer, messages: &Messages) -> String {
    let mut out = String::new();

    // Satu baris keterangan waktu ekspor. Tanpa itu, berkas catatan tidak bisa dipakai
    // untuk menelusuri kejadian karena tidak ada yang menyatakan kapan ia dibuat.
    out.push_str(&messages.text_args("log-export-header", &[("time", &format_time(unix_now()))]));
    out.push('\n');
    out.push('\n');

    for entry in log.entries() {
        out.push_str(&messages.log_line(entry));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Locale;
    use zydra_core::log::{LogEntry, LogLevel};

    fn buffer() -> LogBuffer {
        let mut log = LogBuffer::new(16);
        log.push(
            LogEntry::now(LogLevel::Info, "log-target-selected").with_arg("name", "vault.zip"),
        );
        log.push(LogEntry::now(LogLevel::Success, "result-granted"));
        log
    }

    #[test]
    fn every_entry_becomes_one_line() {
        let text = render(&buffer(), &Messages::new(Locale::En));
        let body: Vec<&str> = text.lines().skip(2).collect();
        assert_eq!(body.len(), 2);
        assert!(
            body[0].contains("vault.zip"),
            "unexpected line: {}",
            body[0]
        );
        assert!(
            body[1].contains("Password found"),
            "unexpected line: {}",
            body[1]
        );
    }

    #[test]
    fn the_exported_text_follows_the_active_language() {
        let english = render(&buffer(), &Messages::new(Locale::En));
        let indonesian = render(&buffer(), &Messages::new(Locale::Id));
        assert!(english.contains("Password found"), "{english}");
        assert!(indonesian.contains("Password ditemukan"), "{indonesian}");
    }

    #[test]
    fn the_header_states_when_the_export_was_made() {
        let text = render(&buffer(), &Messages::new(Locale::En));
        let header = text.lines().next().unwrap();
        // Waktu ditulis sebagai HH:MM:SS.
        assert!(
            header.contains(':') && header.len() > 20,
            "unexpected header: {header}"
        );
    }

    #[test]
    fn an_empty_log_still_produces_a_readable_file() {
        let text = render(&LogBuffer::new(4), &Messages::new(Locale::En));
        assert!(text.lines().count() >= 1);
    }

    #[test]
    fn writing_creates_a_file_that_can_be_read_back() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("activity.log");
        write_log(&buffer(), &Messages::new(Locale::En), &path).unwrap();

        let read_back = std::fs::read_to_string(&path).unwrap();
        assert!(read_back.contains("vault.zip"));
        assert!(read_back.contains("Password found"));
    }

    #[test]
    fn writing_over_an_existing_file_replaces_it() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("activity.log");
        std::fs::write(&path, b"old contents that should disappear").unwrap();

        write_log(&buffer(), &Messages::new(Locale::En), &path).unwrap();

        let read_back = std::fs::read_to_string(&path).unwrap();
        assert!(!read_back.contains("old contents"), "{read_back}");
    }

    #[test]
    fn writing_to_a_directory_that_does_not_exist_is_an_error() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("nowhere").join("activity.log");
        assert!(write_log(&buffer(), &Messages::new(Locale::En), &path).is_err());
    }
}
