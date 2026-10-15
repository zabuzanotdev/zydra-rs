//! Catatan kejadian yang tidak terikat bahasa.
//!
//! Entri menyimpan kunci terjemahan beserta argumennya, bukan kalimat jadi. Karena itu log
//! bisa ditampilkan dan diekspor dalam bahasa yang sedang aktif, dan bahasa bisa diganti
//! setelah kejadian tercatat tanpa kehilangan apa pun.

use std::collections::{BTreeMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::constants::DEFAULT_LOG_CAPACITY;

/// Tingkat kepentingan sebuah entri.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLevel {
    /// Kejadian biasa.
    Info,
    /// Sesuatu yang sebaiknya diperhatikan.
    Warn,
    /// Kegagalan.
    Error,
    /// Hasil yang diinginkan tercapai.
    Success,
}

/// Satu kejadian, disimpan sebagai kunci terjemahan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Waktu kejadian, dalam detik sejak epoch Unix.
    pub at: i64,
    /// Tingkat kepentingan.
    pub level: LogLevel,
    /// Kunci terjemahan.
    pub key: String,
    /// Argumen untuk kunci tersebut.
    pub args: BTreeMap<String, String>,
}

impl LogEntry {
    /// Membuat entri bertanda waktu sekarang.
    pub fn now(level: LogLevel, key: impl Into<String>) -> LogEntry {
        LogEntry {
            at: unix_now(),
            level,
            key: key.into(),
            args: BTreeMap::new(),
        }
    }

    /// Menambahkan satu argumen.
    pub fn with_arg(mut self, name: impl Into<String>, value: impl Into<String>) -> LogEntry {
        self.args.insert(name.into(), value.into());
        self
    }
}

/// Waktu sekarang dalam detik sejak epoch Unix.
///
/// Jam sistem yang mundur sebelum epoch menghasilkan `0` alih-alih panik.
pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

/// Cincin penyimpanan berbatas untuk entri log.
///
/// Batasnya penting: serangan brute force bisa berjalan berjam-jam, dan menahan seluruh
/// catatannya di memori akan menghabiskan RAM tanpa manfaat.
#[derive(Debug, Clone)]
pub struct LogBuffer {
    entries: VecDeque<LogEntry>,
    capacity: usize,
}

impl LogBuffer {
    /// Membuat cincin dengan kapasitas tertentu. Kapasitas `0` berarti tidak menyimpan
    /// apa pun.
    pub fn new(capacity: usize) -> LogBuffer {
        LogBuffer {
            entries: VecDeque::with_capacity(capacity.min(1_024)),
            capacity,
        }
    }

    /// Menambahkan entri. Entri tertua dibuang saat kapasitas penuh.
    pub fn push(&mut self, entry: LogEntry) {
        if self.capacity == 0 {
            return;
        }
        while self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Membuat entri bertanda waktu sekarang lalu menyimpannya.
    pub fn record(&mut self, level: LogLevel, key: impl Into<String>) {
        self.push(LogEntry::now(level, key));
    }

    /// Entri dari yang tertua ke yang terbaru.
    pub fn entries(&self) -> impl DoubleEndedIterator<Item = &LogEntry> {
        self.entries.iter()
    }

    /// Jumlah entri yang tersimpan.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Apakah tidak ada entri tersimpan.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Kapasitas cincin.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Mengosongkan cincin tanpa mengubah kapasitasnya.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for LogBuffer {
    fn default() -> LogBuffer {
        LogBuffer::new(DEFAULT_LOG_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(at: i64, key: &str) -> LogEntry {
        LogEntry {
            at,
            level: LogLevel::Info,
            key: key.to_string(),
            args: BTreeMap::new(),
        }
    }

    #[test]
    fn entries_are_kept_in_the_order_they_arrived() {
        let mut buffer = LogBuffer::new(8);
        buffer.push(entry(1, "first"));
        buffer.push(entry(2, "second"));
        let keys: Vec<&str> = buffer.entries().map(|e| e.key.as_str()).collect();
        assert_eq!(keys, vec!["first", "second"]);
    }

    #[test]
    fn the_oldest_entry_is_dropped_when_capacity_is_reached() {
        let mut buffer = LogBuffer::new(3);
        for index in 0..5 {
            buffer.push(entry(index, &format!("k{index}")));
        }
        assert_eq!(buffer.len(), 3);
        let keys: Vec<&str> = buffer.entries().map(|e| e.key.as_str()).collect();
        assert_eq!(keys, vec!["k2", "k3", "k4"]);
    }

    #[test]
    fn a_zero_capacity_buffer_keeps_nothing() {
        let mut buffer = LogBuffer::new(0);
        buffer.push(entry(1, "ignored"));
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn clearing_keeps_the_capacity() {
        let mut buffer = LogBuffer::new(4);
        buffer.push(entry(1, "a"));
        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.capacity(), 4);
        buffer.push(entry(2, "b"));
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn arguments_survive_a_round_trip() {
        let entry = LogEntry::now(LogLevel::Warn, "some-key")
            .with_arg("count", "42")
            .with_arg("name", "vault.zip");
        assert_eq!(entry.args.get("count").map(String::as_str), Some("42"));
        assert_eq!(
            entry.args.get("name").map(String::as_str),
            Some("vault.zip")
        );
    }

    #[test]
    fn a_timestamped_entry_gets_a_plausible_time() {
        let entry = LogEntry::now(LogLevel::Info, "k");
        assert!(
            entry.at > 1_600_000_000,
            "timestamp looks wrong: {}",
            entry.at
        );
    }

    #[test]
    fn the_default_buffer_uses_the_configured_capacity() {
        assert_eq!(LogBuffer::default().capacity(), DEFAULT_LOG_CAPACITY);
    }
}
