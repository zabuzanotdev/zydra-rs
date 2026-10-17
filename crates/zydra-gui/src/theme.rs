//! Token visual.
//!
//! Seluruh warna, sudut, dan jarak hidup di sini. Tidak ada panel atau widget yang boleh
//! menulis nilai warna langsung: kalau ada, mengubah tema berarti berburu ke seluruh
//! proyek, dan cepat atau lambat akan ada satu tempat yang tertinggal.
//!
//! Palet ini adalah arah "Cyber Industrial" yang disetujui: latar gelap kebiruan, panel
//! berkartu dengan sudut membulat, aksen cyan, dan angka dalam monospace.

use egui::{Color32, Context, CornerRadius, Stroke, Visuals};

/// Ukuran huruf yang dipakai antarmuka.
#[derive(Debug, Clone, Copy)]
pub struct TypeScale {
    /// Angka persentase besar.
    pub display: f32,
    /// Judul bagian.
    pub heading: f32,
    /// Teks biasa.
    pub body: f32,
    /// Label kecil huruf besar.
    pub label: f32,
    /// Konsol log dan angka monospace.
    pub mono: f32,
}

/// Ketebalan garis tepi, dalam piksel logis.
pub const HAIRLINE: f32 = 1.0;

/// Seluruh nilai visual aplikasi.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Latar jendela.
    pub bg_app: Color32,
    /// Latar panel dan kartu.
    pub bg_panel: Color32,
    /// Latar konsol log.
    pub bg_console: Color32,
    /// Latar jalur bilah progres.
    pub bg_track: Color32,
    /// Garis tepi kartu.
    pub border: Color32,
    /// Garis tepi kontrol yang bisa ditekan.
    pub border_strong: Color32,
    /// Warna teks utama.
    pub text: Color32,
    /// Warna teks sekunder.
    pub text_muted: Color32,
    /// Warna aksen.
    pub accent: Color32,
    /// Warna keberhasilan.
    pub success: Color32,
    /// Warna peringatan.
    pub warning: Color32,
    /// Warna kegagalan.
    pub error: Color32,
    /// Sudut kartu.
    pub radius_card: u8,
    /// Sudut kontrol.
    pub radius_control: u8,
    /// Satuan jarak.
    pub space_unit: f32,
    /// Skala huruf.
    pub type_scale: TypeScale,
}

impl Default for Theme {
    fn default() -> Theme {
        Theme {
            bg_app: Color32::from_rgb(0x0b, 0x0f, 0x14),
            bg_panel: Color32::from_rgb(0x11, 0x18, 0x23),
            bg_console: Color32::from_rgb(0x08, 0x0c, 0x11),
            bg_track: Color32::from_rgb(0x1a, 0x25, 0x31),
            border: Color32::from_rgb(0x1e, 0x2b, 0x3a),
            border_strong: Color32::from_rgb(0x2a, 0x3b, 0x4f),
            text: Color32::from_rgb(0xe6, 0xed, 0xf3),
            text_muted: Color32::from_rgb(0x8b, 0x9b, 0xb0),
            accent: Color32::from_rgb(0x22, 0xd3, 0xee),
            success: Color32::from_rgb(0x4a, 0xde, 0x80),
            warning: Color32::from_rgb(0xfb, 0xbf, 0x24),
            error: Color32::from_rgb(0xf8, 0x71, 0x71),
            radius_card: 8,
            radius_control: 6,
            space_unit: 4.0,
            type_scale: TypeScale {
                display: 30.0,
                heading: 15.0,
                body: 13.0,
                label: 10.0,
                mono: 12.0,
            },
        }
    }
}

impl Theme {
    /// Jarak kelipatan satuan.
    pub fn gap(&self, units: f32) -> f32 {
        self.space_unit * units
    }

    /// Sudut kartu dalam bentuk yang dipakai egui.
    pub fn card_radius(&self) -> CornerRadius {
        CornerRadius::same(self.radius_card)
    }

    /// Sudut kontrol dalam bentuk yang dipakai egui.
    pub fn control_radius(&self) -> CornerRadius {
        CornerRadius::same(self.radius_control)
    }

    /// Garis tepi kartu.
    pub fn card_stroke(&self) -> Stroke {
        Stroke::new(HAIRLINE, self.border)
    }

