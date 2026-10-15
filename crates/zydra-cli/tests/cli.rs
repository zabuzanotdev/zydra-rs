//! Tes end-to-end untuk baris perintah.
//!
//! Tes ini menjalankan binary yang sesungguhnya, bukan fungsi internal, supaya yang diuji
//! adalah jalur yang benar-benar dipakai pengguna: argumen, keluaran, dan kode keluar.
//! Arsip uji dibuat di dalam tes, jadi tidak ada berkas biner di repositori.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use zip::write::SimpleFileOptions;
use zip::AesMode;

const SECRET: &str = "kr4k3n_1987";

/// Jalur binary yang dibangun cargo untuk tes ini.
fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_zydra")
}

fn run(arguments: &[&str]) -> Output {
    Command::new(binary())
        .args(arguments)
        .output()
        .expect("failed to run the zydra binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn exit_code(output: &Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

/// Membuat arsip ZIP berenkripsi AES-256.
fn encrypted_archive(directory: &Path, password: &str) -> PathBuf {
    let path = directory.join("vault.zip");
    let file = std::fs::File::create(&path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().with_aes_encryption(AesMode::Aes256, password);
    writer.start_file("secret.txt", options).unwrap();
    writer
        .write_all(b"isi rahasia yang panjangnya melewati satu buffer baca")
        .unwrap();
    writer.finish().unwrap();
    path
}

fn wordlist(directory: &Path, lines: &[&str]) -> PathBuf {
    let path = directory.join("words.txt");
    let mut file = std::fs::File::create(&path).unwrap();
    for line in lines {
        writeln!(file, "{line}").unwrap();
    }
    path
}

#[test]
fn a_wordlist_finds_the_password_and_exits_zero() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), SECRET);
    let words = wordlist(directory.path(), &["password", "letmein", SECRET]);

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
    ]);

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.contains(SECRET),
        "password missing from output: {text}"
    );
    assert!(text.contains("Password found"), "unexpected output: {text}");
}

#[test]
fn a_wordlist_without_the_password_exits_one() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), SECRET);
    let words = wordlist(directory.path(), &["password", "letmein"]);

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
    ]);

    assert_eq!(exit_code(&output), 1, "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Password not found"));
}

#[test]
fn a_brute_force_search_works_from_the_command_line() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), "42");

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-b",
        "digits",
        "-m",
        "1",
        "-x",
        "2",
    ]);

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("42"));
}

#[test]
fn the_old_letters_charset_name_still_works() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), "ab");

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-b",
        "letters",
        "-m",
        "2",
        "-x",
        "2",
        "--threads",
        "2",
    ]);

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("ab"));
}

#[test]
fn a_missing_target_is_a_usage_error() {
    let output = run(&["-d", "words.txt"]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("Choose a target file"));
}

#[test]
fn asking_for_both_methods_is_a_usage_error() {
    let output = run(&[
        "-f",
        "vault.zip",
        "-d",
        "words.txt",
        "-b",
        "digits",
        "-m",
        "1",
        "-x",
        "2",
    ]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("not both"));
}

#[test]
fn an_unknown_charset_class_is_named_in_the_error() {
    let output = run(&["-f", "vault.zip", "-b", "nope", "-m", "1", "-x", "2"]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("nope"));
}

#[test]
fn an_unsupported_file_type_is_a_target_error() {
    let directory = tempfile::tempdir().unwrap();
    let notes = directory.path().join("notes.txt");
    std::fs::write(&notes, b"nothing to see").unwrap();
    let words = wordlist(directory.path(), &["a"]);

    let output = run(&[
        "-f",
        &notes.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
    ]);

    assert_eq!(exit_code(&output), 3, "stderr: {}", stderr(&output));
    assert!(stderr(&output).contains("Unsupported file type"));
}

#[test]
fn a_missing_archive_is_a_target_error() {
    let directory = tempfile::tempdir().unwrap();
    let words = wordlist(directory.path(), &["a"]);

    let output = run(&[
        "-f",
        &directory.path().join("gone.zip").to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
    ]);

    assert_eq!(exit_code(&output), 3);
    assert!(stderr(&output).contains("cannot be read"));
}

