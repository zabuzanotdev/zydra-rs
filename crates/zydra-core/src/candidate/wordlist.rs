//! Sumber kandidat dari berkas wordlist.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::candidate::CandidateSource;
use crate::constants::WORDLIST_READ_BUFFER_BYTES;
use crate::error::CoreError;

/// Memotong akhir baris dari potongan byte. Hanya `\n` dan `\r` yang dibuang; spasi
/// dipertahankan supaya password berspasi tetap bisa dicoba.
fn strip_eol(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && (bytes[end - 1] == b'\n' || bytes[end - 1] == b'\r') {
        end -= 1;
    }
    &bytes[..end]
}

/// Membaca kandidat satu per satu dari sebuah berkas teks.
pub struct WordlistSource {
    reader: BufReader<File>,
    buffer: Vec<u8>,
    total: u64,
    position: u64,
    finished: bool,
}

impl WordlistSource {
    /// Membuka wordlist dan menghitung jumlah kandidatnya sekali di awal.
    ///
    /// Baris yang kosong setelah akhir baris dipotong tidak dihitung dan tidak
    /// dihasilkan.
    pub fn open(path: &Path) -> Result<WordlistSource, CoreError> {
        let total = count_candidates(path)?;
        let file = File::open(path)?;
        Ok(WordlistSource {
            reader: BufReader::with_capacity(WORDLIST_READ_BUFFER_BYTES, file),
            buffer: Vec::new(),
            total,
            position: 0,
            finished: false,
        })
    }

    /// Membaca potongan berikutnya ke buffer. `false` berarti berkas sudah habis atau
    /// tidak bisa dibaca lagi.
    fn read_chunk(&mut self) -> bool {
        if self.finished {
            return false;
        }
        self.buffer.clear();
        match self.reader.read_until(b'\n', &mut self.buffer) {
            Ok(0) | Err(_) => {
                self.finished = true;
                false
            }
            Ok(_) => true,
        }
    }
}

fn count_candidates(path: &Path) -> Result<u64, CoreError> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(WORDLIST_READ_BUFFER_BYTES, file);
    let mut buffer = Vec::new();
    let mut count = 0u64;
    loop {
        buffer.clear();
        match reader.read_until(b'\n', &mut buffer)? {
            0 => break,
            _ => {
                if !strip_eol(&buffer).is_empty() {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

impl CandidateSource for WordlistSource {
    fn next_candidate(&mut self) -> Option<String> {
        loop {
            if !self.read_chunk() {
                return None;
            }
            let line = strip_eol(&self.buffer);
            if line.is_empty() {
                continue;
            }
            self.position += 1;
            return Some(String::from_utf8_lossy(line).into_owned());
        }
    }

    fn total(&self) -> u64 {
        self.total
    }

    fn position(&self) -> u64 {
        self.position
    }

    fn skip(&mut self, count: u64) {
        let mut remaining = count;
        while remaining > 0 {
            if !self.read_chunk() {
                return;
            }
            if strip_eol(&self.buffer).is_empty() {
                continue;
            }
            self.position += 1;
            remaining -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(bytes).unwrap();
        file.flush().unwrap();
        file
    }

    fn drain(source: &mut WordlistSource) -> Vec<String> {
        let mut out = Vec::new();
        while let Some(candidate) = source.next_candidate() {
            out.push(candidate);
        }
        out
    }

    #[test]
    fn reads_lines_in_order() {
        let file = write_temp(b"alpha\nbravo\ncharlie\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(drain(&mut source), vec!["alpha", "bravo", "charlie"]);
    }

    #[test]
    fn strips_carriage_returns() {
        let file = write_temp(b"alpha\r\nbravo\r\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(drain(&mut source), vec!["alpha", "bravo"]);
    }

    #[test]
    fn a_last_line_without_a_newline_is_still_a_candidate() {
        let file = write_temp(b"alpha\nbravo");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(drain(&mut source), vec!["alpha", "bravo"]);
    }

    #[test]
    fn empty_lines_are_skipped_and_not_counted() {
        let file = write_temp(b"alpha\n\n\nbravo\n\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(source.total(), 2);
        assert_eq!(drain(&mut source), vec!["alpha", "bravo"]);
    }

    #[test]
    fn a_line_of_only_spaces_is_a_real_candidate() {
        let file = write_temp(b"alpha\n \n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(drain(&mut source), vec!["alpha", " "]);
    }

    #[test]
    fn invalid_utf8_is_replaced_instead_of_dropping_the_line() {
        let file = write_temp(b"alp\xffha\nbravo\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        let candidates = drain(&mut source);
        assert_eq!(candidates.len(), 2);
        assert!(candidates[0].starts_with("alp"));
        assert!(candidates[0].ends_with("ha"));
        assert_eq!(candidates[1], "bravo");
    }

    #[test]
    fn total_matches_the_number_of_candidates_actually_yielded() {
        let file = write_temp(b"one\ntwo\n\nthree\nfour\r\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        let total = source.total();
        assert_eq!(total, drain(&mut source).len() as u64);
    }

    #[test]
    fn position_counts_yielded_candidates() {
        let file = write_temp(b"one\ntwo\nthree\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        assert_eq!(source.position(), 0);
        source.next_candidate();
        source.next_candidate();
        assert_eq!(source.position(), 2);
    }

    #[test]
    fn skip_lands_on_the_next_candidate() {
        let file = write_temp(b"one\ntwo\nthree\nfour\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        source.skip(2);
        assert_eq!(source.position(), 2);
        assert_eq!(source.next_candidate().as_deref(), Some("three"));
    }

    #[test]
    fn skip_counts_only_real_candidates() {
        let file = write_temp(b"one\n\ntwo\n\nthree\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        source.skip(1);
        assert_eq!(source.next_candidate().as_deref(), Some("two"));
    }

    #[test]
    fn skipping_past_the_end_yields_nothing_and_does_not_panic() {
        let file = write_temp(b"one\ntwo\n");
        let mut source = WordlistSource::open(file.path()).unwrap();
        source.skip(99);
        assert_eq!(source.next_candidate(), None);
        assert_eq!(source.next_candidate(), None);
    }

    #[test]
    fn opening_a_missing_file_is_an_error() {
        assert!(WordlistSource::open(std::path::Path::new("no-such-file-here.txt")).is_err());
    }
}
