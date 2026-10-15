//! Panel pilihan di sisi kiri.
//!
//! Panel ini hanya menyunting [`UiSettings`]. Ia tidak menyiapkan serangan, tidak
//! memanggil mesin, dan tidak menyimpan keadaan sendiri; semua keputusan ada di
//! `state.rs`, dan aksi yang diklik dikembalikan ke pemanggil.

use egui::Ui;

use zydra_core::model::charset::CharsetSet;
use zydra_i18n::{format, Messages};

use crate::responsive::LayoutMode;
use crate::state::{Method, PlanProblem, UiSettings};
use crate::theme::Theme;
use crate::widgets::{self, card, controls, stat_tile::PillTone};

/// Aksi yang diminta pengguna dari panel ini.
#[derive(Debug, Default, Clone, Copy)]
pub struct SidebarActions {
    /// Tombol mulai ditekan.
    pub start: bool,
    /// Tombol hentikan ditekan.
    pub stop: bool,
    /// Tombol mulai ulang ditekan.
    pub reset: bool,
    /// Tombol lanjutkan ditekan.
    pub resume: bool,
    /// Tombol pilih berkas target ditekan.
    pub browse_target: bool,
    /// Tombol pilih wordlist ditekan.
    pub browse_wordlist: bool,
}

/// Kelas karakter beserta kunci terjemahannya.
const CHARSET_KEYS: [(CharsetSet, &str); 5] = [
    (CharsetSet::LOWERCASE, "charset-lowercase"),
    (CharsetSet::UPPERCASE, "charset-uppercase"),
    (CharsetSet::DIGITS, "charset-digits"),
    (CharsetSet::SYMBOLS, "charset-symbols"),
    (CharsetSet::SPACE, "charset-space"),
];

/// Menggambar panel pilihan.
#[allow(clippy::too_many_arguments)]
pub fn sidebar(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    settings: &mut UiSettings,
    layout: LayoutMode,
    running: bool,
    finished: bool,
    resume_available: bool,
) -> SidebarActions {
    let mut actions = SidebarActions::default();
    let full_width = layout.buttons_are_full_width();
    let editable = !running;

    ui.add_enabled_ui(editable, |ui| {
        target_section(ui, theme, messages, settings, &mut actions, full_width);
        ui.add_space(theme.gap(3.0));
        method_section(ui, theme, messages, settings);
        ui.add_space(theme.gap(3.0));
        match settings.method {
            Method::Dictionary => {
                wordlist_section(ui, theme, messages, settings, &mut actions, full_width)
            }
            Method::BruteForce => brute_force_section(ui, theme, messages, settings),
            Method::Mask => mask_section(ui, theme, messages, settings),
        }
    });

    // Aturan berlaku untuk metode apa pun, jadi bagiannya berdiri sendiri dan tidak
    // menghilang saat metodenya berganti.
    ui.add_enabled_ui(editable, |ui| {
        ui.add_space(theme.gap(3.0));
        rules_section(ui, theme, messages, settings);
        plan_problem(ui, theme, messages, settings);
    });

    ui.add_space(theme.gap(4.0));
    action_section(
        ui,
        theme,
        messages,
        settings,
        &mut actions,
        running,
        finished,
        resume_available,
        full_width,
    );

    actions
}

fn target_section(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    settings: &mut UiSettings,
    actions: &mut SidebarActions,
    full_width: bool,
) {
    let display = settings
        .target
        .as_ref()
        .map(|target| target.display_name.as_str());
    if controls::file_row(
        ui,
        theme,
        &messages.text("sidebar-target"),
        display,
        &messages.text("sidebar-target-empty"),
        &messages.text("action-browse"),
        full_width,
    ) {
        actions.browse_target = true;
    }
}

fn method_section(ui: &mut Ui, theme: &Theme, messages: &Messages, settings: &mut UiSettings) {
    widgets::section_label(ui, theme, &messages.text("sidebar-method"));
    ui.add_space(theme.gap(1.0));
    let options = [
        (
            Method::Dictionary,
            messages.text("sidebar-method-dictionary"),
        ),
        (
            Method::BruteForce,
            messages.text("sidebar-method-brute-force"),
        ),
        (Method::Mask, messages.text("sidebar-method-mask")),
    ];
    if let Some(chosen) = controls::segmented(ui, theme, &options, settings.method) {
        settings.method = chosen;
    }
}

fn mask_section(ui: &mut Ui, theme: &Theme, messages: &Messages, settings: &mut UiSettings) {
    widgets::section_label(ui, theme, &messages.text("sidebar-mask"));
    ui.add_space(theme.gap(1.0));
    ui.add(
        egui::TextEdit::singleline(&mut settings.mask)
            .hint_text(messages.text("sidebar-mask-hint"))
            .desired_width(f32::INFINITY),
    );
}

