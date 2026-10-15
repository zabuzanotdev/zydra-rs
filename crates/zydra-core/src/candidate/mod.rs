//! Sumber kandidat password. Mesin tidak peduli dari mana kandidat berasal.

pub mod wordlist;

/// Deretan kandidat password.
///
/// Implementasi harus bisa dipindahkan ke thread lain. Urutan kandidat harus stabil
/// supaya [`CandidateSource::skip`] bisa dipakai untuk melanjutkan serangan.
pub trait CandidateSource: Send {
    /// Kandidat berikutnya, atau `None` kalau sudah habis.
    fn next_candidate(&mut self) -> Option<String>;

    /// Jumlah kandidat yang akan dihasilkan. Boleh menjenuh pada `u64::MAX`.
    fn total(&self) -> u64;

    /// Jumlah kandidat yang sudah dihasilkan.
    fn position(&self) -> u64;

    /// Melewati `count` kandidat tanpa mengembalikannya.
    ///
    /// Implementasi bawaan ini selalu benar tetapi bisa lambat. Sumber yang bisa
    /// menghitung posisinya secara langsung sebaiknya menggantinya.
    fn skip(&mut self, count: u64) {
        let mut remaining = count;
        while remaining > 0 {
            if self.next_candidate().is_none() {
                break;
            }
            remaining -= 1;
        }
    }
}
