use crate::escape::EscapeMode;
use crate::theme;
use common::sync::{Spinlock, SyncLazy};
use core::fmt::Write;
use core::{ptr, slice};
use limine::LimineFramebufferRequest;
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};

/// Requests a list of framebuffers from the bootloader.
static FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest::new(0);

pub static TERMINAL_OUTPUT: SyncLazy<Spinlock<TerminalOutput>> =
    SyncLazy::new(|| Spinlock::new(TerminalOutput::new()));

const BORDER_PADDING: usize = 1;
const LINE_SPACEING: usize = 2;
const RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

/// Logger output creating a write-only terminal based on a framebuffer.
pub struct TerminalOutput {
    // Framebuffer
    buffer: Option<&'static mut [u8]>,
    red_shift: u8,
    green_shift: u8,
    blue_shift: u8,
    reserved_mask: u32,
    pitch: u64,

    // Position tracking
    row: usize,
    column: usize,
    max_rows: usize,
    max_columns: usize,

    // Colors and Escape Seqences
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
    escape_mode: EscapeMode,
}

impl TerminalOutput {
    /// Creates an uninilized terminal output.
    fn new() -> Self {
        TerminalOutput {
            buffer: None,
            red_shift: 0,
            green_shift: 0,
            blue_shift: 0,
            reserved_mask: 0,
            pitch: 0,

            row: 0,
            column: 0,
            max_rows: 0,
            max_columns: 0,

            fg: theme::DEFAULT_FG_COLOR,
            bg: theme::DEFAULT_BG_COLOR,
            escape_mode: EscapeMode::None,
        }
    }

    /// Initializes the terminal output.
    /// This can only be called onces, subseqent calls do nothing.
    pub fn init(&mut self) {
        // Check if buffer is already set and do nothing if.
        if self.buffer.is_some() {
            return;
        }

        // Check if got a framebuffer from the boot loader.
        if let Some(response) = FRAMEBUFFER_REQUEST.get_response().get() {
            // Find a framebuffer that fits our basic requirements.
            // We want a framebuffer with 8 bits per color channel and 32 bits per pixel.
            let fb = response.framebuffers().iter().find(|x| {
                x.bpp == 32
                    && x.red_mask_size == 8
                    && x.green_mask_size == 8
                    && x.blue_mask_size == 8
            });

            // If a fitting framebuffer was found...
            if let Some(fb) = fb {
                // ...calculate a mask of all color channels.
                // The bitwise not gives us a mask of reserved bits.
                let mask = ((u8::MAX as u32) << fb.red_mask_shift)
                    | ((u8::MAX as u32) << fb.green_mask_shift)
                    | ((u8::MAX as u32) << fb.blue_mask_shift);

                // If the framebuffer address is not null... (it should be, otherwise something is really wrong with the bootloader)
                if let Some(address) = fb.address.as_ptr() {
                    // ...memset the buffer to `0`
                    // SAFETY: address is not null, we assume the size given by the bootloader is correct and alignment isn't a concern with u8s
                    unsafe { ptr::write_bytes(address, 12, fb.size()) };

                    // and initilize the struct
                    // SAFETY: See above with `write_bytes`. In addition, initilize is only run once,
                    // since buffer is set after this line, preventing subseqent runs.
                    // This means there will only be one instance of this slice in the whole program.
                    self.buffer = Some(unsafe { slice::from_raw_parts_mut(address, fb.size()) });
                    self.red_shift = fb.red_mask_shift;
                    self.green_shift = fb.green_mask_shift;
                    self.blue_shift = fb.blue_mask_shift;
                    self.reserved_mask = !mask;
                    self.pitch = fb.pitch;

                    // Calculate the max rows and columns we have.
                    self.max_rows = (fb.width as usize - BORDER_PADDING * 2)
                        / get_raster_width(FontWeight::Regular, RASTER_HEIGHT);
                    self.max_columns = (fb.height as usize - BORDER_PADDING * 2)
                        / (RASTER_HEIGHT.val() + LINE_SPACEING);
                }
            }
        }
    }