#[test]
fn a_file_that_is_not_a_rar_archive_is_a_target_error() {
    let directory = tempfile::tempdir().unwrap();
    let rar = directory.path().join("vault.rar");
    std::fs::write(&rar, b"a rar extension without a rar signature").unwrap();
    let words = wordlist(directory.path(), &["a"]);

    let output = run(&["-f", &rar.to_string_lossy(), "-d", &words.to_string_lossy()]);

    assert_eq!(exit_code(&output), 3, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("RAR"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn json_output_is_machine_readable() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), SECRET);
    let words = wordlist(directory.path(), &["password", SECRET]);

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
        "--json",
    ]);

    assert_eq!(exit_code(&output), 0);
    let text = stdout(&output);
    assert!(text.contains("\"kind\":\"started\""), "output: {text}");
    assert!(text.contains("\"kind\":\"found\""), "output: {text}");
    assert!(
        text.contains("\"password\":\"kr4k3n_1987\""),
        "output: {text}"
    );
    // Keluaran terstruktur tidak boleh tercampur laporan manusia.
    assert!(!text.contains("Password found"), "output: {text}");
}

#[test]
fn json_output_reports_a_failure_with_a_stable_code() {
    let directory = tempfile::tempdir().unwrap();
    let rar = directory.path().join("vault.rar");
    std::fs::write(&rar, b"not a rar archive at all").unwrap();
    let words = wordlist(directory.path(), &["a"]);

    let output = run(&[
        "-f",
        &rar.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
        "--json",
    ]);

    assert_eq!(exit_code(&output), 3);
    assert!(
        stdout(&output).contains("\"code\":\"backend-unavailable-rar\""),
        "output: {}",
        stdout(&output)
    );
}

#[test]
fn quiet_output_has_no_escape_codes() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), SECRET);
    let words = wordlist(directory.path(), &[SECRET]);

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
        "--quiet",
    ]);

    assert_eq!(exit_code(&output), 0);
    assert!(
        !stdout(&output).contains('\u{1b}'),
        "quiet output contained colour: {:?}",
        stdout(&output)
    );
}

#[test]
fn the_language_flag_switches_the_whole_interface() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), SECRET);
    let words = wordlist(directory.path(), &["password"]);

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
        "--lang",
        "id",
    ]);

    assert_eq!(exit_code(&output), 1);
    assert!(
        stdout(&output).contains("Password tidak ditemukan"),
        "output: {}",
        stdout(&output)
    );
}

#[test]
fn help_is_localised_and_is_not_an_error() {
    let english = run(&["--help"]);
    assert_eq!(exit_code(&english), 0, "stderr: {}", stderr(&english));
    assert!(stdout(&english).contains("Recover the password"));

    let indonesian = run(&["--lang", "id", "--help"]);
    assert_eq!(exit_code(&indonesian), 0);
    assert!(
        stdout(&indonesian).contains("Memulihkan password"),
        "output: {}",
        stdout(&indonesian)
    );
}

#[test]
fn an_unknown_language_is_rejected() {
    let output = run(&["--lang", "fr", "-f", "vault.zip", "-d", "words.txt"]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("fr"));
}

#[test]
fn a_rar_archive_is_cracked_from_the_command_line() {
    // Fixture ini milik crate zydra-core; tes ini menjalankan binary yang sesungguhnya
    // terhadap berkas yang sama, supaya backend RAR terbukti lewat jalur pengguna.
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("zydra-core")
        .join("tests")
        .join("fixtures")
        .join("rar4_solid_encrypted.rar");
    let directory = tempfile::tempdir().unwrap();
    let words = wordlist(directory.path(), &["letmein", "123456", "password"]);

    let output = run(&[
        "-f",
        &fixture.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
        "--quiet",
    ]);

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("password"), "output: {text}");
}

#[test]
fn a_pdf_archive_is_cracked_from_the_command_line() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("zydra-core")
        .join("tests")
        .join("fixtures")
        .join("encrypted.pdf");
    let directory = tempfile::tempdir().unwrap();
    let words = wordlist(directory.path(), &["letmein", "kr4k3n_1987"]);

    let output = run(&[
        "-f",
        &fixture.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
        "--quiet",
    ]);

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("kr4k3n_1987"));
}

