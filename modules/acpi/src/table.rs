// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::AcpiTableHeader;

/// Common trait implemented for every ACPI table struct in this library.
pub trait AcpiTable {
    /// The 4 byte signature in the [`crate::AcpiTableHeader`] that identifies this table.
    const SIGNATURE: &'static [u8; 4];

    /// Returns the [`crate::AcpiTableHeader`] of this table.
    fn header(&self) -> &AcpiTableHeader;

    /// Validates this ACPI table using it's checksum.
    fn validate(&self) -> bool {
        self.header().validate()
    }
}
