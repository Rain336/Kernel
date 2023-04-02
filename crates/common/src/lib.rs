//! # Common Kernel Library
//!
//! The common kernel library contins difrent parts of the kernel that are not architecture specific or
//! only in small parts that can easily be gated using `#[cfg]`.
//!
//! - [`sync`] Primitives of syncronization.
#![no_std]

pub mod sync;
