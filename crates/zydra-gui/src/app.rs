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
use zydra_core::config::{AppConfig, ConfigStore, LanguageChoice};
use zydra_core::engine::crack::EngineConfig;
use zydra_core::model::target::CrackTarget;
use zydra_i18n::{Locale, Messages};

use crate::controller::Controller;
use crate::panels::{settings, sidebar, workspace};
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
    /// Apakah ada sesi tersimpan untuk target yang sedang dipilih.
    resume_available: bool,
    /// Keadaan berjalan pada bingkai sebelumnya, untuk mengetahui kapan serangan berakhir.
    was_running: bool,
    /// Pengaturan yang bertahan antar sesi.
    config: AppConfig,
    /// Tempat pengaturan itu disimpan. `None` kalau direktori konfigurasi tidak bisa
    /// ditentukan, dan panelnya tetap bisa dipakai untuk sesi ini saja.
    config_store: Option<ConfigStore>,
}

impl ZydraApp {
    /// Membuat aplikasi dan menerapkan temanya.
    pub fn new(context: &eframe::CreationContext<'_>) -> ZydraApp {
        let theme = Theme::default();
        // Font dipasang sebelum tema supaya ukuran teks dihitung dengan huruf yang benar.
        crate::fonts::apply(&context.egui_ctx);
        theme.apply(&context.egui_ctx);
        theme.apply_spacing(&context.egui_ctx);

        let settings = UiSettings::default();
        let mut app = ZydraApp {
            theme,
            messages: Messages::detected(),
            controller: Controller::default(),
            settings,
            notice: None,
            resume_available: false,
            was_running: false,
            config: AppConfig::default(),
            config_store: None,
        };
        app.settings.sidebar_open = true;
        app
    }

    /// Memuat pengaturan tersimpan, lalu menerapkannya.
    ///
    /// Dipanggil setelah konteks egui siap karena bahasa yang tersimpan harus dipasang ke
    /// katalog sebelum bingkai pertama digambar. Tanpa itu, jendela akan tampil dalam bahasa
    /// sistem selama satu bingkai sebelum berubah.
    pub fn load_config(&mut self) {
        let Ok(store) = ConfigStore::in_user_config_dir() else {
            return;
        };
        let (config, problem) = store.load();
        self.config = config;
        self.config_store = Some(store);
        self.apply_language();

        // Pengaturan yang rusak diganti bawaan, tetapi pengguna yang mengubah berkasnya
        // sendiri perlu tahu bahwa perubahannya diabaikan.
        if let Some(error) = problem {
            self.controller.note(
                zydra_core::log::LogLevel::Warn,
                "error-unknown",
                &[("detail", &error.to_string())],
            );
        }
    }

    /// Menerapkan pilihan bahasa dari pengaturan.
    fn apply_language(&mut self) {
        if let Some(locale) = locale_for(self.config.language) {
            self.messages.set_locale(locale);
        }
    }

    /// Menyimpan pengaturan, dan mencatat hasilnya.
    fn save_config(&mut self) {
        let Some(store) = self.config_store.as_ref() else {
            // Tanpa tempat menyimpan, pengaturannya tetap berlaku untuk sesi ini. Menolak
            // menyimpan akan lebih membingungkan daripada menerimanya.
            self.controller
                .note(zydra_core::log::LogLevel::Info, "settings-saved", &[]);
            return;
        };

        match store.save(&self.config) {
            Ok(()) => self
                .controller
                .note(zydra_core::log::LogLevel::Info, "settings-saved", &[]),
            Err(error) => {
                self.controller
                    .note(zydra_core::log::LogLevel::Error, "settings-invalid", &[]);
                self.controller.note(
                    zydra_core::log::LogLevel::Error,
                    "error-unknown",
                    &[("detail", &error.to_string())],
                );
            }
        }
    }

    /// Batas serangan dari pengaturan.
    fn limits(&self) -> AttackLimits {
        AttackLimits {
            max_combinations: self.config.max_combinations,
            max_brute_length: self.config.max_brute_length,
        }
    }

    /// Pengaturan mesin dari pengaturan.
    fn engine_config(&self) -> EngineConfig {
        EngineConfig {
            workers: self.config.workers,
            batch_size: self.config.batch_size,
            progress_interval: Duration::from_millis(self.config.progress_interval_ms),
        }
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

    /// Menyegarkan pengetahuan tentang ada atau tidaknya sesi tersimpan.
    ///
    /// Dipanggil saat target berubah dan saat serangan berakhir, bukan setiap bingkai:
    /// pemeriksaannya membaca sistem berkas, dan melakukannya terus-menerus akan membuat
    /// antarmuka tersendat tanpa manfaat.
    fn refresh_resume(&mut self) {
        self.resume_available = self
            .settings
            .target
            .as_ref()
            .map(crate::controller::session_available)
            .unwrap_or(false);
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
                self.refresh_resume();
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
            self.start(ctx, layout, false);
        }

        if actions.resume {
            self.start(ctx, layout, true);
        }

        if actions.stop {
            self.controller.stop();
        }

        if actions.reset {
            self.controller.reset();
            self.notice = None;
            self.refresh_resume();
        }
    }

    /// Menulis catatan aktivitas ke berkas yang dipilih pengguna.
    fn export_activity_log(&mut self) {
        let suggested = suggested_log_file_name(self.settings.target.as_ref());
        let Some(path) = rfd::FileDialog::new().set_file_name(suggested).save_file() else {
            return;
        };

        match zydra_i18n::export::write_log(self.controller.log(), &self.messages, &path) {
            Ok(()) => {
                self.notice = None;
                // Hasilnya masuk ke catatan yang sama, supaya pengguna menemukannya di
                // tempat yang memang ia lihat alih-alih di pemberitahuan yang bisa terlewat.
                self.controller.note(
                    zydra_core::log::LogLevel::Info,
                    "log-exported",
                    &[("path", path.to_string_lossy().as_ref())],
                );
            }
            Err(_) => {
                self.notice = Some("error-log-export".to_string());
            }
        }
    }

