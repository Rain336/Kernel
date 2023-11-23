// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! ## Testing Utils
//!
//! This module provides utilities writing unit tests.
//! It works by shadowing the println! macro and redirecting it to a shared buffer.
//! This shared buffer can than be asserted for expected outputs.
//!
use std::sync::Mutex;

/// A globally shared buffer that output is written into.
pub static TESTING_STDOUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// The replacement println! macro for testing.
macro_rules! println {
    ( $( $tokens:tt )* ) => {{
        let mut buffer = crate::testing::TESTING_STDOUT.lock().unwrap();
        let _ = writeln!(*buffer, $( $tokens )* );
    }};
}

/// Asserts that the exactly one line with the given `expected` text was printed.
pub fn assert_output(expected: &str) {
    let mut lock = TESTING_STDOUT.lock().unwrap();

    let idx = lock
        .iter()
        .position(|x| *x == b'\n')
        .expect("No lines where printed");

    assert_eq!(
        lock.len(),
        idx + 1,
        "More than one line was printed: {}",
        std::str::from_utf8(&lock[..]).unwrap()
    );

    let str = std::str::from_utf8(&lock[..idx]).unwrap();
    assert_eq!(str, expected);

    lock.clear();
}

/// Asserts that exactly one line starting with `ERROR: ` was printed.
pub fn assert_error() {
    let mut lock = TESTING_STDOUT.lock().unwrap();

    let idx = lock
        .iter()
        .position(|x| *x == b'\n')
        .expect("No lines where printed");

    let str = std::str::from_utf8(&lock[..idx]).unwrap();
    assert!(
        str.starts_with("ERROR: "),
        "Expected to get an error, but got: {}",
        str
    );

    lock.clear();
}
