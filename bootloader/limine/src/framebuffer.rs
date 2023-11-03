use limine::FramebufferRequest;
use runner::interface::FramebufferInfo;

static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new(0);

pub fn get_framebuffer_info() -> Option<FramebufferInfo> {
    if let Some(response) = FRAMEBUFFER_REQUEST.get_response().get() {
        let fb = response.framebuffers().iter().find(|x| {
            x.bpp == 32 && x.red_mask_size == 8 && x.green_mask_size == 8 && x.blue_mask_size == 8
        });

        if let Some(fb) = fb {
            if let Some(address) = fb.address.as_ptr() {
                return Some(FramebufferInfo {
                    address,
                    size: fb.size(),
                    width: fb.width,
                    height: fb.height,
                    pitch: fb.pitch,
                    red_mask_shift: fb.red_mask_shift,
                    green_mask_shift: fb.green_mask_shift,
                    blue_mask_shift: fb.blue_mask_shift,
                });
            }
        }
    }

    None
}
