//! Kartu statistik dan pil keadaan.
//!
//! Keduanya menampilkan keadaan, bukan menerima masukan. Kartu statistik memakai angka
//! monospace supaya lebarnya tidak bergoyang saat nilainya berubah.

use egui::{Frame, Margin, RichText, Ui};

use crate::theme::Theme;

/// Satu angka beserta labelnya.
pub fn stat_tile(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    value: &str,
    colour: Option<egui::Color32>,
) {
    Frame::default()
        .fill(theme.bg_panel)
        .stroke(theme.card_stroke())
        .corner_radius(theme.control_radius())
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(label.to_uppercase())
                        .size(theme.type_scale.label)
                        .color(theme.text_muted),
                );
                ui.label(
                    RichText::new(value)
                        .size(theme.type_scale.body)
                        .monospace()
                        .color(colour.unwrap_or(theme.text)),
                );
            });
        });
}

/// Warna pil untuk tiap keadaan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PillTone {
    /// Sesuatu sedang berjalan.
    Running,
    /// Berhasil.
    Success,
    /// Gagal.
    Failure,
    /// Netral.
    Neutral,
}

impl PillTone {
    /// Warna yang dipakai pil ini pada sebuah tema.
    pub fn colour(self, theme: &Theme) -> egui::Color32 {
        match self {
            PillTone::Running => theme.accent,
            PillTone::Success => theme.success,
            PillTone::Failure => theme.error,
            PillTone::Neutral => theme.text_muted,
        }
    }
}

/// Pil keadaan: teks kecil dengan latar tembus pandang.
pub fn status_pill(ui: &mut Ui, theme: &Theme, tone: PillTone, text: &str) {
    let colour = tone.colour(theme);
    Frame::default()
        .fill(colour.gamma_multiply(0.15))
        .stroke(egui::Stroke::new(1.0_f32, colour.gamma_multiply(0.45)))
        .corner_radius(egui::CornerRadius::same(255))
        .inner_margin(Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(theme.type_scale.label)
                    .color(colour),
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_tone_maps_to_a_distinct_colour() {
        let theme = Theme::default();
        let colours = [
            PillTone::Running.colour(&theme),
            PillTone::Success.colour(&theme),
            PillTone::Failure.colour(&theme),
            PillTone::Neutral.colour(&theme),
        ];
        for (index, colour) in colours.iter().enumerate() {
            for other in colours.iter().skip(index + 1) {
                assert_ne!(colour, other, "two tones share a colour");
            }
        }
    }

    #[test]
    fn the_running_tone_uses_the_accent_colour() {
        let theme = Theme::default();
        assert_eq!(PillTone::Running.colour(&theme), theme.accent);
    }
}
