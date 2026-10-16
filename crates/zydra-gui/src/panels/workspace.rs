//! Area kerja di sisi kanan: keadaan, kemajuan, statistik, catatan, dan hasil.
//!
//! Panel ini hanya membaca pengendali. Ia tidak pernah memanggil mesin dan tidak pernah
//! mengubah keadaan serangan; satu-satunya aksi yang bisa diminta adalah menyalin
//! password.

use egui::Ui;

use zydra_i18n::{format, Messages};

use crate::controller::{Controller, Phase};
use crate::responsive::LayoutMode;
use crate::state::UiSettings;
use crate::theme::Theme;
use crate::widgets::{
    self, card, log_console, progress_bar, result_banner, stat_tile,
    stat_tile::{status_pill, PillTone},
};

/// Aksi yang diminta pengguna dari area kerja.
#[derive(Debug, Default, Clone, Copy)]
pub struct WorkspaceActions {
    /// Tombol salin password ditekan.
    pub copy: bool,
    /// Tombol ekspor catatan ditekan.
    pub export_log: bool,
}

/// Menggambar area kerja.
pub fn workspace(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    controller: &Controller,
    settings: &UiSettings,
    layout: LayoutMode,
) -> WorkspaceActions {
    let mut actions = WorkspaceActions::default();

    status_row(ui, theme, messages, controller, settings, layout);
    ui.add_space(theme.gap(3.0));
    progress_section(ui, theme, messages, controller, layout);
    ui.add_space(theme.gap(3.0));
    statistics(ui, theme, messages, controller, layout);
    ui.add_space(theme.gap(3.0));
    result_section(ui, theme, messages, controller, &mut actions, layout);
    ui.add_space(theme.gap(3.0));
    activity(ui, theme, messages, controller, &mut actions, layout);

    actions
}

fn status_row(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    controller: &Controller,
    settings: &UiSettings,
    layout: LayoutMode,
) {
    let phase = controller.phase();
    let (key, tone) = match phase {
        Phase::Idle => ("workspace-status-idle", PillTone::Neutral),
        Phase::Running { .. } => ("workspace-status-running", PillTone::Running),
        Phase::Done { .. } => ("workspace-status-done", PillTone::Success),
        Phase::Failed { .. } => ("workspace-status-failed", PillTone::Failure),
        Phase::Cancelled => ("workspace-status-cancelled", PillTone::Neutral),
    };

    ui.horizontal(|ui| {
        status_pill(ui, theme, tone, &messages.text(key));
        if let Some(target) = controller.target().or(settings.target.as_ref()) {
            let limit = if layout.labels_are_short() { 22 } else { 48 };
            widgets::mono(ui, theme, &widgets::truncate(&target.display_name, limit));
        }
    });
}

fn progress_section(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    controller: &Controller,
    layout: LayoutMode,
) {
    let progress = match controller.phase() {
        Phase::Running { progress } => Some(*progress),
        _ => None,
    };
    let fraction = progress.map(|data| data.fraction()).unwrap_or(0.0);
    let tried = progress.map(|data| data.tried).unwrap_or(0);
    let total = controller.total().unwrap_or(0);

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format::percent(fraction))
                .family(crate::fonts::display_family())
                .size(theme.type_scale.display)
                .color(theme.text),
        );
        ui.label(
            egui::RichText::new("%")
                .size(theme.type_scale.heading)
                .color(theme.text_muted),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
            widgets::muted(
                ui,
                theme,
                &format!("{} / {}", format::count(tried), format::count(total)),
            );
        });
    });

    ui.add_space(theme.gap(2.0));
    progress_bar::progress_bar(ui, theme, fraction, crate::constants::PROGRESS_BAR_HEIGHT);

    if layout.labels_are_short() {
        return;
    }
    ui.add_space(theme.gap(1.0));
    widgets::muted(ui, theme, &messages.text("workspace-progress"));
}

