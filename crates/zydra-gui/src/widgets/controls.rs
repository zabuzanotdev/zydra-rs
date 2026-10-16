//! Kontrol masukan: pemilih bertingkat, baris berkas, dan tombol aksi.

use egui::{Frame, Margin, RichText, Ui, Vec2};

use crate::theme::Theme;

/// Pemilih bertingkat: sederet pilihan yang saling meniadakan.
///
/// Mengembalikan pilihan yang baru ditekan, atau `None` kalau tidak ada yang berubah.
pub fn segmented<T: PartialEq + Copy>(
    ui: &mut Ui,
    theme: &Theme,
    options: &[(T, String)],
    selected: T,
) -> Option<T> {
    let mut chosen = None;
    Frame::default()
        .fill(theme.bg_panel)
        .stroke(theme.card_stroke())
        .corner_radius(theme.control_radius())
        .inner_margin(Margin::same(2))
        .show(ui, |ui| {
            // Membungkus, bukan memanjang: tiga pilihan tidak muat dalam satu baris di
            // sidebar selebar 240 piksel, dan memaksanya akan memotong label atau
            // meluberkan panelnya.
            ui.horizontal_wrapped(|ui| {
                for (value, label) in options {
                    let is_selected = *value == selected;
                    let text = RichText::new(label).size(theme.type_scale.body);
                    if ui.selectable_label(is_selected, text).clicked() && !is_selected {
                        chosen = Some(*value);
                    }
                }
            });
        });
    chosen
}

/// Baris pemilih berkas.
///
/// Mengembalikan `true` kalau tombol pilih ditekan.
pub fn file_row(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    value: Option<&str>,
    empty_text: &str,
    browse_text: &str,
    full_width: bool,
) -> bool {
    let mut clicked = false;
    super::section_label(ui, theme, label);
    ui.add_space(theme.gap(1.0));
    Frame::default()
        .fill(theme.bg_panel)
        .stroke(theme.card_stroke())
        .corner_radius(theme.control_radius())
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                match value {
                    Some(name) => super::mono(ui, theme, super::truncate(name, 26).as_str()),
                    None => super::muted(ui, theme, empty_text),
                }
                // Tombol pilih selalu di kanan; saat lebar penuh ia pindah ke bawah.
                if !full_width {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        clicked = ui.button(browse_text).clicked();
                    });
                }
            });
            if full_width {
                ui.add_space(theme.gap(1.0));
                clicked = ui
                    .add_sized(
                        Vec2::new(ui.available_width(), 26.0),
                        egui::Button::new(browse_text),
                    )
                    .clicked();
            }
        });
    clicked
}

/// Tombol aksi utama.
pub fn primary_button(
    ui: &mut Ui,
    theme: &Theme,
    text: &str,
    enabled: bool,
    full_width: bool,
) -> bool {
    let button = egui::Button::new(
        RichText::new(text)
            .size(theme.type_scale.body)
            .color(theme.bg_app),
    )
    .fill(theme.accent)
    .corner_radius(theme.control_radius());

    if full_width {
        ui.add_enabled_ui(enabled, |ui| {
            ui.add_sized(Vec2::new(ui.available_width(), 32.0), button)
                .clicked()
        })
        .inner
    } else {
        ui.add_enabled(enabled, button).clicked()
    }
}

/// Tombol aksi kedua.
pub fn secondary_button(
    ui: &mut Ui,
    theme: &Theme,
    text: &str,
    enabled: bool,
    full_width: bool,
) -> bool {
    let button = egui::Button::new(RichText::new(text).size(theme.type_scale.body))
        .fill(theme.bg_panel)
        .corner_radius(theme.control_radius());

    if full_width {
        ui.add_enabled_ui(enabled, |ui| {
            ui.add_sized(Vec2::new(ui.available_width(), 30.0), button)
                .clicked()
        })
        .inner
    } else {
        ui.add_enabled(enabled, button).clicked()
    }
}
