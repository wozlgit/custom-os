#[macro_export]
macro_rules! LIMINE_BASE_REVISION {
    ($x:expr) => { 
        #[used]
        static _LIMINE_BASE_REVISION: [u64; 3] = [ 0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, $x ];
    }
}

#[macro_export]
macro_rules! LIMINE_FRAMEBUFFER_REQUEST_ID {
    () => { [ 0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, 0x9d5827dcd881dd75, 0xa3148604f6fab11b ] }
}

#[macro_export]
macro_rules! LIMINE_STACK_SIZE_REQUEST_ID {
    () => { [ 0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, 0x224ef0460a8e8926, 0xe1cb0fc25f46ea3d ] }
}

#[repr(C)]
pub struct LimineFramebufferRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *mut LimineFramebufferResponse
}

#[repr(C)]
pub struct LimineFramebufferResponse {
    pub revision: u64,
    pub framebuffer_count: u64,
    pub framebuffers: *mut *mut LimineFramebuffer
}

#[repr(u8)]
pub enum LimineFramebufferMemoryModel {
    Rgb = 1
}

#[repr(C)]
pub struct LimineFramebuffer {
    pub address: *mut (),
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16, // Bits per pixel
    pub memory_model: LimineFramebufferMemoryModel,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    pub unused: [u8; 7],
    pub edid_size: u64,
    pub edid: *mut (),

    /* Response revision 1 */
    pub mode_count: u64,
    pub modes: *mut *mut LimineVideoMode
}

#[repr(C)]
pub struct LimineVideoMode {
    pub pitch: u64,
    pub width: u64,
    pub height: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8
}

unsafe impl Sync for LimineFramebufferRequest {}
unsafe impl Sync for LimineFramebuffer {}
unsafe impl Send for LimineFramebuffer {}

#[repr(C)]
pub struct LimineStackSizeRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *const LimineStackSizeResponse,
    pub stack_size: u64
}

#[repr(C)]
pub struct LimineStackSizeResponse {
    pub revision: u64
}

unsafe impl Sync for LimineStackSizeRequest {}
