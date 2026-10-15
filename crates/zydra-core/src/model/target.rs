//! Jenis berkas target dan deskriptornya.

use std::path::{Path, PathBuf};

/// Jenis berkas yang bisa dicoba.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TargetKind {
    /// Arsip ZIP.
    Zip,
    /// Arsip RAR.
    Rar,
    /// Dokumen PDF.
    Pdf,
    /// Berkas `shadow` Unix.
    Shadow,
    /// Tidak dikenali.
    Unsupported,
}

impl TargetKind {
    /// Menentukan jenis dari nama berkas. Ekstensi diperiksa lebih dulu; baru kemudian
    /// nama yang memuat `shadow` tanpa membedakan huruf besar-kecil.
    pub fn detect(path: &Path) -> TargetKind {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let extension = name.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
        match extension {
            "zip" => TargetKind::Zip,
            "rar" => TargetKind::Rar,
            "pdf" => TargetKind::Pdf,
            _ if name.contains("shadow") => TargetKind::Shadow,
            _ => TargetKind::Unsupported,
        }
    }

    /// Identifier stabil untuk berkas sesi dan konfigurasi.
    pub fn as_str(self) -> &'static str {
        match self {
            TargetKind::Zip => "zip",
            TargetKind::Rar => "rar",
            TargetKind::Pdf => "pdf",
            TargetKind::Shadow => "shadow",
            TargetKind::Unsupported => "unsupported",
        }
    }
}

/// Berkas target beserta nama tampilannya.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrackTarget {
    /// Jenis berkas.
    pub kind: TargetKind,
    /// Lokasi berkas.
    pub path: PathBuf,
    /// Nama yang ditampilkan di antarmuka.
    pub display_name: String,
}

impl CrackTarget {
    /// Membuat target dari sebuah path. Nama berkas dipakai sebagai nama tampilan; kalau
    /// path tidak punya nama berkas, seluruh path dipakai.
    pub fn from_path(path: impl Into<PathBuf>) -> CrackTarget {
        let path = path.into();
        let kind = TargetKind::detect(&path);
        let display_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        CrackTarget {
            kind,
            path,
            display_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_zip_rar_and_pdf_by_extension() {
        assert_eq!(TargetKind::detect(Path::new("vault.zip")), TargetKind::Zip);
        assert_eq!(TargetKind::detect(Path::new("vault.rar")), TargetKind::Rar);
        assert_eq!(TargetKind::detect(Path::new("vault.pdf")), TargetKind::Pdf);
    }

    #[test]
    fn extension_matching_ignores_case() {
        assert_eq!(TargetKind::detect(Path::new("VAULT.ZIP")), TargetKind::Zip);
        assert_eq!(TargetKind::detect(Path::new("Vault.PdF")), TargetKind::Pdf);
    }

    #[test]
    fn detects_a_shadow_file_by_name_without_extension() {
        assert_eq!(
            TargetKind::detect(Path::new("/etc/shadow")),
            TargetKind::Shadow
        );
        assert_eq!(TargetKind::detect(Path::new("SHADOW")), TargetKind::Shadow);
        assert_eq!(
            TargetKind::detect(Path::new("shadow.bak")),
            TargetKind::Shadow
        );
    }

    #[test]
    fn a_known_extension_wins_over_the_shadow_name() {
        assert_eq!(TargetKind::detect(Path::new("shadow.zip")), TargetKind::Zip);
    }

    #[test]
    fn anything_else_is_unsupported() {
        assert_eq!(
            TargetKind::detect(Path::new("notes.txt")),
            TargetKind::Unsupported
        );
        assert_eq!(
            TargetKind::detect(Path::new("noext")),
            TargetKind::Unsupported
        );
    }

    #[test]
    fn from_path_keeps_the_file_name_as_the_display_name() {
        let target = CrackTarget::from_path("/tmp/vault.zip");
        assert_eq!(target.kind, TargetKind::Zip);
        assert_eq!(target.display_name, "vault.zip");
    }

    #[test]
    fn kind_as_str_is_stable_for_persistence() {
        assert_eq!(TargetKind::Zip.as_str(), "zip");
        assert_eq!(TargetKind::Shadow.as_str(), "shadow");
        assert_eq!(TargetKind::Unsupported.as_str(), "unsupported");
    }
}