#[test]
fn an_unencrypted_archive_is_reported_as_a_target_error() {
    let directory = tempfile::tempdir().unwrap();
    let archive = directory.path().join("plain.zip");
    let file = std::fs::File::create(&archive).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    writer
        .start_file("plain.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"no password here").unwrap();
    writer.finish().unwrap();
    let words = wordlist(directory.path(), &["password"]);

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
    ]);

    // Tanpa pemeriksaan ini, arsip tanpa enkripsi terbuka dengan password apa pun dan
    // kandidat pertama dilaporkan sebagai password yang ditemukan.
    assert_eq!(exit_code(&output), 3, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("not encrypted"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn an_unencrypted_rar_archive_is_reported_as_a_target_error() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("zydra-core")
        .join("tests")
        .join("fixtures")
        .join("rar4_plain.rar");
    let directory = tempfile::tempdir().unwrap();
    let words = wordlist(directory.path(), &["password"]);

    let output = run(&[
        "-f",
        &fixture.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
    ]);

    assert_eq!(exit_code(&output), 3, "stderr: {}", stderr(&output));
    assert!(stderr(&output).contains("not encrypted"));
}

#[test]
fn version_is_reported() {
    let output = run(&["--version"]);
    assert_eq!(exit_code(&output), 0);
    assert!(stdout(&output).contains("zydra"));
}

#[test]
fn the_activity_log_can_be_exported_to_a_file() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), SECRET);
    let words = wordlist(directory.path(), &["letmein", SECRET]);
    let log = directory.path().join("activity.log");

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
        "--export-log",
        &log.to_string_lossy(),
        "--quiet",
    ]);

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    let exported = std::fs::read_to_string(&log).unwrap();
    assert!(exported.contains("vault.zip"), "exported: {exported}");
    // Hasil akhirnya ikut tercatat, bukan hanya kejadian sebelum hasilnya keluar.
    assert!(exported.contains("Password found"), "exported: {exported}");
    assert!(
        exported.contains(SECRET) || exported.contains("Password"),
        "exported: {exported}"
    );
}

#[test]
fn the_exported_log_follows_the_active_language() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), SECRET);
    let words = wordlist(directory.path(), &["letmein"]);
    let log = directory.path().join("activity.log");

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
        "--export-log",
        &log.to_string_lossy(),
        "--lang",
        "id",
        "--quiet",
    ]);

    assert_eq!(exit_code(&output), 1, "stderr: {}", stderr(&output));
    let exported = std::fs::read_to_string(&log).unwrap();
    assert!(
        exported.contains("Catatan aktivitas Zydra"),
        "exported: {exported}"
    );
    assert!(
        exported.contains("Password tidak ditemukan"),
        "exported: {exported}"
    );
}

#[test]
fn exporting_to_an_unwritable_path_is_a_target_error() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), SECRET);
    let words = wordlist(directory.path(), &[SECRET]);
    let log = directory.path().join("nowhere").join("activity.log");

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
        "--export-log",
        &log.to_string_lossy(),
        "--quiet",
    ]);

    assert_eq!(exit_code(&output), 3, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("could not be written"),
        "stderr: {}",
        stderr(&output)
    );
}

/// Menjalankan binary dengan direktori sesi yang ditentukan.
fn run_with_sessions(arguments: &[&str], session_dir: &std::path::Path) -> Output {
    Command::new(binary())
        .args(arguments)
        .env("ZYDRA_SESSION_DIR", session_dir)
        .output()
        .expect("failed to run the zydra binary")
}

/// Menulis sesi tersimpan untuk sebuah berkas target, seolah serangan sebelumnya terputus.
fn write_session(
    session_dir: &std::path::Path,
    target: &std::path::Path,
    plan: zydra_core::attack::AttackPlan,
    position: u64,
) {
    use zydra_core::model::target::CrackTarget;
    use zydra_core::session::store::{
        FileSessionStore, SessionState, SessionStore, TargetFingerprint,
    };

    let store = FileSessionStore::new(session_dir);
    let fingerprint = TargetFingerprint::of(&CrackTarget::from_path(target)).unwrap();
    store
        .save(&SessionState::new(fingerprint, plan, position))
        .unwrap();
}

#[test]
fn a_run_can_be_resumed_from_a_saved_position() {
    let directory = tempfile::tempdir().unwrap();
    let sessions = directory.path().join("sessions");
    // Password berada di ujung ruang pencarian, jadi kalau posisinya salah dilanjutkan
    // hasilnya akan berbeda.
    let archive = encrypted_archive(directory.path(), "999");

    write_session(
        &sessions,
        &archive,
        zydra_core::attack::AttackPlan::BruteForce {
            charset: zydra_core::model::charset::CharsetSet::DIGITS,
            min_length: 3,
            max_length: 3,
        },
        950,
    );

    let output = run_with_sessions(
        &[
            "-f",
            &archive.to_string_lossy(),
            "-b",
            "digits",
            "-m",
            "3",
            "-x",
            "3",
            "--resume",
            "--quiet",
        ],
        &sessions,
    );

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("999"), "output: {text}");
    assert!(text.contains("Resuming from"), "output: {text}");
}

