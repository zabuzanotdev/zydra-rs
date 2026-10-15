//! Menggambar keluaran terminal.
//!
//! Seluruh kata berasal dari katalog terjemahan; modul ini hanya mengurus tata letak dan
//! warna. Mode `--quiet` dan `--json` mematikan warna serta bilah progres sekaligus.

use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::{OwoColorize, Stream};

use zydra_core::log::LogEntry;
use zydra_core::model::failure::CrackFailure;
use zydra_core::model::progress::CrackProgress;
use zydra_i18n::Messages;

use crate::json;

/// Template bilah progres. Hanya berisi penanda, bukan kata, supaya tidak ada teks yang
/// perlu diterjemahkan di sini.
const BAR_TEMPLATE: &str = "{bar:40} {percent} {msg}";

/// Penyaji keluaran untuk satu kali jalan.
pub struct Reporter {
    messages: Messages,
    plain: bool,
    json: bool,
}

impl Reporter {
    /// Membuat penyaji. `plain` mematikan warna dan bilah, `json` menggantinya dengan
    /// keluaran terstruktur.
    pub fn new(messages: Messages, quiet: bool, json: bool) -> Reporter {
        Reporter {
            messages,
            plain: quiet || json,
            json,
        }
    }

    /// Apakah keluaran terstruktur diminta.
    pub fn is_json(&self) -> bool {
        self.json
    }

    /// Katalog terjemahan yang dipakai.
    pub fn messages(&self) -> &Messages {
        &self.messages
    }

    /// Mencetak satu baris log.
    pub fn log(&self, entry: &LogEntry) {
        if self.json {
            return;
        }
        println!("{}", self.messages.log_line(entry));
    }

    /// Mencetak baris peristiwa terstruktur.
    pub fn json_line(&self, fields: &[(&str, String)]) {
        println!("{}", json::object(fields));
    }

    /// Membuat bilah progres, atau `None` kalau keluarannya polos.
    pub fn progress_bar(&self, total: u64) -> Option<ProgressBar> {
        if self.plain {
            return None;
        }
        let bar = ProgressBar::new(total);
        let style = ProgressStyle::with_template(BAR_TEMPLATE)
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("##-");
        bar.set_style(style);
        Some(bar)
    }

    /// Memperbarui bilah progres dari sebuah langkah kemajuan.
    pub fn progress(&self, bar: &ProgressBar, progress: &CrackProgress) {
        if let CrackProgress::Trying(data) = progress {
            bar.set_position(data.tried.min(data.total));
            bar.set_message(self.messages.progress(progress));
        }
    }

    /// Mencetak ringkasan sebelum pekerjaan dimulai.
    ///
    /// Pasangan yang diterima adalah kunci terjemahan dan nilainya, bukan label siap
    /// pakai. Dengan begitu tidak mungkin ada kunci mentah yang bocor ke layar karena
    /// lupa menerjemahkan.
    pub fn header(&self, rows: &[(&str, String)]) {
        if self.json {
            return;
        }
        let labelled: Vec<(String, &String)> = rows
            .iter()
            .map(|(key, value)| (self.messages.text(key), value))
            .collect();
        let width = labelled
            .iter()
            .map(|(label, _)| label.chars().count())
            .max()
            .unwrap_or(0);
        for (label, value) in labelled {
            println!(
                "{}  {}",
                self.accent(&format!("{label:<width$}")),
                self.bold(value)
            );
        }
    }

    /// Mencetak hasil akhir.
    pub fn success(&self, password: &str, detail: Option<&str>) {
        println!("{}", self.ok(&self.messages.text("result-granted")));
        println!(
            "{}  {}",
            self.messages.text("result-key"),
            self.bold(password)
        );
        if let Some(detail) = detail {
            println!("{}  {detail}", self.messages.text("result-detail"));
        }
    }

    /// Mencetak bahwa tidak ada kandidat yang cocok.
    pub fn not_found(&self) {
        println!("{}", self.bad(&self.messages.text("result-denied")));
    }

    /// Mencetak sebuah kegagalan.
    pub fn failure(&self, failure: &CrackFailure) {
        eprintln!("{}", self.bad(&self.messages.failure(failure)));
    }

    /// Mencetak durasi total.
    pub fn footer(&self, elapsed: Duration) {
        if self.json {
            return;
        }
        println!(
            "{}  {}",
            self.accent(&self.messages.text("cli-elapsed")),
            crate::render::format_duration(elapsed)
        );
    }

    fn accent(&self, text: &str) -> String {
        if self.plain {
            return text.to_string();
        }
        text.if_supports_color(Stream::Stdout, |text| text.cyan())
            .to_string()
    }

    fn bold(&self, text: &str) -> String {
        if self.plain {
            return text.to_string();
        }
        text.if_supports_color(Stream::Stdout, |text| text.bold())
            .to_string()
    }

    fn ok(&self, text: &str) -> String {
        if self.plain {
            return text.to_string();
        }
        // Satu gaya per panggilan: merangkai dua gaya akan meminjam nilai antara yang
        // dibuat di dalam closure, dan itu tidak boleh keluar dari closure.
        text.if_supports_color(Stream::Stdout, |text| text.green())
            .to_string()
    }

    fn bad(&self, text: &str) -> String {
        if self.plain {
            return text.to_string();
        }
        text.if_supports_color(Stream::Stdout, |text| text.red())
            .to_string()
    }
}

/// Durasi sebagai `HH:MM:SS`, atau `MM:SS` kalau di bawah satu jam.
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_under_an_hour_omit_the_hour_field() {
        assert_eq!(format_duration(Duration::from_secs(0)), "00:00");
        assert_eq!(format_duration(Duration::from_secs(59)), "00:59");
        assert_eq!(format_duration(Duration::from_secs(3_599)), "59:59");
    }

    #[test]
    fn durations_over_an_hour_include_it() {
        assert_eq!(format_duration(Duration::from_secs(3_600)), "01:00:00");
    }
}
