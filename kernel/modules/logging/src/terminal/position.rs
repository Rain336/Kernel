// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
pub struct Position {
    row: usize,
    column: usize,
    max_rows: usize,
    max_columns: usize,
}

impl Position {
    pub fn new() -> Self {
        Position {
            row: 0,
            column: 0,
            max_rows: 0,
            max_columns: 0,
        }
    }

    /// Sets the limits of the screen.
    pub fn set_limits(&mut self, max_rows: usize, max_columns: usize) {
        self.max_rows = max_rows;
        self.max_columns = max_columns;
    }

    /// Advances to the next line.
    /// Returns `true` if the end of the screen has been reached.
    pub fn newline(&mut self) -> bool {
        // Reset the row.
        self.row = 0;

        // if we reached the end of the screen...
        if self.column == self.max_columns {
            true
        } else {
            // If we didn't reach the end of the screen, we can just increment row.
            self.column += 1;

            false
        }
    }

    /// Advances to the next character.
    /// Returns `true` if the end of the line has been reached.
    pub fn next(&mut self) -> bool {
        self.row += 1;

        self.row >= self.max_rows
    }

    /// Gets the current row.
    pub fn row(&self) -> usize {
        self.row
    }

    /// Gets the current column.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Gets the column length.
    pub fn max_columns(&self) -> usize {
        self.max_columns
    }
}
