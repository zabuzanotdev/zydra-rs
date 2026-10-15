//! Kartu: wadah dengan latar panel, garis tepi, dan sudut membulat.
//!
//! Hampir seluruh isi antarmuka duduk di dalam kartu. Karena itu bentuknya diambil dari
//! tema, bukan ditulis ulang di setiap pemakaian.

use egui::{Frame, Margin, Ui};

use crate::theme::Theme;

/// Menjalankan isi di dalam sebuah kartu.
pub fn card<R>(ui: &mut Ui, theme: &Theme, contents: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::default()
        .fill(theme.bg_panel)
        .stroke(theme.card_stroke())
        .corner_radius(theme.card_radius())
        .inner_margin(Margin::same(10))
        .show(ui, contents)
        .inner
}

/// Kartu dengan judul kecil di atasnya.
pub fn card_with_title<R>(
    ui: &mut Ui,
    theme: &Theme,
    title: &str,
    contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    card(ui, theme, |ui| {
        super::section_label(ui, theme, title);
        ui.add_space(theme.gap(1.0));
        contents(ui)
    })
}
