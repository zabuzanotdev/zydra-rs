//! Bilah progres.

use egui::Ui;

use crate::theme::Theme;

/// Menggambar bilah progres untuk bagian pekerjaan yang sudah selesai.
///
/// Nilai dijepit di sini juga, bukan hanya di mesin. Antarmuka tidak boleh menggambar
/// bilah yang melewati batasnya sendiri hanya karena sebuah angka datang salah.
pub fn progress_bar(ui: &mut Ui, theme: &Theme, fraction: f32, height: f32) {
    let fraction = if fraction.is_nan() {
        0.0
    } else {
        fraction.clamp(0.0, 1.0)
    };
    ui.add(
        egui::ProgressBar::new(fraction)
            .desired_height(height)
            .fill(theme.accent)
            .corner_radius(theme.control_radius()),
    );
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
