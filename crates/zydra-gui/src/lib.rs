//! Antarmuka grafis Zydra.
//!
//! Crate ini hanya mengurus tampilan. Logika pemulihan ada di `zydra-core`, dan seluruh
//! kalimat ada di `zydra-i18n`. Aturan yang dijaga di sini: tidak ada widget yang membaca
//! konfigurasi atau memanggil mesin, dan tidak ada warna yang ditulis di luar
//! [`theme`].

pub mod app;
pub mod constants;
pub mod controller;
pub mod fonts;
pub mod icon;
pub mod panels;
pub mod responsive;
pub mod state;
pub mod theme;
pub mod widgets;

pub use app::ZydraApp;
