#![no_std]
#![no_main]

mod limine;
mod graphics;
mod text_rendering;

use core::{panic::PanicInfo, ptr::null_mut, fmt::Write, write};
use limine::{LimineFramebufferRequest, LimineFramebuffer};
use graphics::{Rgb, Color};
use spin::{Mutex, Lazy};
use text_rendering::TEXT_RENDERER;

LIMINE_BASE_REVISION! { 1 }

#[used]
static LIMINE_FB_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest {
    id: LIMINE_FRAMEBUFFER_REQUEST_ID!(),
    revision: 0,
    response: null_mut()
};

static FRAMEBUFFER: Lazy<Mutex<&'static mut LimineFramebuffer>> = Lazy::new(|| {
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
});

#[no_mangle]
extern "C" fn _start() -> ! {
    let bg_color = Color::new(*FRAMEBUFFER.lock(), Rgb { r: 100, g: 255, b: 0 });
    FRAMEBUFFER.lock().fill(bg_color);
    let hello_world = "Hello world!";
    TEXT_RENDERER.lock().add_text(*FRAMEBUFFER.lock(), hello_world);
    TEXT_RENDERER.lock().write_fmt(format_args!("Test number {}", 2)).unwrap();
    writeln!(*TEXT_RENDERER.lock(), "Abc: {}", 19).unwrap();
    writeln!(*TEXT_RENDERER.lock(), "Test new line").unwrap();
    panic!("Panic test");
    loop {}
}

pub fn sleep(loop_iters: u64) {
    let mut i: u64 = 0;
    while i < loop_iters {
        i += 1;
    }
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    write!(*TEXT_RENDERER.lock(), "{}", panic_info).unwrap();
    loop {}
}