#[test]
fn resuming_without_a_saved_session_starts_from_the_beginning() {
    let directory = tempfile::tempdir().unwrap();
    let sessions = directory.path().join("sessions");
    let archive = encrypted_archive(directory.path(), "42");

    let output = run_with_sessions(
        &[
            "-f",
            &archive.to_string_lossy(),
            "-b",
            "digits",
            "-m",
            "1",
            "-x",
            "2",
            "--resume",
            "--quiet",
        ],
        &sessions,
    );

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("No saved session"));
}

#[test]
fn resuming_a_changed_file_is_refused_rather_than_silently_restarted() {
    let directory = tempfile::tempdir().unwrap();
    let sessions = directory.path().join("sessions");
    let archive = encrypted_archive(directory.path(), "42");

    write_session(
        &sessions,
        &archive,
        zydra_core::attack::AttackPlan::BruteForce {
            charset: zydra_core::model::charset::CharsetSet::DIGITS,
            min_length: 1,
            max_length: 2,
        },
        5,
    );

    // Berkas targetnya berubah setelah sesi disimpan.
    std::fs::write(&archive, b"the file is no longer the same archive").unwrap();

    let output = run_with_sessions(
        &[
            "-f",
            &archive.to_string_lossy(),
            "-b",
            "digits",
            "-m",
            "1",
            "-x",
            "2",
            "--resume",
            "--quiet",
        ],
        &sessions,
    );

    assert_eq!(exit_code(&output), 3, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("does not match"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn a_completed_run_leaves_nothing_to_resume() {
    let directory = tempfile::tempdir().unwrap();
    let sessions = directory.path().join("sessions");
    let archive = encrypted_archive(directory.path(), "42");

    let first = run_with_sessions(
        &[
            "-f",
            &archive.to_string_lossy(),
            "-b",
            "digits",
            "-m",
            "1",
            "-x",
            "2",
            "--quiet",
        ],
        &sessions,
    );
    assert_eq!(exit_code(&first), 0);

    // Sesi dibersihkan setelah serangan selesai, jadi percobaan berikutnya yang meminta
    // lanjut akan memulai dari awal alih-alih melanjutkan dari posisi yang sudah selesai.
    let second = run_with_sessions(
        &[
            "-f",
            &archive.to_string_lossy(),
            "-b",
            "digits",
            "-m",
            "1",
            "-x",
            "2",
            "--resume",
            "--quiet",
        ],
        &sessions,
    );
    assert!(stdout(&second).contains("No saved session"));
}

#[test]
fn a_mask_search_finds_the_password() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), "z9");

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "--mask",
        "?l?d",
        "--threads",
        "4",
    ]);

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("z9"), "output: {text}");
    assert!(text.contains("mask"), "method should say mask: {text}");
}

#[test]
fn a_mask_search_can_be_narrowed_with_rules() {
    let directory = tempfile::tempdir().unwrap();
    // Kata dasar huruf kecil semua, password sebenarnya berhuruf besar di awal.
    let archive = encrypted_archive(directory.path(), "Z9");

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "--mask",
        "?l?d",
        "--rules",
        "capitalize",
        "--threads",
        "4",
    ]);

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Z9"));
}

#[test]
fn rules_are_applied_to_a_wordlist_too() {
    let directory = tempfile::tempdir().unwrap();
    let archive = encrypted_archive(directory.path(), "Secret7");
    let words = wordlist(directory.path(), &["secret", "other"]);

    let output = run(&[
        "-f",
        &archive.to_string_lossy(),
        "-d",
        &words.to_string_lossy(),
        "--rules",
        "capitalize+append-digits:9",
    ]);

    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Secret7"));
}

#[test]
fn an_invalid_mask_is_a_usage_error() {
    let output = run(&["-f", "vault.zip", "--mask", "?z"]);
    assert_eq!(exit_code(&output), 2);
    assert!(
        stderr(&output).contains("?z"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn an_invalid_rule_is_a_usage_error() {
    let output = run(&["-f", "vault.zip", "--mask", "?d", "--rules", "nope"]);
    assert_eq!(exit_code(&output), 2);
    assert!(
        stderr(&output).contains("nope"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn a_mask_cannot_be_combined_with_a_wordlist() {
    let output = run(&["-f", "vault.zip", "-d", "words.txt", "--mask", "?d"]);
    assert_eq!(exit_code(&output), 2);
    assert!(stderr(&output).contains("not both"));
}

#[test]
fn the_mask_help_explains_the_tokens() {
    let output = run(&["--help"]);
    let text = stdout(&output);
    assert!(text.contains("--mask"), "output: {text}");
    assert!(text.contains("?l"), "output: {text}");
    assert!(text.contains("--rules"), "output: {text}");
}