    /// Menerapkan tema ke konteks egui.
    pub fn apply(&self, context: &Context) {
        let mut visuals = Visuals::dark();
        visuals.panel_fill = self.bg_app;
        visuals.window_fill = self.bg_panel;
        visuals.extreme_bg_color = self.bg_console;
        visuals.faint_bg_color = self.bg_track;
        visuals.code_bg_color = self.bg_console;
        visuals.text_edit_bg_color = Some(self.bg_console);
        visuals.warn_fg_color = self.warning;
        visuals.error_fg_color = self.error;
        visuals.override_text_color = Some(self.text);
        visuals.window_stroke = self.card_stroke();
        visuals.window_corner_radius = self.control_radius();
        visuals.menu_corner_radius = self.control_radius();
        visuals.window_shadow = egui::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(80),
        };
        visuals.popup_shadow = egui::Shadow {
            offset: [0, 4],
            blur: 12,
            spread: 0,
            color: Color32::from_black_alpha(96),
        };
        visuals.button_frame = true;
        visuals.collapsing_header_frame = false;
        visuals.indent_has_left_vline = false;
        visuals.striped = false;
        visuals.selection.bg_fill = self.accent.gamma_multiply(0.35);
        visuals.selection.stroke = Stroke::new(HAIRLINE, self.accent);
        visuals.hyperlink_color = self.accent;

        let widgets = &mut visuals.widgets;
        widgets.noninteractive.bg_fill = self.bg_panel;
        widgets.noninteractive.bg_stroke = self.card_stroke();
        widgets.noninteractive.fg_stroke = Stroke::new(HAIRLINE, self.text_muted);
        widgets.noninteractive.corner_radius = self.control_radius();

        widgets.inactive.bg_fill = self.bg_panel;
        widgets.inactive.bg_stroke = Stroke::new(HAIRLINE, self.border_strong);
        widgets.inactive.fg_stroke = Stroke::new(HAIRLINE, self.text);
        widgets.inactive.corner_radius = self.control_radius();

        widgets.hovered.bg_fill = self.bg_track;
        widgets.hovered.bg_stroke = Stroke::new(HAIRLINE, self.accent);
        widgets.hovered.fg_stroke = Stroke::new(HAIRLINE, self.text);
        widgets.hovered.corner_radius = self.control_radius();

        widgets.active.bg_fill = self.accent.gamma_multiply(0.30);
        widgets.active.bg_stroke = Stroke::new(HAIRLINE, self.accent);
        widgets.active.fg_stroke = Stroke::new(HAIRLINE, self.text);
        widgets.active.corner_radius = self.control_radius();

        widgets.open.bg_fill = self.bg_track;
        widgets.open.bg_stroke = Stroke::new(HAIRLINE, self.border_strong);
        widgets.open.fg_stroke = Stroke::new(HAIRLINE, self.text);
        widgets.open.corner_radius = self.control_radius();

        context.set_visuals(visuals);
    }

    /// Menerapkan jarak antar elemen.
    pub fn apply_spacing(&self, context: &Context) {
        let mut style = (*context.style()).clone();
        style.spacing.item_spacing = egui::vec2(self.gap(2.0), self.gap(1.5));
        style.spacing.button_padding = egui::vec2(self.gap(2.5), self.gap(1.5));
        style.spacing.window_margin = egui::Margin::same(8);
        context.set_style(style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_palette_matches_the_approved_design() {
        let theme = Theme::default();
        assert_eq!(theme.bg_app, Color32::from_rgb(0x0b, 0x0f, 0x14));
        assert_eq!(theme.bg_panel, Color32::from_rgb(0x11, 0x18, 0x23));
        assert_eq!(theme.accent, Color32::from_rgb(0x22, 0xd3, 0xee));
        assert_eq!(theme.success, Color32::from_rgb(0x4a, 0xde, 0x80));
    }

    #[test]
    fn the_console_is_darker_than_the_panel_it_sits_in() {
        let theme = Theme::default();
        let console = theme.bg_console.r() as u16 + theme.bg_console.g() as u16;
        let panel = theme.bg_panel.r() as u16 + theme.bg_panel.g() as u16;
        assert!(console < panel, "the console should read as a recess");
    }

    #[test]
    fn the_type_scale_descends_from_display_to_label() {
        let scale = Theme::default().type_scale;
        assert!(scale.display > scale.heading);
        assert!(scale.heading > scale.body);
        assert!(scale.body > scale.label);
    }

    #[test]
    fn gaps_are_multiples_of_the_base_unit() {
        let theme = Theme::default();
        assert_eq!(theme.gap(1.0), theme.space_unit);
        assert_eq!(theme.gap(4.0), theme.space_unit * 4.0);
    }

    #[test]
    fn corner_radii_are_whole_pixels() {
        let theme = Theme::default();
        assert_eq!(theme.card_radius(), CornerRadius::same(8));
        assert_eq!(theme.control_radius(), CornerRadius::same(6));
    }
}
