//! Konsol log.
//!
//! Entri disimpan sebagai kunci terjemahan, bukan kalimat jadi, sehingga konsol selalu
//! tampil dalam bahasa yang sedang aktif. Mengganti bahasa di tengah jalan akan mengubah
//! seluruh baris lama sekaligus, termasuk yang tercatat sebelum bahasa diganti.

use egui::{Frame, Margin, RichText, ScrollArea, Ui};

use zydra_core::log::{LogBuffer, LogEntry, LogLevel};
use zydra_i18n::Messages;

use crate::constants::TEXT_ROW_HEIGHT;
use crate::theme::Theme;

/// Menggambar konsol log dengan tinggi tetap.
pub fn log_console(
    ui: &mut Ui,
    theme: &Theme,
    messages: &Messages,
    log: &LogBuffer,
    rows: usize,
    height: f32,
) {
    Frame::default()
        .fill(theme.bg_console)
        .stroke(theme.card_stroke())
        .corner_radius(theme.control_radius())
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ScrollArea::vertical()
                .max_height(height)
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if log.is_empty() {
                        ui.label(
                            RichText::new(messages.text("log-panel-empty"))
                                .size(theme.type_scale.mono)
                                .color(theme.text_muted),
                        );
                        return;
                    }
                    // Hanya baris terakhir yang digambar: menyimpan ribuan entri tidak
                    // berguna kalau yang terlihat hanya beberapa baris terakhir.
                    let skip = log.len().saturating_sub(rows);
                    for entry in log.entries().skip(skip) {
                        entry_line(ui, theme, messages, entry);
                    }
                });
        });
}

/// Tinggi konsol untuk sejumlah baris tertentu.
pub fn console_height(rows: usize) -> f32 {
    (rows as f32 * TEXT_ROW_HEIGHT).max(crate::constants::LOG_MIN_HEIGHT)
}

fn entry_line(ui: &mut Ui, theme: &Theme, messages: &Messages, entry: &LogEntry) {
    let colour = match entry.level {
        LogLevel::Info => theme.text_muted,
        LogLevel::Warn => theme.warning,
        LogLevel::Error => theme.error,
        LogLevel::Success => theme.success,
    };
    ui.label(
        RichText::new(messages.log_line(entry))
            .size(theme.type_scale.mono)
            .monospace()
            .color(colour),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_console_never_collapses_below_the_minimum_height() {
        assert!(console_height(0) >= crate::constants::LOG_MIN_HEIGHT);
        assert!(console_height(1) >= crate::constants::LOG_MIN_HEIGHT);
    }

    #[test]
    fn more_rows_means_a_taller_console() {
        assert!(console_height(8) > console_height(4));
    }

    #[test]
    fn a_tall_console_grows_past_the_minimum() {
        let expected = 20.0 * TEXT_ROW_HEIGHT;
        assert!((console_height(20) - expected).abs() < f32::EPSILON);
    }
}
