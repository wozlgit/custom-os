#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, generic_const_exprs)]

pub mod cpuid;
pub mod graphics;
pub mod interrupts;
pub mod interrupts_general;
pub mod limine;
pub mod msr;
pub mod mtrr;
pub mod text_rendering;

use core::panic::PanicInfo;
use core::ptr::{self, null, null_mut};

use cpuid::is_cpuid_supported;
use limine::{LimineFramebuffer, LimineFramebufferRequest, LimineStackSizeRequest};
use msr::read_msr_only_low_order_32bits;
use spin::{Lazy, Mutex};

use crate::cpuid::{get_cpu_info, CPUBasicFeatureFlags};

LIMINE_BASE_REVISION! { 1 }

#[used]
static LIMINE_FB_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest {
    id: LIMINE_FRAMEBUFFER_REQUEST_ID!(),
    revision: 0,
    response: null_mut()
};

#[used]
static LIMINE_STACK_SIZE_REQUEST: LimineStackSizeRequest = LimineStackSizeRequest {
    id: LIMINE_STACK_SIZE_REQUEST_ID!(),
    revision: 0,
    response: null(),
    stack_size: 1024 * 512 // 512 KiB
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
    interrupts::load_idt();
    println!("CPUID support: {}", is_cpuid_supported());
    if is_cpuid_supported() {
        let ci = get_cpu_info();
        println!("Physical address bit width: {}", ci.physical_adress_bit_width);
        // println!("{:x}_{:x}", ci.family_id, ci.model);
        // mtrr::print_mtrr_memory_mappings(ci);
        let apic_supported = ci.feature_flags.contains(CPUBasicFeatureFlags::APIC);
        println!("Feature flag APIC: {}", apic_supported);
        if apic_supported {
            let apic_base_msr = read_msr_only_low_order_32bits(0x1b);
            let is_bsp = (apic_base_msr >> 8) & 0b1;
            let enabled = (apic_base_msr >> 11) & 0b1;
            let base_addr = apic_base_msr & 0xfffff000;
            println!("Is BSP: {}", is_bsp);
            println!("Is enabled: {}", enabled);
            println!("Base adress: {:X}", base_addr);
            let lapic_version_reg = unsafe { ptr::read(0xfee00030 as *const u32) };
            if lapic_version_reg & 0xf0 == 0x0 {
                println!("82489DX Discrete APIC");
            }
            else {
                println!("Integrated APIC");
            }
        }
    }
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
    println!("{}", panic_info);
    loop {}
}
