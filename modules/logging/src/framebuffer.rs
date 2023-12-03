// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use common::addr::PhysAddr;
use common::memory::physical_to_virtual;
use core::ptr;

pub const LINE_SPACING: usize = 2;

pub struct Framebuffer {
    buffer: *mut u8,
    size: usize,
    red_shift: u8,
    green_shift: u8,
    blue_shift: u8,
    reserved_mask: u32,
    pitch: u64,
}

impl Framebuffer {
    pub fn new(
        buffer: *mut u8,
        size: usize,
        red_shift: u8,
        green_shift: u8,
        blue_shift: u8,
        reserved_mask: u32,
        pitch: u64,
    ) -> Self {
        Framebuffer {
            buffer,
            size,
            red_shift,
            green_shift,
            blue_shift,
            reserved_mask,
            pitch,
        }
    }

    /// Updates the buffer address to the new memory model.
    pub fn rewire(&mut self) {
        let ptr = PhysAddr::new_truncate(self.buffer as u64);
        self.buffer = physical_to_virtual(ptr).as_mut_ptr::<u8>();
    }

    /// Encodes a red, green and blue color value into a combined [`u32`].
    pub const fn encode_color(&self, (red, green, blue): (u8, u8, u8)) -> u32 {
        ((red as u32) << self.red_shift)
            | ((green as u32) << self.green_shift)
            | ((blue as u32) << self.blue_shift)
            | self.reserved_mask
    }

    /// Sets a pixel at position x, y to the given color.
    /// The color needs to be encoded with `encode_color` first.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        let index = (y * self.pitch as usize) + (x * 4);
        let color = color.to_ne_bytes();

        unsafe { ptr::copy_nonoverlapping(color.as_ptr(), self.buffer.add(index), color.len()) };
    }

    /// Sets all pixels starting at the given y coordinates until the end of the buffer to the default background color.
    pub fn clear_pixels(&mut self, y: usize) {
        let y = y * self.pitch as usize;
        unsafe { ptr::write_bytes(self.buffer.add(y), 12, self.size - y) };
    }

    /// Copies the given amount of pixels at index `from` to index `to`.
    pub fn copy_pixels(&mut self, from_y: usize, to_y: usize, size: usize) {
        let from_y = from_y * self.pitch as usize;
        let to_y = to_y * self.pitch as usize;
        let size = size * self.pitch as usize;
        unsafe { ptr::copy(self.buffer.add(from_y), self.buffer.add(to_y), size) };
    }
}

unsafe impl Send for Framebuffer {}
