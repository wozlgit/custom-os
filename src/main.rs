#![no_std]
#![no_main]

use core::{panic::PanicInfo, ptr::null_mut};

macro_rules! LIMINE_BASE_REVISION {
    ($x:expr) => { 
        #[used]
        static _LIMINE_BASE_REVISION: [u64; 3] = [ 0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, $x ];
    }
}
// macro_rules! LIMINE_COMMON_MAGIC {
//     () => { 0xc7b1dd30df4c8b88, 0x0a82e883a194f07b } // Invalid Rust sadly :(
// }

macro_rules! LIMINE_FRAMEBUFFER_REQUEST_ID {
    () => { [ 0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, 0x9d5827dcd881dd75, 0xa3148604f6fab11b ] }
}

#[repr(C)]
struct LimineFramebufferRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut LimineFramebufferResponse
}

#[repr(C)]
struct LimineFramebufferResponse {
    revision: u64,
    framebuffer_count: u64,
    framebuffers: *mut *mut LimineFramebuffer
}

#[repr(u8)]
enum LimineFramebufferMemoryModel {
    Rgb = 1
}

#[repr(C)]
struct LimineFramebuffer {
    address: *mut (),
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16, // Bits per pixel
    memory_model: LimineFramebufferMemoryModel,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
    unused: [u8; 7],
    edid_size: u64,
    edid: *mut (),

    /* Response revision 1 */
    mode_count: u64,
    modes: *mut *mut LimineVideoMode
}

#[repr(C)]
struct LimineVideoMode {
    pitch: u64,
    width: u64,
    height: u64,
    bpp: u16,
    memory_model: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8
}

unsafe impl Sync for LimineFramebufferRequest {}

// LIMINE_BASE_REVISION! { 1 }
#[used]
static _LIMINE_BASE_REVISION: [u64; 3] = [ 0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 1 ];

#[used]
static LIMINE_FB_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest {
    id: LIMINE_FRAMEBUFFER_REQUEST_ID!(),
    revision: 0,
    response: null_mut()
};

#[no_mangle]
extern "C" fn _start() -> ! {
    if LIMINE_FB_REQUEST.response.is_null() {
        // ERROR
    }
    else {
        let limine_fb_response = unsafe { &mut *LIMINE_FB_REQUEST.response };
        if limine_fb_response.framebuffer_count < 1 {
            // ERROR
        }
        else {
            let fb1 = unsafe { &mut **limine_fb_response.framebuffers.offset(0) };
            // assert_eq!(fb1.bpp, (fb1.red_mask_size + fb1.blue_mask_size + fb1.green_mask_size).into());
            let fb_address: *mut u32 = fb1.address as *mut u32;
            for x in 0..fb1.width {
                for y in 0..fb1.height {
                    unsafe {
                        let pixel_offset = y * (fb1.pitch / 4) + x;
                        *fb_address.offset(pixel_offset.try_into().unwrap()) = 0xaaffaaff;
                    }
                }
            }
        }
    }
    loop {}
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    loop {}
}
