//! Bilah progres.

use egui::{CornerRadius, Ui};

use crate::theme::Theme;

/// Menggambar bilah progres untuk bagian pekerjaan yang sudah selesai.
///
/// Nilai dijepit di sini juga, bukan hanya di mesin. Antarmuka tidak boleh menggambar
/// bilah yang melewati batasnya sendiri hanya karena sebuah angka datang salah.
///
/// Digambar manual (jalur + isi) supaya memakai token tema (`bg_track` untuk jalur,
/// `accent` untuk isi), bukan warna bawaan egui yang membuat bilahnya terlihat mentah.
pub fn progress_bar(ui: &mut Ui, theme: &Theme, fraction: f32, height: f32) {
    let fraction = if fraction.is_nan() {
        0.0
    } else {
        fraction.clamp(0.0, 1.0)
    };
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let radius = CornerRadius::same((height / 2.0) as u8);
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, radius, theme.bg_track);
    painter.rect_stroke(rect, radius, theme.card_stroke(), egui::StrokeKind::Inside);
    if fraction > 0.0 {
        let fill_width = (rect.width() * fraction).max(height);
        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, height));
        painter.rect_filled(fill_rect, radius, theme.accent);
    }
}

/// Bilah jalur kosong, dipakai saat belum ada pekerjaan.
pub fn empty_track(ui: &mut Ui, theme: &Theme, height: f32) {
    progress_bar(ui, theme, 0.0, height);
}

#[cfg(test)]
mod tests {
    /// Nilai yang dijepit harus tetap berada di rentang yang bisa digambar.
    #[test]
    fn the_clamp_used_by_the_bar_holds_at_the_edges() {
        let clamp = |value: f32| {
            if value.is_nan() {
                0.0
            } else {
                value.clamp(0.0, 1.0)
            }
        };
        assert_eq!(clamp(-5.0), 0.0);
        assert_eq!(clamp(0.0), 0.0);
        assert_eq!(clamp(0.5), 0.5);
        assert_eq!(clamp(1.0), 1.0);
        assert_eq!(clamp(9.0), 1.0);
        assert_eq!(clamp(f32::NAN), 0.0);
    }
}
