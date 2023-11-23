// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#[macro_use]
#[cfg(test)]
mod testing;
mod args;
mod cli;
mod tui;

#[cfg(test)]
use std::io::Write;

fn main() {
    let args = match args::parse() {
        Ok(args) => args,
        Err(err) => {
            println!("ERROR: {err}");
            return;
        }
    };

    if let Err(err) = args.run() {
        println!("ERROR: {err}")
    }
}
