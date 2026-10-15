//! Widget yang dipakai berulang di seluruh antarmuka.
//!
//! Setiap widget adalah fungsi yang menerima data dan tidak memegang keadaan aplikasi.
//! Tidak ada widget yang membaca konfigurasi atau memanggil mesin; itu pekerjaan
//! pengendali. Aturan itu yang membuat tampilan bisa diubah tanpa menyentuh logika.

pub mod card;
pub mod controls;
pub mod log_console;
pub mod progress_bar;
pub mod result_banner;
pub mod stat_tile;

use egui::{RichText, Ui};

use crate::theme::Theme;

/// Label kecil huruf besar, dipakai sebagai judul bagian.
pub fn section_label(ui: &mut Ui, theme: &Theme, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(theme.type_scale.label)
            .color(theme.text_muted),
    );
}

/// Teks biasa.
pub fn body(ui: &mut Ui, theme: &Theme, text: &str) {
    ui.label(
        RichText::new(text)
            .size(theme.type_scale.body)
            .color(theme.text),
    );
}

/// Teks sekunder.
pub fn muted(ui: &mut Ui, theme: &Theme, text: &str) {
    ui.label(
        RichText::new(text)
            .size(theme.type_scale.body)
            .color(theme.text_muted),
    );
}

/// Teks monospace. Dipakai untuk angka, nama berkas, dan password.
///
/// Angka selalu monospace supaya lebarnya tidak berubah saat nilainya berubah; tanpa itu
/// seluruh baris akan bergoyang setiap kali kemajuan diperbarui.
pub fn mono(ui: &mut Ui, theme: &Theme, text: &str) {
    ui.label(
        RichText::new(text)
            .size(theme.type_scale.mono)
            .monospace()
            .color(theme.text),
    );
}

/// Teks monospace dengan warna tertentu.
pub fn mono_coloured(ui: &mut Ui, theme: &Theme, text: &str, colour: egui::Color32) {
    ui.label(
        RichText::new(text)
            .size(theme.type_scale.mono)
            .monospace()
            .color(colour),
    );
}

/// Menyingkat teks yang terlalu panjang untuk ruang yang tersedia.
pub fn truncate(text: &str, max_characters: usize) -> String {
    if text.chars().count() <= max_characters {
        return text.to_string();
    }
    let keep = max_characters.saturating_sub(1);
    let mut shortened: String = text.chars().take(keep).collect();
    shortened.push('…');
    shortened
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_is_left_alone() {
        assert_eq!(truncate("vault.zip", 20), "vault.zip");
        assert_eq!(truncate("exactly-ten", 11), "exactly-ten");
    }

    #[test]
    fn long_text_is_shortened_with_an_ellipsis() {
        assert_eq!(truncate("abcdefghij", 5), "abcd…");
    }

    #[test]
    fn truncation_counts_characters_not_bytes() {
        // Memotong berdasarkan byte akan membelah karakter multibyte dan menghasilkan
        // teks rusak, jadi ini diuji langsung.
        assert_eq!(truncate("rahasia-rahasia", 8), "rahasia…");
        assert_eq!(truncate("日本語のファイル", 4), "日本語…");
    }

    #[test]
    fn a_zero_limit_does_not_panic() {
        assert_eq!(truncate("anything", 0), "…");
    }
}
