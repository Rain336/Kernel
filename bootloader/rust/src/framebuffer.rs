// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use bootloader_api::info::{Optional, PixelFormat};
use bootloader_api::BootInfo;
use runner::interface::FramebufferInfo;

pub fn get_framebuffer_info(info: &mut BootInfo) -> Option<FramebufferInfo> {
    let fb = match &mut info.framebuffer {
        Optional::Some(x) => x,
        Optional::None => return None,
    };

    if fb.info().bytes_per_pixel != 32 {
        return None;
    }

    let (red_mask_shift, green_mask_shift, blue_mask_shift) = as_shifts(&fb.info().pixel_format)?;

    Some(FramebufferInfo {
        address: fb.buffer_mut().as_mut_ptr(),
        size: fb.info().byte_len,
        width: fb.info().width as u64,
        height: fb.info().height as u64,
        pitch: fb.info().stride as u64,
        red_mask_shift,
        green_mask_shift,
        blue_mask_shift,
    })
}

const fn as_shifts(format: &PixelFormat) -> Option<(u8, u8, u8)> {
    match format {
        PixelFormat::Rgb => Some((24, 16, 8)),
        PixelFormat::Bgr => Some((8, 16, 24)),
        PixelFormat::Unknown {
            red_position,
            green_position,
            blue_position,
        } => Some((*red_position, *green_position, *blue_position)),
        _ => None,
    }
}
