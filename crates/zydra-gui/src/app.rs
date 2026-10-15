//! Perakitan aplikasi.
//!
//! [`ZydraApp`] adalah titik temu, bukan otak. Ia memegang keadaan antarmuka dan
//! pengendali, meneruskan aksi dari panel ke tempat yang berhak menanganinya, lalu
//! menggambar. Tidak ada aturan pemulihan password di berkas ini, dan tidak ada kalimat
//! yang ditulis langsung di sini.

use std::time::Duration;

use eframe::App;
use egui::{Context, RichText};

use zydra_core::attack::AttackLimits;
use zydra_core::engine::crack::EngineConfig;
use zydra_core::model::target::CrackTarget;
use zydra_i18n::{Locale, Messages};

use crate::controller::Controller;
use crate::panels::{sidebar, workspace};
use crate::responsive::LayoutMode;
use crate::state::UiSettings;
use crate::theme::Theme;

/// Selang penggambaran ulang saat serangan berjalan.
///
/// Tanpa ini, antarmuka hanya akan diperbarui saat ada masukan, dan bilah progres akan
/// tampak membeku walaupun pekerjaannya berjalan normal.
const RUNNING_REPAINT: Duration = Duration::from_millis(50);

/// Aplikasi Zydra.
pub struct ZydraApp {
    theme: Theme,
    messages: Messages,
    controller: Controller,
    settings: UiSettings,
    /// Kunci terjemahan untuk masalah terakhir, ditampilkan sampai ada aksi berikutnya.
    notice: Option<String>,
}

impl ZydraApp {
    /// Membuat aplikasi dan menerapkan temanya.
    pub fn new(context: &eframe::CreationContext<'_>) -> ZydraApp {
        let theme = Theme::default();
        theme.apply(&context.egui_ctx);
        theme.apply_spacing(&context.egui_ctx);

        let settings = UiSettings::default();
        let mut app = ZydraApp {
            theme,
            messages: Messages::detected(),
            controller: Controller::default(),
            settings,
            notice: None,
        };
        app.settings.sidebar_open = true;
        app
    }

    /// Aplikasi dengan bahasa tertentu. Dipakai pengujian dan pengembangan.
    pub fn with_locale(context: &eframe::CreationContext<'_>, locale: Locale) -> ZydraApp {
        let mut app = ZydraApp::new(context);
        app.messages = Messages::new(locale);
        app
    }

