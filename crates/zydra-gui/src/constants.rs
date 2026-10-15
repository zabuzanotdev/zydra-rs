//! Angka dan batas yang hanya dipakai antarmuka.
//!
//! Sengaja tidak ditaruh di `zydra-core`. Core tidak boleh tahu apa pun soal tata letak,
//! dan menaruh batas piksel di sana akan membuat aturan dependensi berlubang.

/// Lebar jendela minimum. Di bawah ini tata letak `Compact` tidak lagi masuk akal.
pub const MIN_WINDOW_WIDTH: f32 = 360.0;

/// Tinggi jendela minimum.
pub const MIN_WINDOW_HEIGHT: f32 = 480.0;

/// Lebar jendela saat pertama dibuka.
pub const DEFAULT_WINDOW_WIDTH: f32 = 1180.0;

/// Tinggi jendela saat pertama dibuka.
pub const DEFAULT_WINDOW_HEIGHT: f32 = 760.0;

/// Ambang lebar untuk tata letak dua kolom penuh.
pub const BREAKPOINT_WIDE: f32 = 1100.0;

/// Ambang lebar untuk tata letak menengah.
pub const BREAKPOINT_MEDIUM: f32 = 720.0;

/// Ambang lebar untuk tata letak sempit.
pub const BREAKPOINT_NARROW: f32 = 420.0;

/// Lebar sidebar pada tata letak lebar.
pub const SIDEBAR_WIDTH_WIDE: f32 = 240.0;

/// Lebar sidebar pada tata letak menengah.
pub const SIDEBAR_WIDTH_MEDIUM: f32 = 200.0;

/// Tinggi bilah progres.
pub const PROGRESS_BAR_HEIGHT: f32 = 8.0;

/// Tinggi minimum konsol log, supaya tidak pernah menyusut jadi nol.
pub const LOG_MIN_HEIGHT: f32 = 96.0;

/// Tinggi satu baris teks, dipakai untuk menghitung tinggi konsol log.
pub const TEXT_ROW_HEIGHT: f32 = 18.0;
