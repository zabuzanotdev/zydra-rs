//! Font yang dibundel bersama aplikasi.
//!
//! Berkas font dimasukkan ke dalam binary, jadi tidak ada yang perlu dipasang pengguna dan
//! tidak ada tampilan yang bergantung pada font sistem.
//!
//! Dua keluarga dipakai:
//!
//! - **Inter** untuk teks antarmuka. Bentuk hurufnya netral dan terbaca jelas di ukuran
//!   kecil, yang penting karena banyak label di sini berukuran 10 sampai 13 piksel.
//! - **JetBrains Mono** untuk angka, nama berkas, password, dan konsol. Angka monospace
//!   lebarnya tidak berubah saat nilainya berubah, sehingga baris tidak bergoyang setiap
//!   kali kemajuan diperbarui.
//!
//! egui tidak memilih tebal-tipis huruf sendiri; `strong()` hanya mengubah warnanya. Karena
//! itu JetBrains Mono Bold didaftarkan sebagai keluarga bernama terpisah, dan dipakai
//! khusus untuk angka persentase besar.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

const INTER: &[u8] = include_bytes!("../assets/fonts/Inter.ttf");
const MONO_REGULAR: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
const MONO_BOLD: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf");

/// Nama keluarga untuk teks antarmuka.
pub const INTER_FAMILY: &str = "zydra-inter";

/// Nama keluarga untuk teks monospace.
pub const MONO_FAMILY: &str = "zydra-mono";

/// Nama keluarga untuk angka besar yang perlu tampil tebal.
pub const DISPLAY_FAMILY: &str = "zydra-display";

/// Keluarga font untuk angka persentase besar.
pub fn display_family() -> FontFamily {
    FontFamily::Name(DISPLAY_FAMILY.into())
}

/// Susunan font yang dipakai aplikasi.
///
/// Font bawaan egui tetap disisipkan di belakang sebagai cadangan, supaya karakter yang
/// tidak ada di Inter atau JetBrains Mono tetap tergambar alih-alih muncul sebagai kotak.
pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        INTER_FAMILY.to_owned(),
        Arc::new(FontData::from_static(INTER)),
    );
    fonts.font_data.insert(
        MONO_FAMILY.to_owned(),
        Arc::new(FontData::from_static(MONO_REGULAR)),
    );
    fonts.font_data.insert(
        DISPLAY_FAMILY.to_owned(),
        Arc::new(FontData::from_static(MONO_BOLD)),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, INTER_FAMILY.to_owned());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, MONO_FAMILY.to_owned());

    fonts.families.insert(
        display_family(),
        vec![DISPLAY_FAMILY.to_owned(), MONO_FAMILY.to_owned()],
    );

    fonts
}

/// Menerapkan font ke konteks egui.
pub fn apply(context: &egui::Context) {
    context.set_fonts(definitions());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_files_are_real_fonts() {
        // Magic number TrueType: 0x00010000. Kalau berkasnya rusak atau terunduh sebagian,
        // egui akan gagal memuat font saat dijalankan, dan itu jauh lebih mahal daripada
        // pemeriksaan beberapa byte di sini.
        for (name, bytes) in [
            (INTER_FAMILY, INTER),
            (MONO_FAMILY, MONO_REGULAR),
            (DISPLAY_FAMILY, MONO_BOLD),
        ] {
            assert!(bytes.len() > 1_000, "{name} is suspiciously small");
            assert_eq!(
                &bytes[0..4],
                &[0x00, 0x01, 0x00, 0x00],
                "{name} is not a TrueType font"
            );
        }
    }

    #[test]
    fn every_family_has_at_least_one_font() {
        let fonts = definitions();
        for family in [
            FontFamily::Proportional,
            FontFamily::Monospace,
            display_family(),
        ] {
            let entries = fonts
                .families
                .get(&family)
                .unwrap_or_else(|| panic!("family {family:?} is missing"));
            assert!(!entries.is_empty(), "family {family:?} has no fonts");
        }
    }

    #[test]
    fn our_fonts_come_first_so_they_win_over_the_built_in_ones() {
        let fonts = definitions();
        assert_eq!(
            fonts.families[&FontFamily::Proportional]
                .first()
                .map(String::as_str),
            Some(INTER_FAMILY)
        );
        assert_eq!(
            fonts.families[&FontFamily::Monospace]
                .first()
                .map(String::as_str),
            Some(MONO_FAMILY)
        );
    }

    #[test]
    fn the_built_in_fonts_stay_available_as_a_fallback() {
        // Tanpa cadangan, karakter di luar cakupan Inter atau JetBrains Mono akan muncul
        // sebagai kotak kosong.
        let fonts = definitions();
        assert!(
            fonts.families[&FontFamily::Proportional].len() > 1,
            "no fallback font behind Inter"
        );
        assert!(
            fonts.families[&FontFamily::Monospace].len() > 1,
            "no fallback font behind JetBrains Mono"
        );
    }

    #[test]
    fn every_registered_family_name_has_font_data() {
        let fonts = definitions();
        for entries in fonts.families.values() {
            for name in entries {
                assert!(
                    fonts.font_data.contains_key(name),
                    "family lists {name} but no data was registered for it"
                );
            }
        }
    }

    #[test]
    fn the_display_family_uses_the_bold_face_first() {
        let fonts = definitions();
        assert_eq!(
            fonts.families[&display_family()]
                .first()
                .map(String::as_str),
            Some(DISPLAY_FAMILY)
        );
    }
}