    fn top_bar(&mut self, ui: &mut egui::Ui, layout: LayoutMode) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(self.messages.text("app-name"))
                    .size(self.theme.type_scale.heading)
                    .color(self.theme.text),
            );
            ui.label(
                RichText::new(self.messages.text("app-subtitle"))
                    .size(self.theme.type_scale.label)
                    .color(self.theme.accent),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let options = [
                    (Locale::Id, "ID".to_string()),
                    (Locale::En, "EN".to_string()),
                ];
                if let Some(chosen) = crate::widgets::controls::segmented(
                    ui,
                    &self.theme,
                    &options,
                    self.messages.locale(),
                ) {
                    self.messages.set_locale(chosen);
                }

                // Di jendela sempit, panel pilihan pindah ke atas dan perlu tombol
                // buka-tutup. Di jendela lebar ia selalu tampil.
                if !layout.sidebar_is_beside() && layout.sidebar_is_collapsible() {
                    let label = self.messages.text("action-options");
                    if ui
                        .selectable_label(self.settings.sidebar_open, label)
                        .clicked()
                    {
                        self.settings.sidebar_open = !self.settings.sidebar_open;
                    }
                }
            });
        });
    }

    fn handle_sidebar_actions(
        &mut self,
        actions: sidebar::SidebarActions,
        ctx: &Context,
        layout: LayoutMode,
    ) {
        if actions.browse_target {
            if let Some(path) = pick_file() {
                self.settings.target = Some(CrackTarget::from_path(path));
                self.notice = None;
            }
        }

        if actions.browse_wordlist {
            if let Some(path) = pick_file() {
                // Berkas yang tidak bisa dibaca dilaporkan sekarang, bukan nanti saat
                // serangan dimulai, supaya pengguna tidak menunggu untuk sesuatu yang
                // memang tidak akan berjalan.
                if self.settings.set_wordlist(path) {
                    self.notice = None;
                } else {
                    self.notice = Some("error-file-unreadable".to_string());
                }
            }
        }

        if actions.start {
            self.start(ctx, layout);
        }

        if actions.stop {
            self.controller.stop();
        }

        if actions.reset {
            self.controller.reset();
            self.notice = None;
        }
    }

    fn start(&mut self, ctx: &Context, layout: LayoutMode) {
        let Ok(plan) = self.settings.plan() else {
            return;
        };
        let Some(target) = self.settings.target.clone() else {
            return;
        };
        let limits = AttackLimits::default();
        let config = EngineConfig::default();

        // Sebab kegagalan sudah disimpan pengendali dan akan ditampilkan area kerja dengan
        // kalimat yang tepat, jadi tidak ada pemberitahuan umum yang perlu ditambah.
        if self.controller.start(plan, target, &limits, config).is_ok() {
            self.notice = None;
            if layout.sidebar_is_collapsible() {
                // Setelah dimulai, panel pilihan tidak lagi berguna dan ruangnya lebih
                // berharga untuk kemajuan.
                self.settings.sidebar_open = false;
            }
            ctx.request_repaint();
        }
    }
}

impl App for ZydraApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.controller.poll();

        if self.controller.is_running() {
            ctx.request_repaint_after(RUNNING_REPAINT);
        }

        let layout = LayoutMode::from_width(ctx.screen_rect().width());
        let full_width = layout.buttons_are_full_width();

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            self.top_bar(ui, layout);
        });

        let mut actions = sidebar::SidebarActions::default();

        if layout.sidebar_is_beside() {
            let width = layout.sidebar_width().unwrap_or(240.0);
            egui::SidePanel::left("options")
                .exact_width(width)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.add_space(self.theme.gap(2.0));
                    actions = sidebar::sidebar(
                        ui,
                        &self.theme,
                        &self.messages,
                        &mut self.settings,
                        layout,
                        self.controller.is_running(),
                        self.controller.phase().is_finished(),
                    );
                });
        } else {
            egui::TopBottomPanel::top("options").show(ctx, |ui| {
                if self.settings.sidebar_open {
                    ui.add_space(self.theme.gap(1.0));
                    actions = sidebar::sidebar(
                        ui,
                        &self.theme,
                        &self.messages,
                        &mut self.settings,
                        layout,
                        self.controller.is_running(),
                        self.controller.phase().is_finished(),
                    );
                }
            });
        }

        let mut copy_password = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(self.theme.gap(2.0));

            if let Some(key) = self.notice.clone() {
                ui.colored_label(self.theme.error, self.messages.text(&key));
                ui.add_space(self.theme.gap(2.0));
            }

            let workspace_actions = workspace::workspace(
                ui,
                &self.theme,
                &self.messages,
                &self.controller,
                &self.settings,
                layout,
            );

            if workspace_actions.copy {
                if let crate::controller::Phase::Done { password, .. } = self.controller.phase() {
                    copy_password = Some(password.clone());
                }
            }
        });

        if let Some(password) = copy_password {
            ctx.copy_text(password);
            self.settings.mark_copied();
        }

        let _ = full_width;
        self.handle_sidebar_actions(actions, ctx, layout);
    }
}

/// Membuka dialog berkas asli sistem.
///
/// Dialog ini memblokir sampai pengguna memilih atau membatalkan. Itu perilaku yang
/// diharapkan dari dialog asli, dan jedanya tidak terasa karena hanya terjadi saat
/// pengguna sendiri yang memintanya.
fn pick_file() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new().pick_file()
}
