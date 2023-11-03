//! The sync module contains various primitives of synchronization.
//! List of the primitives:
//!
//! - [`lazy`] A lazy cell that is Send + Sync, based on [`once`]
//! - [`once`] A once cell that is Send + Sync
//! - [`rwlock`] A reader-writer Spinlock
//! - [`section`] A critical section RAII type to mark uninterruptible sections of code
//! - [`spinning_top`] Re-exports the spinning top Spinlock crate

mod lazy;
mod once;
mod rwlock;
mod section;

pub use lazy::*;
pub use once::*;
pub use rwlock::*;
pub use section::*;
pub use spinning_top::*;
