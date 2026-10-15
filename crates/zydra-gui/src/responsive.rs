//! Seluruh keputusan tata letak yang bergantung pada lebar jendela.
//!
//! Semua breakpoint hidup di sini dan tidak di tempat lain. Kalau aturan responsif
//! tersebar di panel-panel, cepat atau lambat dua panel akan memakai ambang yang berbeda
//! dan jendela akan terlihat pincang di lebar tertentu.
//!
//! Modul ini sengaja tidak menyentuh egui sama sekali, sehingga bisa diuji sebagai fungsi
//! murni: lebar masuk, tata letak keluar.

use crate::constants::{
    BREAKPOINT_MEDIUM, BREAKPOINT_NARROW, BREAKPOINT_WIDE, SIDEBAR_WIDTH_MEDIUM, SIDEBAR_WIDTH_WIDE,
};

/// Tata letak yang dipakai pada lebar tertentu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Jendela lebar: sidebar tetap di kiri, tiga kolom statistik.
    Wide,
    /// Jendela sedang: sidebar lebih sempit dan bisa diciutkan, dua kolom statistik.
    Medium,
    /// Jendela sempit: sidebar pindah ke atas sebagai panel yang bisa dibuka-tutup.
    Narrow,
    /// Jendela sangat sempit: seperti sempit, dengan label disingkat dan statistik
    /// bertumpuk.
    Compact,
}

impl LayoutMode {
    /// Menentukan tata letak dari lebar jendela.
    pub fn from_width(width: f32) -> LayoutMode {
        if width >= BREAKPOINT_WIDE {
            LayoutMode::Wide
        } else if width >= BREAKPOINT_MEDIUM {
            LayoutMode::Medium
        } else if width >= BREAKPOINT_NARROW {
            LayoutMode::Narrow
        } else {
            LayoutMode::Compact
        }
    }

    /// Lebar sidebar, atau `None` kalau sidebar dipindah ke atas.
    pub fn sidebar_width(self) -> Option<f32> {
        match self {
            LayoutMode::Wide => Some(SIDEBAR_WIDTH_WIDE),
            LayoutMode::Medium => Some(SIDEBAR_WIDTH_MEDIUM),
            LayoutMode::Narrow | LayoutMode::Compact => None,
        }
    }

    /// Apakah sidebar berdiri sendiri di samping isi, bukan di atasnya.
    pub fn sidebar_is_beside(self) -> bool {
        self.sidebar_width().is_some()
    }

    /// Apakah sidebar bisa diciutkan.
    ///
    /// Di jendela lebar ia selalu tampil; memaksa pengguna membukanya hanya menambah
    /// satu klik tanpa menghemat ruang yang berarti.
    pub fn sidebar_is_collapsible(self) -> bool {
        !matches!(self, LayoutMode::Wide)
    }

    /// Apakah sidebar terbuka secara bawaan.
    pub fn sidebar_starts_open(self) -> bool {
        !matches!(self, LayoutMode::Compact)
    }

    /// Jumlah kolom untuk kartu statistik.
    pub fn stat_columns(self) -> usize {
        match self {
            LayoutMode::Wide => 3,
            LayoutMode::Medium => 2,
            LayoutMode::Narrow | LayoutMode::Compact => 1,
        }
    }

    /// Jumlah baris log yang ditampilkan.
    pub fn log_rows(self) -> usize {
        match self {
            LayoutMode::Wide => 8,
            LayoutMode::Medium => 6,
            LayoutMode::Narrow | LayoutMode::Compact => 4,
        }
    }

    /// Apakah tombol aksi memakai lebar penuh.
    pub fn buttons_are_full_width(self) -> bool {
        matches!(self, LayoutMode::Narrow | LayoutMode::Compact)
    }

