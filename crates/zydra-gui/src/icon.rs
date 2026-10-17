//! Ikon aplikasi yang digambar prosedural.
//!
//! Tidak ada berkas gambar yang dibundel: pikselnya dihitung di sini dari token
//! [`Theme`], sehingga ikon taskbar/dock selalu cocok dengan palet aplikasi.
//! Bentuknya huruf Z geometris (tiga bilah) di atas kartu gelap membulat.

use std::sync::Arc;

use egui::IconData;

/// Ukuran ikon. Harus kelipatan 4 sesuai syarat [`IconData`].
const SIZE: u32 = 64;

/// Latar kartu: sama dengan `Theme::bg_app`.
const BG: [u8; 3] = [0x0b, 0x0f, 0x14];
/// Garis tepi: sama dengan `Theme::border_strong`.
const BORDER: [u8; 3] = [0x2a, 0x3b, 0x4f];
/// Aksen: sama dengan `Theme::accent`.
const ACCENT: [u8; 3] = [0x22, 0xd3, 0xee];

/// Ikon aplikasi untuk [`egui::ViewportBuilder::with_icon`].
pub fn app_icon() -> Arc<IconData> {
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (r, g, b, a) = pixel(x, y);
            rgba.extend_from_slice(&[r, g, b, a]);
        }
    }
    Arc::new(IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    })
}

/// Satu piksel ikon: latar membulat, tepi 2px, dan huruf Z aksen.
fn pixel(x: u32, y: u32) -> (u8, u8, u8, u8) {
    let fx = x as f32 + 0.5;
    let fy = y as f32 + 0.5;
    // Kotak membulat: tengah 32, setengah 30 (inset 2), radius 14.
    let qx = (fx - 32.0).abs() - (30.0 - 14.0);
    let qy = (fy - 32.0).abs() - (30.0 - 14.0);
    let ax = qx.max(0.0);
    let ay = qy.max(0.0);
    let dist = (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - 14.0;
    if dist > 0.0 {
        return (0, 0, 0, 0);
    }
    if is_z_mark(x, y) {
        return (ACCENT[0], ACCENT[1], ACCENT[2], 255);
    }
    if dist > -2.0 {
        return (BORDER[0], BORDER[1], BORDER[2], 255);
    }
    (BG[0], BG[1], BG[2], 255)
}

/// Huruf Z dari tiga bilah: atas, diagonal, bawah.
fn is_z_mark(x: u32, y: u32) -> bool {
    let in_top = (20..44).contains(&x) && (20..26).contains(&y);
    let in_bottom = (20..44).contains(&x) && (38..44).contains(&y);
    if in_top || in_bottom {
        return true;
    }
    if !(26..38).contains(&y) {
        return false;
    }
    // Diagonal dari kanan-atas (42,26) ke kiri-bawah (22,38).
    let center = 42.0 - (y as f32 - 26.0) * (20.0 / 12.0);
    (x as f32 - center).abs() <= 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_icon_has_the_advertised_size() {
        let icon = app_icon();
        assert_eq!(icon.width, SIZE);
        assert_eq!(icon.height, SIZE);
        assert_eq!(icon.rgba.len(), (SIZE * SIZE * 4) as usize);
    }

    #[test]
    fn the_corners_stay_transparent_for_window_compositing() {
        // Piksel sudut harus transparan supaya taskbar menggambar sudut membulat,
        // bukan kotak hitam.
        assert_eq!(pixel(0, 0).3, 0);
        assert_eq!(pixel(63, 0).3, 0);
        assert_eq!(pixel(0, 63).3, 0);
        assert_eq!(pixel(63, 63).3, 0);
    }

    #[test]
    fn the_middle_is_opaque() {
        assert_eq!(pixel(32, 32).3, 255);
    }

    #[test]
    fn the_z_mark_uses_the_accent_colour() {
        // Bilah atas Z.
        assert_eq!(
            (pixel(32, 23).0, pixel(32, 23).1, pixel(32, 23).2),
            (ACCENT[0], ACCENT[1], ACCENT[2])
        );
        // Bilah bawah Z.
        assert_eq!(
            (pixel(32, 41).0, pixel(32, 41).1, pixel(32, 41).2),
            (ACCENT[0], ACCENT[1], ACCENT[2])
        );
    }
}
