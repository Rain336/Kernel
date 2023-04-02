//! The sync module contins varius primitives of synchronization.
//! List of primitives:
//!
//! - [`lazy`] A lazy cell that is Send + Sync, baced on [`once`]
//! - [`once`] A once cell that is Send + Sync
//! - [`rwlock`] A reader-writer spinlock
//! - [`section`] A critical section RAII type to mark uninterruptable sections of code
//! - [`spinning_top`] Re-exports the spinning top spinlock crate

mod lazy;
mod once;
mod rwlock;
mod section;

pub use lazy::*;
pub use once::*;
pub use rwlock::*;
pub use section::*;
pub use spinning_top::*;
