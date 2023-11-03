// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use core::slice;

use common::addr::PhysAddr;
use common::memory::physical_to_virtual;

pub const LINE_SPACING: usize = 2;

pub struct Framebuffer {
    buffer: &'static mut [u8],
    red_shift: u8,
    green_shift: u8,
    blue_shift: u8,
    reserved_mask: u32,
    pitch: u64,
}

impl Framebuffer {
    pub fn new(
        buffer: &'static mut [u8],
        red_shift: u8,
        green_shift: u8,
        blue_shift: u8,
        reserved_mask: u32,
        pitch: u64,
    ) -> Self {
        Framebuffer {
            buffer,
            red_shift,
            green_shift,
            blue_shift,
            reserved_mask,
            pitch,
        }
    }

    /// Updates the buffer address to the new memory model.
    pub fn rewire(&mut self) {
        let ptr = PhysAddr::new_truncate(self.buffer.as_ptr() as u64);
        let ptr = physical_to_virtual(ptr).as_mut_ptr::<u8>();
        // SAFETY: since the existing buffer was valid and the new buffer just points to the same physical memory,
        // creating a slice like this should work. The original address is inaccessible and will fault on read/write.
        self.buffer = unsafe { slice::from_raw_parts_mut(ptr, self.buffer.len()) };
    }

    /// Encodes a red, green and blue color value into a combined [`u32`].
    pub const fn encode_color(&self, (red, green, blue): (u8, u8, u8)) -> u32 {
        ((red as u32) << self.red_shift)
            | ((green as u32) << self.green_shift)
            | ((blue as u32) << self.blue_shift)
            | self.reserved_mask
    }

    /// Scrolls the framebuffer up.
    pub fn scroll(&mut self, max_columns: usize) {
        let start = self.pitch as usize + LINE_SPACING;
        let end = (max_columns - 1) * start;

        // we need to copy everything up in the buffer
        self.buffer.copy_within(start..end, 0);

        // and clear the last row, to remove artifacts.
        self.buffer[end..].fill(12);
    }

    /// Sets a pixel at position x, y to the given color.
    /// The color needs to be encoded with `encode_color` first.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        let index = (y * self.pitch as usize) + (x * 4);
        self.buffer[index..(index + 4)].copy_from_slice(&color.to_ne_bytes());
    }
}