fn rules_section(ui: &mut Ui, theme: &Theme, messages: &Messages, settings: &mut UiSettings) {
    ui.checkbox(
        &mut settings.use_rules,
        messages.text("sidebar-rules-enable"),
    );
    ui.add_enabled_ui(settings.use_rules, |ui| {
        ui.add_space(theme.gap(1.0));
        ui.add(
            egui::TextEdit::singleline(&mut settings.rules)
                .hint_text(messages.text("sidebar-rules-hint"))
                .desired_width(f32::INFINITY),
        );
    });
}

fn wordlist_section(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    settings: &mut UiSettings,
    actions: &mut SidebarActions,
    full_width: bool,
) {
    let display = settings
        .wordlist
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());
    if controls::file_row(
        ui,
        theme,
        &messages.text("sidebar-wordlist"),
        display.as_deref(),
        &messages.text("sidebar-wordlist-empty"),
        &messages.text("action-browse"),
        full_width,
    ) {
        actions.browse_wordlist = true;
    }
    if let Some(lines) = settings.wordlist_lines {
        ui.add_space(theme.gap(1.0));
        widgets::muted(ui, theme, &format::count(lines));
    }
}

fn brute_force_section(ui: &mut Ui, theme: &Theme, messages: &Messages, settings: &mut UiSettings) {
    widgets::section_label(ui, theme, &messages.text("sidebar-charset"));
    ui.add_space(theme.gap(1.0));
    card::card(ui, theme, |ui| {
        for (flag, key) in CHARSET_KEYS {
            let mut enabled = settings.charset.contains(flag);
            if ui.checkbox(&mut enabled, messages.text(key)).changed() {
                if enabled {
                    settings.charset |= flag;
                } else {
                    settings.charset.remove(flag);
                }
            }
        }
    });

    ui.add_space(theme.gap(3.0));
    ui.horizontal(|ui| {
        ui.label(messages.text("sidebar-min-length"));
        ui.add(egui::DragValue::new(&mut settings.min_length).range(1..=32));
        ui.label(messages.text("sidebar-max-length"));
        ui.add(egui::DragValue::new(&mut settings.max_length).range(1..=32));
    });

    ui.add_space(theme.gap(1.5));
    let estimate = settings
        .estimate()
        .map(format::count)
        .unwrap_or_else(|| messages.text("stat-value-unknown"));
    widgets::muted(
        ui,
        theme,
        &format!("{}: {estimate}", messages.text("sidebar-combinations")),
    );
}

/// Menjelaskan kenapa rencananya belum bisa dijalankan.
///
/// Dipanggil untuk semua metode, bukan hanya brute force, supaya pilihan yang belum
/// lengkap dijelaskan sebelum tombol mulai ditekan alih-alih setelah gagal.
fn plan_problem(ui: &mut Ui, theme: &Theme, messages: &Messages, settings: &UiSettings) {
    let Err(problem) = settings.plan() else {
        return;
    };
    let key = match problem {
        // Kekurangan ini sudah terlihat dari panelnya sendiri: berkas yang belum dipilih
        // tampil sebagai baris kosong. Menambahkan peringatan hanya menambah kebisingan.
        PlanProblem::MissingTarget | PlanProblem::MissingWordlist => return,
        PlanProblem::NoCharset => "error-no-charset",
        PlanProblem::BadRange => "usage-bad-length",
        PlanProblem::EmptyMask => "sidebar-mask-empty",
    };
    ui.add_space(theme.gap(2.0));
    ui.colored_label(theme.error, messages.text(key));
}

#[allow(clippy::too_many_arguments)]
fn action_section(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    settings: &UiSettings,
    actions: &mut SidebarActions,
    running: bool,
    finished: bool,
    resume_available: bool,
    full_width: bool,
) {
    if running {
        actions.stop =
            controls::secondary_button(ui, theme, &messages.text("action-stop"), true, full_width);
        return;
    }

    // Tombol melanjutkan hanya muncul kalau memang ada yang bisa dilanjutkan. Menampilkannya
    // selalu akan membuat pengguna menebak-nebak apakah serangannya tersimpan.
    if resume_available {
        actions.resume = controls::secondary_button(
            ui,
            theme,
            &messages.text("action-resume"),
            true,
            full_width,
        );
        ui.add_space(theme.gap(1.5));
    }

    actions.start = controls::primary_button(
        ui,
        theme,
        &messages.text("action-start"),
        settings.can_start(),
        full_width,
    );

    if finished {
        ui.add_space(theme.gap(1.5));
        actions.reset =
            controls::secondary_button(ui, theme, &messages.text("action-reset"), true, full_width);
    }
}

/// Nada pil untuk sebuah keadaan. Dipakai bersama oleh panel samping dan area kerja.
pub fn tone_for(running: bool, finished: bool, succeeded: bool) -> PillTone {
    if running {
        PillTone::Running
    } else if finished && succeeded {
        PillTone::Success
    } else if finished {
        PillTone::Failure
    } else {
        PillTone::Neutral
    }
}
