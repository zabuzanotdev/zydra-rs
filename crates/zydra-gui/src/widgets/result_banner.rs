//! Spanduk hasil: satu baris mencolok yang menyatakan hasil akhir.

use egui::{Frame, Margin, RichText, Ui};

use crate::theme::Theme;
use crate::widgets::stat_tile::PillTone;

/// Menggambar spanduk hasil.
///
/// Mengembalikan `true` kalau tombol salin ditekan.
pub fn result_banner(
    ui: &mut Ui,
    theme: &Theme,
    tone: PillTone,
    title: &str,
    value: &str,
    action: Option<&str>,
    full_width: bool,
) -> bool {
    let colour = tone.colour(theme);
    let mut clicked = false;

    Frame::default()
        .fill(colour.gamma_multiply(0.12))
        .stroke(egui::Stroke::new(1.0_f32, colour.gamma_multiply(0.45)))
        .corner_radius(theme.card_radius())
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(title)
                        .size(theme.type_scale.body)
                        .color(colour),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(action) = action {
                        if ui.button(action).clicked() {
                            clicked = true;
                        }
                    }
                    ui.label(
                        RichText::new(value)
                            .size(theme.type_scale.body)
                            .monospace()
                            .color(theme.text),
                    );
                });
            });

            // Di jendela sempit, tombol pindah ke baris sendiri supaya nilainya tidak
            // terpotong.
            if full_width {
                if let Some(action) = action {
                    ui.add_space(theme.gap(1.5));
                    if ui
                        .add_sized(
                            egui::Vec2::new(ui.available_width(), 28.0),
                            egui::Button::new(action),
                        )
                        .clicked()
                    {
                        clicked = true;
                    }
                }
            }
        });

    clicked
}
