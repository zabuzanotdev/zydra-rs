//! Titik masuk `zydra`.
//!
//! Seluruh pekerjaan ada di pustaka crate ini supaya tes integrasi bisa menjalankan jalur
//! yang sama persis dengan yang dijalankan pengguna.

use std::process::ExitCode;

fn main() -> ExitCode {
    // Argumen OS dikonversi secara lossy: jalur yang bukan UTF-8 akan ditolak oleh lapisan
    // berkas dengan pesan biasa, bukan membuat program panik sebelum sempat menjelaskan.
    let args: Vec<String> = std::env::args_os()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();
    ExitCode::from(zydra_cli::run_with_args(&args))
}