fn statistics(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    controller: &Controller,
    layout: LayoutMode,
) {
    let progress = match controller.phase() {
        Phase::Running { progress } => Some(*progress),
        _ => None,
    };
    let unknown = messages.text("stat-value-unknown");
    let speed = progress
        .map(|data| format!("{} p/s", format::count(data.speed_per_second)))
        .unwrap_or_else(|| unknown.clone());
    let elapsed = controller
        .elapsed()
        .map(format::duration)
        .unwrap_or_else(|| unknown.clone());
    let eta = progress
        .and_then(|data| data.eta)
        .map(format::duration)
        .unwrap_or_else(|| unknown.clone());

    let tiles = [
        (messages.text("stat-speed"), speed, Some(theme.success)),
        (messages.text("stat-elapsed"), elapsed, None),
        (messages.text("stat-eta"), eta, None),
    ];

    let columns = layout.stat_columns().max(1);
    let spacing = theme.gap(2.0);
    let width = (ui.available_width() - spacing * (columns as f32 - 1.0)) / columns as f32;
    let width = width.max(72.0);

    let mut index = 0;
    while index < tiles.len() {
        ui.horizontal(|ui| {
            for _ in 0..columns {
                if index >= tiles.len() {
                    break;
                }
                let (label, value, colour) = &tiles[index];
                ui.allocate_ui_with_layout(
                    egui::vec2(width, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        stat_tile::stat_tile(ui, theme, label, value, *colour);
                    },
                );
                index += 1;
            }
        });
        if index < tiles.len() {
            ui.add_space(spacing);
        }
    }
}

fn result_section(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    controller: &Controller,
    actions: &mut WorkspaceActions,
    layout: LayoutMode,
) {
    match controller.phase() {
        Phase::Done { password, detail } => {
            let value = match detail {
                Some(detail) => format!("{password}  ({detail})"),
                None => password.clone(),
            };
            actions.copy = result_banner::result_banner(
                ui,
                theme,
                PillTone::Success,
                &messages.text("result-granted"),
                &value,
                Some(&messages.text("action-copy")),
                layout.buttons_are_full_width(),
            );
        }
        Phase::Failed { failure } => {
            result_banner::result_banner(
                ui,
                theme,
                PillTone::Failure,
                &messages.text("result-denied"),
                &messages.failure(failure),
                None,
                layout.buttons_are_full_width(),
            );
        }
        Phase::Cancelled => {
            result_banner::result_banner(
                ui,
                theme,
                PillTone::Neutral,
                &messages.text("workspace-status-cancelled"),
                &messages.text("result-denied"),
                None,
                layout.buttons_are_full_width(),
            );
        }
        Phase::Idle | Phase::Running { .. } => {}
    }
}

fn activity(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    controller: &Controller,
    actions: &mut WorkspaceActions,
    layout: LayoutMode,
) {
    ui.horizontal(|ui| {
        widgets::section_label(ui, theme, &messages.text("workspace-activity"));
        // Tombol ekspor hanya berguna kalau memang ada yang bisa diekspor.
        if !controller.log().is_empty() {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                actions.export_log = ui.button(messages.text("action-export-log")).clicked();
            });
        }
    });
    ui.add_space(theme.gap(1.0));
    let rows = layout.log_rows();
    log_console::log_console(
        ui,
        theme,
        messages,
        controller.log(),
        rows,
        log_console::console_height(rows),
    );
}

/// Nada pil untuk keadaan pengendali sekarang.
pub fn tone_of(controller: &Controller) -> PillTone {
    match controller.phase() {
        Phase::Running { .. } => PillTone::Running,
        Phase::Done { .. } => PillTone::Success,
        Phase::Failed { .. } => PillTone::Failure,
        Phase::Idle | Phase::Cancelled => PillTone::Neutral,
    }
}

/// Kartu pembungkus dengan judul, dipakai panel lain saat butuh bagian tambahan.
pub fn titled_card<R>(
    ui: &mut Ui,
    theme: &Theme,
    title: &str,
    contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    card::card_with_title(ui, theme, title, contents)
}