    /// Sets a pixel at position x, y to the given color.
    /// The color needs to be encoded with `encode_color` first.
    fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if let Some(buffer) = &mut self.buffer {
            let index = (y * self.pitch as usize) + (x * 4);
            buffer[index..(index + 4)].copy_from_slice(&color.to_ne_bytes());
        }
    }

    /// Writes the given [`RasterizedChar`] into the framebuffer.
    fn write_rasterized_char(&mut self, c: RasterizedChar) {
        let width = get_raster_width(FontWeight::Regular, RASTER_HEIGHT);

        for (y, row) in c.raster().iter().enumerate() {
            for (x, intensity) in row.iter().enumerate() {
                let color = self.apply_intensity(*intensity);

                self.set_pixel(
                    BORDER_PADDING + (self.row * width) + x,
                    BORDER_PADDING + (self.column * (RASTER_HEIGHT.val() + LINE_SPACEING)) + y,
                    color,
                );
            }
        }

        // Update the row and make a newline if needed.
        self.row += 1;
        if self.row >= self.max_rows {
            self.newline();
        }
    }

    /// Gets a color with the given intensity between the current background and forground color.
    /// It tries to estimate a gradiant between the background and forground color with `intensity` being the precentange.
    const fn apply_intensity(&self, intensity: u8) -> u32 {
        // let red =
        //     (((self.bg.0 as i32 - self.fg.0 as i32) * intensity as i32) / 256) + self.bg.0 as i32;
        // let green =
        //     (((self.bg.1 as i32 - self.fg.1 as i32) * intensity as i32) / 256) + self.bg.1 as i32;
        // let blue =
        //     (((self.bg.2 as i32 - self.fg.2 as i32) * intensity as i32) / 256) + self.bg.2 as i32;

        // self.encode_color(red as u8, green as u8, blue as u8)

        // if intensity >= 128 {
        //     self.encode_color(self.fg.0, self.fg.1, self.fg.2)
        // } else {
        //     self.encode_color(self.bg.0, self.bg.1, self.bg.2)
        // }

        let inv = 255 - intensity;
        let red =
            ((self.fg.0 as u16 * intensity as u16) / 256) + ((self.bg.0 as u16 * inv as u16) / 256);
        let green =
            ((self.fg.1 as u16 * intensity as u16) / 256) + ((self.bg.1 as u16 * inv as u16) / 256);
        let blue =
            ((self.fg.2 as u16 * intensity as u16) / 256) + ((self.bg.2 as u16 * inv as u16) / 256);

        self.encode_color(red as u8, green as u8, blue as u8)
    }

    /// Encodes a red, green and blue color value into a combined [`u32`].
    const fn encode_color(&self, red: u8, green: u8, blue: u8) -> u32 {
        ((red as u32) << self.red_shift)
            | ((green as u32) << self.green_shift)
            | ((blue as u32) << self.blue_shift)
            | self.reserved_mask
    }

    /// Makes a newline in the framebuffer.
    fn newline(&mut self) {
        // Reset the row.
        self.row = 0;

        // if we reached the end of the screen...
        if self.column == self.max_columns {
            if let Some(buffer) = &mut self.buffer {
                let start = self.pitch as usize;
                let end = (self.max_columns - 1) * start;

                // ...then we need to copy everything up in the buffer
                buffer.copy_within(start..end, 0);

                // and clear the last row, to remove aritfacts.
                buffer[end..].fill(12);
            }
        } else {
            // If we didin't reach the end of the screen, we can just increment row.
            self.column += 1;
        }
    }
}

impl Write for TerminalOutput {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        if self.buffer.is_none() {
            return Ok(());
        }

        if self.escape_mode != EscapeMode::None
            && self.escape_mode.process(&mut self.fg, &mut self.bg, c)
        {
            return Ok(());
        }

        match c {
            '\r' => {}
            '\n' => self.newline(),
            '\x1b' => {
                self.escape_mode = EscapeMode::ExpectOpenBrace;
            }
            _ if (c as u32) < 32 => {}
            _ => {
                if let Some(c) = get_raster(c, FontWeight::Regular, RASTER_HEIGHT) {
                    self.write_rasterized_char(c)
                } else if let Some(c) = get_raster('?', FontWeight::Regular, RASTER_HEIGHT) {
                    self.write_rasterized_char(c)
                }
            }
        }
        Ok(())
    }
}
