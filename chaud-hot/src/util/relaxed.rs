use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering::Relaxed;

/// A wrapper around [`AtomicU8`] that uses [`Relaxed`] for all operations.
#[repr(transparent)]
pub struct RelaxedU8 {
    val: AtomicU8,
}

impl RelaxedU8 {
    #[inline]
    pub const fn new(val: u8) -> Self {
        Self { val: AtomicU8::new(val) }
    }

    #[inline]
    pub fn load(&self) -> u8 {
        self.val.load(Relaxed)
    }

    #[inline]
    pub fn swap(&self, val: u8) -> u8 {
        self.val.swap(val, Relaxed)
    }

    #[inline]
    pub fn fetch_or(&self, val: u8) -> u8 {
        self.val.fetch_or(val, Relaxed)
    }

    #[inline]
    pub fn fetch_and(&self, val: u8) -> u8 {
        self.val.fetch_and(val, Relaxed)
    }

    #[inline]
    pub fn compare_exchange(&self, current: u8, new: u8) -> Result<u8, u8> {
        self.val.compare_exchange(current, new, Relaxed, Relaxed)
    }
}