    /// Apakah label panjang perlu disingkat.
    pub fn labels_are_short(self) -> bool {
        matches!(self, LayoutMode::Compact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_boundary_belongs_to_the_wider_mode() {
        // Nilai tepat di ambang harus masuk ke mode yang lebih lapang, karena pada lebar
        // itu ruangnya memang cukup.
        assert_eq!(LayoutMode::from_width(BREAKPOINT_WIDE), LayoutMode::Wide);
        assert_eq!(
            LayoutMode::from_width(BREAKPOINT_MEDIUM),
            LayoutMode::Medium
        );
        assert_eq!(
            LayoutMode::from_width(BREAKPOINT_NARROW),
            LayoutMode::Narrow
        );
    }

    #[test]
    fn just_below_each_boundary_drops_a_step() {
        assert_eq!(
            LayoutMode::from_width(BREAKPOINT_WIDE - 0.1),
            LayoutMode::Medium
        );
        assert_eq!(
            LayoutMode::from_width(BREAKPOINT_MEDIUM - 0.1),
            LayoutMode::Narrow
        );
        assert_eq!(
            LayoutMode::from_width(BREAKPOINT_NARROW - 0.1),
            LayoutMode::Compact
        );
    }

    #[test]
    fn the_widths_named_in_the_design_document_land_where_expected() {
        assert_eq!(LayoutMode::from_width(360.0), LayoutMode::Compact);
        assert_eq!(LayoutMode::from_width(420.0), LayoutMode::Narrow);
        assert_eq!(LayoutMode::from_width(720.0), LayoutMode::Medium);
        assert_eq!(LayoutMode::from_width(1100.0), LayoutMode::Wide);
        assert_eq!(LayoutMode::from_width(1600.0), LayoutMode::Wide);
    }

    #[test]
    fn a_zero_or_negative_width_does_not_panic() {
        assert_eq!(LayoutMode::from_width(0.0), LayoutMode::Compact);
        assert_eq!(LayoutMode::from_width(-100.0), LayoutMode::Compact);
    }

    #[test]
    fn a_huge_width_stays_wide() {
        assert_eq!(LayoutMode::from_width(100_000.0), LayoutMode::Wide);
    }

    #[test]
    fn the_sidebar_sits_beside_the_content_only_when_there_is_room() {
        assert!(LayoutMode::Wide.sidebar_is_beside());
        assert!(LayoutMode::Medium.sidebar_is_beside());
        assert!(!LayoutMode::Narrow.sidebar_is_beside());
        assert!(!LayoutMode::Compact.sidebar_is_beside());
    }

    #[test]
    fn the_sidebar_gets_narrower_before_it_moves() {
        let wide = LayoutMode::Wide.sidebar_width().unwrap();
        let medium = LayoutMode::Medium.sidebar_width().unwrap();
        assert!(medium < wide, "medium sidebar should be narrower");
    }

    #[test]
    fn only_the_widest_layout_keeps_the_sidebar_always_open() {
        assert!(!LayoutMode::Wide.sidebar_is_collapsible());
        assert!(LayoutMode::Medium.sidebar_is_collapsible());
        assert!(LayoutMode::Narrow.sidebar_is_collapsible());
        assert!(LayoutMode::Compact.sidebar_is_collapsible());
    }

    #[test]
    fn the_sidebar_starts_closed_only_when_space_is_tightest() {
        assert!(LayoutMode::Wide.sidebar_starts_open());
        assert!(LayoutMode::Medium.sidebar_starts_open());
        assert!(LayoutMode::Narrow.sidebar_starts_open());
        assert!(!LayoutMode::Compact.sidebar_starts_open());
    }

    #[test]
    fn statistic_columns_shrink_as_the_window_narrows() {
        assert_eq!(LayoutMode::Wide.stat_columns(), 3);
        assert_eq!(LayoutMode::Medium.stat_columns(), 2);
        assert_eq!(LayoutMode::Narrow.stat_columns(), 1);
        assert_eq!(LayoutMode::Compact.stat_columns(), 1);
    }

    #[test]
    fn the_log_keeps_at_least_four_rows() {
        for mode in [
            LayoutMode::Wide,
            LayoutMode::Medium,
            LayoutMode::Narrow,
            LayoutMode::Compact,
        ] {
            assert!(mode.log_rows() >= 4, "{mode:?} would show too few rows");
        }
    }

    #[test]
    fn buttons_go_full_width_only_when_the_window_is_narrow() {
        assert!(!LayoutMode::Wide.buttons_are_full_width());
        assert!(!LayoutMode::Medium.buttons_are_full_width());
        assert!(LayoutMode::Narrow.buttons_are_full_width());
        assert!(LayoutMode::Compact.buttons_are_full_width());
    }

    #[test]
    fn labels_are_shortened_only_at_the_tightest_size() {
        assert!(!LayoutMode::Wide.labels_are_short());
        assert!(!LayoutMode::Narrow.labels_are_short());
        assert!(LayoutMode::Compact.labels_are_short());
    }
}
