use super::CriticalSection;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, Ordering};

const UNINITIALIZED: u8 = 0;
const INITIALIZING: u8 = 1;
const INITIALIZED: u8 = 2;

/// A cell that can be initialized once and returns it's initialized value on subsequent requests.
/// The cell has an overhead of one byte for tracking if the cell has already been initialized or is still initializing.
pub struct SyncOnceCell<T> {
    once: AtomicU8,
    value: UnsafeCell<MaybeUninit<T>>,
    _marker: PhantomData<T>,
}

impl<T> SyncOnceCell<T> {
    /// Creates a new uninitialized once cell.
    pub const fn new() -> Self {
        SyncOnceCell {
            once: AtomicU8::new(UNINITIALIZED),
            value: UnsafeCell::new(MaybeUninit::uninit()),
            _marker: PhantomData,
        }
    }

    /// Gets the initialized value of the once cell or `None` if it hasn't been initialized yet.
    pub fn get(&self) -> Option<&T> {
        if self.once.load(Ordering::Relaxed) == INITIALIZED || self.is_initialized() {
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    /// Gets the initialized mutable value of the once cell or `None` if it hasn't been initialized yet.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.once.load(Ordering::Relaxed) == INITIALIZED || self.is_initialized() {
            Some(unsafe { self.get_unchecked_mut() })
        } else {
            None
        }
    }

    /// Tries to initialize the call to `value`, return `Ok(())` on success, `Err(value)` otherwise.
    pub fn set(&self, value: T) -> Result<(), T> {
        let _section = CriticalSection::new();
        match self.once.compare_exchange(
            UNINITIALIZED,
            INITIALIZING,
            Ordering::AcqRel,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                unsafe { (*self.value.get()).write(value) };
                self.once.store(INITIALIZED, Ordering::Release);
                Ok(())
            }
            Err(_) => Err(value),
        }
    }

    /// Gets the initialized value of the once cell, if it hasn't been initialized, otherwise calls the given function to initialize it.
    pub fn get_or_init(&self, f: impl FnOnce() -> T) -> &T {
        let _section = CriticalSection::new();
        loop {
            match self.once.compare_exchange_weak(
                UNINITIALIZED,
                INITIALIZING,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    unsafe { (*self.value.get()).write(f()) };
                    self.once.store(INITIALIZED, Ordering::Release);
                    return unsafe { self.get_unchecked() };
                }
                Err(INITIALIZED) => return unsafe { self.get_unchecked() },
                Err(_) => continue,
            }
        }
    }

    /// Unwraps the cell into it's inner value, if it has been initialized.
    pub fn into_inner(self) -> Option<T> {
        if self.once.load(Ordering::Relaxed) == INITIALIZED || self.is_initialized() {
            Some(unsafe { (*self.value.get()).assume_init_read() })
        } else {
            None
        }
    }

    /// Returns whenever the call has been initialized.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.once.load(Ordering::Acquire) == INITIALIZED
    }

    /// Gets the value of the cell, without checking if it's initialized.
    ///
    /// ## Safety
    ///
    /// If the function is called while the cell is still uninitialized, a reference to uninitiated memory is returned.
    pub unsafe fn get_unchecked(&self) -> &T {
        debug_assert!(self.is_initialized());
        (*self.value.get()).assume_init_ref()
    }

    /// Gets the value of the cell as mutable, without checking if it's initialized.
    ///
    /// ## Safety
    ///
    /// If the function is called while the cell is still uninitialized, a reference to uninitiated memory is returned.
    pub unsafe fn get_unchecked_mut(&mut self) -> &mut T {
        debug_assert!(self.is_initialized());
        (*self.value.get()).assume_init_mut()
    }
}

unsafe impl<T: Send + Sync> Sync for SyncOnceCell<T> {}
unsafe impl<T: Send> Send for SyncOnceCell<T> {}

impl<T> Default for SyncOnceCell<T> {
    fn default() -> Self {
        SyncOnceCell::new()
    }
}
