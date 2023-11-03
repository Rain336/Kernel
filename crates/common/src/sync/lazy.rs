use super::SyncOnceCell;
use core::cell::Cell;
use core::ops::Deref;

/// A Cell that runs it's given parameter on first access and stores it for subsequent accesses.
/// Can be used for lazy static variables.
/// Is based on [`SyncOnceCell`].
pub struct SyncLazy<T, F = fn() -> T> {
    cell: SyncOnceCell<T>,
    init: Cell<Option<F>>,
}

impl<T, F> SyncLazy<T, F> {
    /// Creates a new [`SyncLazy`] with the given function as initializer.
    pub const fn new(f: F) -> Self {
        SyncLazy {
            cell: SyncOnceCell::new(),
            init: Cell::new(Some(f)),
        }
    }
}

impl<T, F: FnOnce() -> T> SyncLazy<T, F> {
    /// Forces the given [`SyncLazy`] to initialize, if it didn't already.
    pub fn force(this: &SyncLazy<T, F>) -> &T {
        this.cell.get_or_init(|| match this.init.take() {
            Some(f) => f(),
            None => panic!("Lazy instance has previously been poisoned"),
        })
    }
}

unsafe impl<T, F: Send> Sync for SyncLazy<T, F> where SyncOnceCell<T>: Sync {}

impl<T, F: FnOnce() -> T> Deref for SyncLazy<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        SyncLazy::force(self)
    }
}

impl<T, F: FnOnce() -> T> AsRef<T> for SyncLazy<T, F> {
    fn as_ref(&self) -> &T {
        SyncLazy::force(self)
    }
}

impl<T: Default> Default for SyncLazy<T> {
    fn default() -> SyncLazy<T> {
        SyncLazy::new(T::default)
    }
}
