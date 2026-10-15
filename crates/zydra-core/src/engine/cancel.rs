//! Tanda pembatalan yang bisa dibagikan ke semua worker.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Tanda berbagi untuk menghentikan pekerjaan yang sedang berjalan.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    /// Membuat tanda yang belum dibatalkan.
    pub fn new() -> CancelToken {
        CancelToken(Arc::new(AtomicBool::new(false)))
    }

    /// Meminta semua worker berhenti.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Apakah pembatalan sudah diminta.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_token_is_not_cancelled() {
        assert!(!CancelToken::new().is_cancelled());
    }

    #[test]
    fn cancelling_is_visible_through_a_clone() {
        let token = CancelToken::new();
        let handle = token.clone();
        token.cancel();
        assert!(handle.is_cancelled());
    }

    #[test]
    fn the_default_token_is_not_cancelled() {
        assert!(!CancelToken::default().is_cancelled());
    }

    #[test]
    fn a_token_can_be_moved_to_another_thread() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CancelToken>();
    }
}
