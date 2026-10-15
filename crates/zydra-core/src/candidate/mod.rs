//! Sumber kandidat password. Mesin tidak peduli dari mana kandidat berasal.

pub mod bruteforce;
pub mod wordlist;

/// Deretan kandidat password.
///
/// Implementasi harus bisa dipindahkan ke thread lain. Urutan kandidat harus stabil
/// supaya [`CandidateSource::skip`] bisa dipakai untuk melanjutkan serangan.
///
/// **Kontrak kehabisan yang wajib dipenuhi:** setelah [`CandidateSource::next_candidate`]
/// mengembalikan `None`, setiap pemanggilan berikutnya harus tetap mengembalikan `None`.
/// Sumber yang memulai ulang dari awal setelah habis akan membuat mesin paralel menelusuri
/// ruang yang sama tanpa henti, karena worker lain masih menarik kandidat setelah worker
/// pertama menemui ujungnya.
pub trait CandidateSource: Send {
    /// Kandidat berikutnya, atau `None` kalau sudah habis. Setelah `None`, selamanya
    /// `None`.
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