    fn start(&mut self, ctx: &Context, layout: LayoutMode, resume: bool) {
        let Ok(plan) = self.settings.plan() else {
            return;
        };
        let Some(target) = self.settings.target.clone() else {
            return;
        };
        let limits = self.limits();
        let config = self.engine_config();
        self.controller
            .use_zip_max_entry_bytes(self.config.zip_max_entry_bytes);

        // Sebab kegagalan sudah disimpan pengendali dan akan ditampilkan area kerja dengan
        // kalimat yang tepat, jadi tidak ada pemberitahuan umum yang perlu ditambah.
        if self
            .controller
            .start(plan, target, &limits, config, resume)
            .is_ok()
        {
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

        // Serangan yang baru berakhir meninggalkan sesi tersimpan, jadi tombol melanjutkan
        // perlu dievaluasi ulang tepat pada saat itu.
        let running = self.controller.is_running();
        if self.was_running && !running {
            self.refresh_resume();
        }
        self.was_running = running;

        let layout = LayoutMode::from_width(ctx.screen_rect().width());
        let full_width = layout.buttons_are_full_width();

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            self.top_bar(ui, layout);
        });

        let mut actions = sidebar::SidebarActions::default();
        let mut settings_actions = settings::SettingsActions::default();

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
                        self.resume_available,
                    );

                    ui.add_space(self.theme.gap(4.0));
                    // Dilipat supaya panel pilihan tetap pendek; pengaturan diubah jauh
                    // lebih jarang daripada target dan metode.
                    settings_actions =
                        egui::CollapsingHeader::new(self.messages.text("settings-title"))
                            .default_open(false)
                            .show(ui, |ui| {
                                settings::settings_panel(
                                    ui,
                                    &self.theme,
                                    &self.messages,
                                    &mut self.config,
                                    layout,
                                )
                            })
                            .body_returned
                            .unwrap_or_default();
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
                        self.resume_available,
                    );
                }
            });
        }

        if settings_actions.save {
            self.save_config();
        }
        if settings_actions.reset {
            self.config.reset();
            self.apply_language();
        }

        let mut copy_password = None;
        let mut export_log = false;

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

            if workspace_actions.export_log {
                export_log = true;
            }
        });

        if let Some(password) = copy_password {
            ctx.copy_text(password);
            self.settings.mark_copied();
        }

        if export_log {
            self.export_activity_log();
        }

        let _ = full_width;
        self.handle_sidebar_actions(actions, ctx, layout);
    }
}

/// Bahasa yang dipilih pengaturan, atau `None` kalau harus mengikuti sistem.
fn locale_for(choice: LanguageChoice) -> Option<Locale> {
    match choice {
        LanguageChoice::Auto => None,
        LanguageChoice::En => Some(Locale::En),
        LanguageChoice::Id => Some(Locale::Id),
    }
}

/// Nama berkas yang diusulkan saat mengekspor catatan.
///
/// Diturunkan dari nama berkas target supaya beberapa pekerjaan yang diekspor ke direktori
/// yang sama tidak saling menimpa. Karakter yang tidak aman untuk nama berkas diganti.
pub fn suggested_log_file_name(target: Option<&CrackTarget>) -> String {
    let stem = target
        .and_then(|target| {
            std::path::Path::new(&target.display_name)
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
        })
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "zydra".to_string());

    let safe: String = stem
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .take(40)
        .collect();

    format!("{safe}-activity.log")
}

/// Membuka dialog berkas asli sistem.
///
/// Dialog ini memblokir sampai pengguna memilih atau membatalkan. Itu perilaku yang
/// diharapkan dari dialog asli, dan jedanya tidak terasa karena hanya terjadi saat
/// pengguna sendiri yang memintanya.
fn pick_file() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new().pick_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_suggested_name_follows_the_target() {
        let target = CrackTarget::from_path("/tmp/vault.zip");
        assert_eq!(suggested_log_file_name(Some(&target)), "vault-activity.log");
    }

    #[test]
    fn different_targets_get_different_suggested_names() {
        // Beberapa pekerjaan yang diekspor ke direktori yang sama tidak boleh saling
        // menimpa tanpa disadari.
        let first = CrackTarget::from_path("/tmp/vault.zip");
        let second = CrackTarget::from_path("/tmp/archive.rar");
        assert_ne!(
            suggested_log_file_name(Some(&first)),
            suggested_log_file_name(Some(&second))
        );
    }

    #[test]
    fn without_a_target_a_generic_name_is_used() {
        assert_eq!(suggested_log_file_name(None), "zydra-activity.log");
    }

    #[test]
    fn odd_characters_in_the_target_name_are_replaced() {
        let target = CrackTarget {
            kind: zydra_core::model::target::TargetKind::Zip,
            path: "x".into(),
            display_name: "my file: v2?.zip".to_string(),
        };
        let name = suggested_log_file_name(Some(&target));
        assert!(
            name.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'),
            "unsafe suggested name: {name}"
        );
        assert!(name.ends_with("-activity.log"), "{name}");
    }

    #[test]
    fn a_very_long_target_name_is_shortened() {
        let target = CrackTarget {
            kind: zydra_core::model::target::TargetKind::Zip,
            path: "x".into(),
            display_name: format!("{}.zip", "a".repeat(200)),
        };
        let name = suggested_log_file_name(Some(&target));
        assert!(
            name.len() < 80,
            "suggested name is too long: {len}",
            len = name.len()
        );
    }
}
