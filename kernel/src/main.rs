#![no_std]
#![no_main]

mod limine;
mod graphics;

use core::{panic::PanicInfo, ptr::null_mut};
use lazy_static::lazy_static;
use limine::{LimineFramebufferRequest, LimineFramebuffer};
use graphics::{Rgb, Color};
use glyph_textures_from_font_lib::*;
use spin::Mutex;

LIMINE_BASE_REVISION! { 1 }

#[used]
static LIMINE_FB_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest {
    id: LIMINE_FRAMEBUFFER_REQUEST_ID!(),
    revision: 0,
    response: null_mut()
};

lazy_static! {
    static ref FRAMEBUFFER: Mutex<&'static mut LimineFramebuffer> = {
        if LIMINE_FB_REQUEST.response.is_null() {
             // ERROR
             loop {}
        }
        else {
            let limine_fb_response = unsafe { &mut *LIMINE_FB_REQUEST.response };
            if limine_fb_response.framebuffer_count < 1 {
                // ERROR
                loop {}
            }
            else {
                // Use the first framebuffer
                Mutex::new(unsafe { &mut **limine_fb_response.framebuffers })
            }
        }
    };
}

static FONT_BYTES_ALIGN_DUMMY: &'static _AlignDummy<f32, [u8]> = &_AlignDummy { 
    _align: [],
    bytes: *include_bytes!("../../glyph_textures_from_font/glyph_bitmaps.bin")
};

#[no_mangle]
extern "C" fn _start() -> ! {
    let mut fb = FRAMEBUFFER.lock();
    let font_bytes = &FONT_BYTES_ALIGN_DUMMY.bytes;

    let glyph_bitmaps = GlyphBitmapIterator::new(font_bytes);
    let text_color = Rgb { r: 255, g: 0, b: 100 };
    let bg_color = Color::new(*fb, Rgb { r: 100, g: 255, b: 0 });
    fb.fill(bg_color);
    fb.print_text(text_color, glyph_bitmaps);
    loop {}
}

fn sleep(loop_iters: u64) {
    let mut i: u64 = 0;
    while i < loop_iters {
        i += 1;
    }
}

#[panic_handler]
fn panic(_panic_info: &PanicInfo) -> ! {
    FRAMEBUFFER.lock().display_num(20);
    loop {}
}
