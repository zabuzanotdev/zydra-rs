//! Titik masuk antarmuka grafis.

use eframe::egui;
use zydra_gui::app::ZydraApp;
use zydra_gui::constants::{
    DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH,
};
use zydra_i18n::Messages;

fn main() -> eframe::Result<()> {
    // Judul jendela ikut bahasa sistem, sama seperti isi antarmukanya.
    let messages = Messages::detected();
    let title = messages.text("app-window-title");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size([DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT])
            .with_min_inner_size([MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        "zydra",
        options,
        Box::new(|context| {
            let mut app = ZydraApp::new(context);
            // Pengaturan dimuat sebelum bingkai pertama digambar, supaya bahasa yang
            // tersimpan tidak tampil terlambat satu bingkai.
            app.load_config();
            Ok(Box::new(app))
        }),
    )
}
