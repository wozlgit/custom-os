use core::arch::asm;
use core::ops::{Index, IndexMut};

use spin::Lazy;

use crate::println;

#[repr(C, align(16))]
struct InterruptDescriptor {
    handler_address_0_15: u16,
    handler_cs: u16,
    options: u16,
    handler_address_16_31: u16,
    handler_address_32_63: u32,
    reserved: u32
}

impl InterruptDescriptor {
    #[inline]
    fn new_internal(address: usize, code_segment: u16, options: u16) -> InterruptDescriptor {
        let addr_ptr = &address as *const usize;
        InterruptDescriptor {
            handler_address_0_15: unsafe { *(addr_ptr as *const u16) },
            handler_address_16_31: unsafe { *(addr_ptr.byte_add(2) as *const u16) },
            handler_address_32_63: unsafe { *(addr_ptr.byte_add(4) as *const u32) },
            reserved: 0,
            options,
            handler_cs: code_segment
        }
    }

    #[inline]
    fn with_options(address: usize, options: u16) -> InterruptDescriptor {
        let mut id = InterruptDescriptor::new_internal(address as usize, 0, options);
        // Encode code segment selector (just using the current code segment, since in 64-bit
        // mode segmentation isn't used, and thus there's not really a reason to have multiple
        // code segments
        let segment_selector_output_addr = &mut id.handler_cs as *mut u16 as usize;
        unsafe {
            asm!(
                "mov [{segment_selector_addr:r}], CS",
                segment_selector_addr = in(reg) segment_selector_output_addr
            );
        }
        id
    }

    pub fn new(address: *const ()) -> InterruptDescriptor {
        let options = InterruptDescriptorFlags::Present as u16
            | InterruptDescriptorFlags::TypeInterruptGate as u16
            | InterruptDescriptorFlags::dpl(3)
            | InterruptDescriptorFlags::ist(0);
        InterruptDescriptor::with_options(address as usize, options)
    }

    pub const fn empty() -> InterruptDescriptor {
        InterruptDescriptor {
            handler_address_0_15: 0,
            handler_address_16_31: 0,
            handler_address_32_63: 0,
            options: 0,
            reserved: 0,
            handler_cs: 0
        }
    }
}

#[repr(u16)]
enum InterruptDescriptorFlags {
    Present = 0b1000000000000000,
    TypeInterruptGate = 0b0000111000000000,
    TypeTrapGate = 0b0000111100000000
}

impl InterruptDescriptorFlags {
    #[inline]
    const fn dpl(dpl: u8) -> u16 {
        (dpl as u16) << 13
    }

    #[inline]
    const fn ist(ist: u8) -> u16 {
        ist as u16
    }
}

struct Idt([InterruptDescriptor; 256]);

impl Idt {
    fn new() -> Idt {
        Idt([const { InterruptDescriptor::empty() }; 256])
    }
}

impl Index<usize> for Idt {
    type Output = InterruptDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Idt {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

static IDT: Lazy<Idt> = Lazy::new(|| {
    let mut idt = Idt::new();
    idt[3] = InterruptDescriptor::new(breakpoint_interrupt as *const ());
    idt[4] = InterruptDescriptor::new(double_fault_interrupt as *const ());
    idt
});

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    address: u64
}

fn load_idt_general(idt_addr: *const (), idt_size_in_bytes: u16) {
    let idtr = Idtr {
        limit: idt_size_in_bytes,
        address: idt_addr as usize as u64
    };
    let idtr_addr = &idtr as *const Idtr as usize;
    unsafe {
        asm!(
            "lidt [{idtr_mem_addr}]",
            idtr_mem_addr = in(reg) idtr_addr
        );
    }
}

pub fn load_idt() {
    load_idt_general(
        &(*IDT) as *const Idt as *const (),
        core::mem::size_of::<Idt>() as u16
    );
}

#[repr(C)]
#[derive(Debug)]
struct InterruptStackFrame {
    return_ss: u16,
    return_stack_pointer: u64,
    rflags: u64,
    return_cs: u16,
    return_address: u64
}

extern "x86-interrupt" fn breakpoint_interrupt(stack_frame: InterruptStackFrame) {
    println!(
        "EXCEPTION: Breakpoint exception occurred! Stack frame: \n{:#?}",
        stack_frame
    );
}

extern "x86-interrupt" fn double_fault_interrupt(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!(
        "EXCEPTION: Double fault occurred! Stack frame: \n{:#?}",
        stack_frame
    );
}
