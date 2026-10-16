//! Panel pengaturan.
//!
//! Panel ini menyunting [`AppConfig`] dan menyimpannya saat pengguna menekan simpan. Ia
//! tidak menerapkan perubahan ke serangan yang sedang berjalan: mengubah jumlah worker di
//! tengah jalan akan membuat hasilnya sulit dijelaskan, dan pengguna yang mengubahnya
//! hampir selalu bermaksud untuk serangan berikutnya.

use egui::Ui;

use zydra_core::config::{AppConfig, LanguageChoice};
use zydra_i18n::{Locale, Messages};

use crate::responsive::LayoutMode;
use crate::theme::Theme;
use crate::widgets::{self, card, controls};

/// Aksi yang diminta pengguna dari panel pengaturan.
#[derive(Debug, Default, Clone, Copy)]
pub struct SettingsActions {
    /// Tombol simpan ditekan.
    pub save: bool,
    /// Tombol kembalikan ke bawaan ditekan.
    pub reset: bool,
}

/// Menggambar panel pengaturan.
pub fn settings_panel(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    config: &mut AppConfig,
    layout: LayoutMode,
) -> SettingsActions {
    let mut actions = SettingsActions::default();
    let full_width = layout.buttons_are_full_width();

    widgets::section_label(ui, theme, &messages.text("settings-title"));
    ui.add_space(theme.gap(1.0));

    card::card(ui, theme, |ui| {
        language_row(ui, theme, messages, config);
        ui.add_space(theme.gap(2.0));
        workers_row(ui, theme, messages, config);
        ui.add_space(theme.gap(2.0));
        number_row(
            ui,
            theme,
            messages,
            "settings-max-combinations",
            &mut config.max_combinations,
            1..=u64::MAX,
        );
        ui.add_space(theme.gap(2.0));
        number_row(
            ui,
            theme,
            messages,
            "settings-max-brute-length",
            &mut config.max_brute_length,
            1..=64,
        );
    });

    ui.add_space(theme.gap(2.0));
    actions.save =
        controls::primary_button(ui, theme, &messages.text("settings-save"), true, full_width);
    ui.add_space(theme.gap(1.5));
    actions.reset = controls::secondary_button(
        ui,
        theme,
        &messages.text("settings-reset"),
        true,
        full_width,
    );

    actions
}

fn language_row(ui: &mut Ui, theme: &Theme, messages: &Messages, config: &mut AppConfig) {
    widgets::muted(ui, theme, &messages.text("settings-language"));
    ui.add_space(theme.gap(1.0));
    let options = [
        (
            LanguageChoice::Auto,
            messages.text("settings-language-auto"),
        ),
        (LanguageChoice::En, Locale::En.code().to_string()),
        (LanguageChoice::Id, Locale::Id.code().to_string()),
    ];
    if let Some(chosen) = controls::segmented(ui, theme, &options, config.language) {
        config.language = chosen;
    }
}

fn workers_row(ui: &mut Ui, theme: &Theme, messages: &Messages, config: &mut AppConfig) {
    ui.horizontal(|ui| {
        widgets::muted(ui, theme, &messages.text("settings-workers"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::DragValue::new(&mut config.workers).range(0..=256));
        });
    });
    // Nol bukan kesalahan, jadi dijelaskan alih-alih ditolak.
    if config.workers == 0 {
        widgets::muted(ui, theme, &messages.text("settings-workers-auto"));
    }
}

/// Satu baris angka: label di kiri, nilai di kanan.
fn number_row(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    key: &str,
    value: &mut impl egui::emath::Numeric,
    range: std::ops::RangeInclusive<impl egui::emath::Numeric>,
) {
    ui.horizontal(|ui| {
        widgets::muted(ui, theme, &messages.text(key));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::DragValue::new(value).range(range));
        });
    });
}
