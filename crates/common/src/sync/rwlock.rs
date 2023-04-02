use core::sync::atomic::{AtomicU32, Ordering};
use spinning_top::lock_api::{
    GuardSend, RawRwLock, RawRwLockDowngrade, RawRwLockUpgrade, RawRwLockUpgradeDowngrade, RwLock,
    RwLockReadGuard, RwLockUpgradableReadGuard, RwLockWriteGuard,
};

const READER: u32 = 4;
const UPGRADED: u32 = 2;
const WRITER: u32 = 1;

/// The [`RawRwLock`] implementation of the the reader-writer spinlock.
pub struct RawRwSpinlock(AtomicU32);

unsafe impl RawRwLock for RawRwSpinlock {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = RawRwSpinlock(AtomicU32::new(0));

    type GuardMarker = GuardSend;

    fn lock_shared(&self) {
        while !self.try_lock_shared() {
            core::hint::spin_loop()
        }
    }

    fn try_lock_shared(&self) -> bool {
        let old = self.0.fetch_add(READER, Ordering::Acquire);
        if (old & (WRITER | UPGRADED)) != 0 {
            self.0.fetch_sub(READER, Ordering::Release);
            false
        } else {
            true
        }
    }

    unsafe fn unlock_shared(&self) {
        self.0.fetch_sub(READER, Ordering::Release);
    }

    fn lock_exclusive(&self) {
        while !self.try_lock_exclusive() {
            core::hint::spin_loop()
        }
    }

    fn try_lock_exclusive(&self) -> bool {
        self.0
            .compare_exchange(0, WRITER, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
    }

    unsafe fn unlock_exclusive(&self) {
        self.0.fetch_and(!(UPGRADED | WRITER), Ordering::Release);
    }

    fn is_locked(&self) -> bool {
        self.0.load(Ordering::Acquire) != 0
    }

    fn is_locked_exclusive(&self) -> bool {
        (self.0.load(Ordering::Acquire) & WRITER) == WRITER
    }
}

unsafe impl RawRwLockDowngrade for RawRwSpinlock {
    unsafe fn downgrade(&self) {
        self.0.fetch_add(READER, Ordering::Acquire);
        self.unlock_exclusive()
    }
}

unsafe impl RawRwLockUpgrade for RawRwSpinlock {
    fn lock_upgradable(&self) {
        while !self.try_lock_upgradable() {
            core::hint::spin_loop()
        }
    }

    fn try_lock_upgradable(&self) -> bool {
        let old = self.0.fetch_or(UPGRADED, Ordering::Acquire);

        (old & (UPGRADED | WRITER)) == 0
    }

    unsafe fn unlock_upgradable(&self) {
        self.0.fetch_and(!UPGRADED, Ordering::AcqRel);
    }

    unsafe fn upgrade(&self) {
        while !self.try_upgrade() {
            core::hint::spin_loop()
        }
    }

    unsafe fn try_upgrade(&self) -> bool {
        self.0
            .compare_exchange(UPGRADED, WRITER, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
    }
}

unsafe impl RawRwLockUpgradeDowngrade for RawRwSpinlock {
    unsafe fn downgrade_upgradable(&self) {
        self.0.fetch_add(READER - UPGRADED, Ordering::AcqRel);
    }

    unsafe fn downgrade_to_upgradable(&self) {
        self.0.fetch_or(UPGRADED, Ordering::Acquire);
        self.0.fetch_and(!WRITER, Ordering::Release);
    }
}

/// A many reader one writer spinlock.
/// The lock can have up to 30 concurrent readers and supports upgrading locks.
pub type RwSpinlock<T> = RwLock<RawRwSpinlock, T>;
pub type RwSpinlockReadGuard<'a, T> = RwLockReadGuard<'a, RawRwSpinlock, T>;
pub type RwSpinlockUpgradableReadGuard<'a, T> = RwLockUpgradableReadGuard<'a, RawRwSpinlock, T>;
pub type RwSpinlockWriteGuard<'a, T> = RwLockWriteGuard<'a, RawRwSpinlock, T>;
