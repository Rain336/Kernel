use super::CriticalSection;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, Ordering};

const UNINITIALIZED: u8 = 0;
const INITILIAZING: u8 = 1;
const INITILIAZED: u8 = 2;

/// A cell that can be initilized once and returns it's initilized value on subseqent requests.
/// The cell has an overhead of one byte for tracking if the cell has already been initilized or is still initilizing.
pub struct SyncOnceCell<T> {
    once: AtomicU8,
    value: UnsafeCell<MaybeUninit<T>>,
    _marker: PhantomData<T>,
}

impl<T> SyncOnceCell<T> {
    /// Creates a new uninitilized once cell.
    pub const fn new() -> Self {
        SyncOnceCell {
            once: AtomicU8::new(UNINITIALIZED),
            value: UnsafeCell::new(MaybeUninit::uninit()),
            _marker: PhantomData,
        }
    }

    /// Gets the initilized value of the once cell or `None` if it hasn't been initilized yet.
    pub fn get(&self) -> Option<&T> {
        if self.once.load(Ordering::Relaxed) == INITILIAZED || self.is_initialized() {
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    /// Gets the initilized mutable value of the once cell or `None` if it hasn't been initilized yet.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.once.load(Ordering::Relaxed) == INITILIAZED || self.is_initialized() {
            Some(unsafe { self.get_unchecked_mut() })
        } else {
            None
        }
    }

    /// Tries to initilize the call to `value`, return `Ok(())` on success, `Err(value)` otherwise.
    pub fn set(&self, value: T) -> Result<(), T> {
        let _section = CriticalSection::new();
        match self.once.compare_exchange(
            UNINITIALIZED,
            INITILIAZING,
            Ordering::AcqRel,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                unsafe { (*self.value.get()).write(value) };
                self.once.store(INITILIAZED, Ordering::Release);
                Ok(())
            }
            Err(_) => Err(value),
        }
    }

    /// Gets the initilized value of the once cell, if it hasn't been initilized, otherwise calls the given function to initilize it.
    pub fn get_or_init(&self, f: impl FnOnce() -> T) -> &T {
        let _section = CriticalSection::new();
        loop {
            match self.once.compare_exchange_weak(
                UNINITIALIZED,
                INITILIAZING,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    unsafe { (*self.value.get()).write(f()) };
                    self.once.store(INITILIAZED, Ordering::Release);
                    return unsafe { self.get_unchecked() };
                }
                Err(INITILIAZED) => return unsafe { self.get_unchecked() },
                Err(_) => continue,
            }
        }
    }

    /// Unwraps the cell into it's inner value, if it has been initilized.
    pub fn into_inner(self) -> Option<T> {
        if self.once.load(Ordering::Relaxed) == INITILIAZED || self.is_initialized() {
            Some(unsafe { (*self.value.get()).assume_init_read() })
        } else {
            None
        }
    }

    /// Returns wheever the call has been initilized.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.once.load(Ordering::Acquire) == INITILIAZED
    }

    /// Gets the value of the cell, without checking if it's initilized.
    ///
    /// # Safety
    ///
    /// If the function is called while the cell is still uninitilized, a referance to uninitilzed memory is returned.
    pub unsafe fn get_unchecked(&self) -> &T {
        debug_assert!(self.is_initialized());
        (*self.value.get()).assume_init_ref()
    }

    /// Gets the value of the cell as mutable, without checking if it's initilized.
    ///
    /// # Safety
    ///
    /// If the function is called while the cell is still uninitilized, a referance to uninitilzed memory is returned.
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
