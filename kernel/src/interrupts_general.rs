use core::arch::asm;
use core::fmt::Debug;
use core::ops::{Index, IndexMut};

#[repr(C, align(16))]
pub struct InterruptDescriptor {
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

    pub fn with_options(address: usize, options: u16) -> InterruptDescriptor {
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
pub enum InterruptDescriptorFlags {
    Present = 0b1000000000000000,
    TypeInterruptGate = 0b0000111000000000,
    TypeTrapGate = 0b0000111100000000
}

impl InterruptDescriptorFlags {
    #[inline]
    pub const fn dpl(dpl: u8) -> u16 {
        (dpl as u16) << 13
    }

    #[inline]
    pub const fn ist(ist: u8) -> u16 {
        ist as u16
    }
}

#[repr(transparent)]
pub struct InterruptHandler(InterruptDescriptor);
impl InterruptHandler {
    pub fn set_to_handler(&mut self, handler_addr: extern "x86-interrupt" fn(InterruptStackFrame)) {
        self.0 = InterruptDescriptor::new(handler_addr as *const ());
    }

    pub const fn empty() -> InterruptHandler {
        InterruptHandler(InterruptDescriptor::empty())
    }
}
#[repr(transparent)]
pub struct InterruptHandlerWithErrorCode(InterruptDescriptor);
impl InterruptHandlerWithErrorCode {
    pub fn set_to_handler(&mut self, handler_addr: extern "x86-interrupt" fn(InterruptStackFrame, u64)) {
        self.0 = InterruptDescriptor::new(handler_addr as *const ());
    }

    pub const fn empty() -> InterruptHandlerWithErrorCode {
        InterruptHandlerWithErrorCode(InterruptDescriptor::empty())
    }
}
#[repr(transparent)]
pub struct AbortInterruptHandlerWithErrorCode(InterruptDescriptor);
impl AbortInterruptHandlerWithErrorCode {
    pub fn set_to_handler(&mut self, handler_addr: extern "x86-interrupt" fn(InterruptStackFrame, u64) -> !) {
        self.0 = InterruptDescriptor::new(handler_addr as *const ());
    }

    pub const fn empty() -> AbortInterruptHandlerWithErrorCode {
        AbortInterruptHandlerWithErrorCode(InterruptDescriptor::empty())
    }
}

#[repr(C)]
pub struct Idt {
    pub divide_by_zero: InterruptHandler,
    pub debug: InterruptHandler,
    pub non_maskable_interrupt: InterruptHandler,
    pub breakpoint: InterruptHandler,
    pub overflow: InterruptHandler,
    pub bound_range: InterruptHandler,
    pub invalid_opcode: InterruptHandler,
    pub device_not_available: InterruptHandler,
    pub double_fault: AbortInterruptHandlerWithErrorCode,
    _reserved0: InterruptDescriptor,
    pub invalid_tss: InterruptHandlerWithErrorCode,
    pub segment_not_present: InterruptHandlerWithErrorCode,
    pub stack: InterruptHandlerWithErrorCode,
    pub general_protection: InterruptHandlerWithErrorCode,
    pub page_fault: InterruptHandlerWithErrorCode,
    _reserved1: InterruptDescriptor,
    pub floating_point_exception_pending: InterruptHandler,
    pub alignment_check: InterruptHandler,
    pub machine_check: InterruptHandler,
    pub simd_floating_point: InterruptHandler,
    _reserved2: InterruptDescriptor,
    pub control_protection: InterruptHandlerWithErrorCode,
    _reserved3: [InterruptDescriptor; 6],
    pub hypervisor_injection: InterruptHandler, // These 3 werent actually specified in the
    // interrupt table
    pub vmm_communication: InterruptHandler,
    pub security_exception: InterruptHandler,
    _reserved4: InterruptDescriptor,
    user_defined: [InterruptHandler; 224]
}

impl Idt {
    pub fn new() -> Idt {
        Idt {
            divide_by_zero: InterruptHandler::empty(),
            debug: InterruptHandler::empty(),
            non_maskable_interrupt: InterruptHandler::empty(),
            breakpoint: InterruptHandler::empty(),
            overflow: InterruptHandler::empty(),
            bound_range: InterruptHandler::empty(),
            invalid_opcode: InterruptHandler::empty(),
            device_not_available: InterruptHandler::empty(),
            double_fault: AbortInterruptHandlerWithErrorCode::empty(),
            _reserved0: InterruptDescriptor::empty(),
            invalid_tss: InterruptHandlerWithErrorCode::empty(),
            segment_not_present: InterruptHandlerWithErrorCode::empty(),
            stack: InterruptHandlerWithErrorCode::empty(),
            general_protection: InterruptHandlerWithErrorCode::empty(),
            page_fault: InterruptHandlerWithErrorCode::empty(),
            _reserved1: InterruptDescriptor::empty(),
            floating_point_exception_pending: InterruptHandler::empty(),
            alignment_check: InterruptHandler::empty(),
            machine_check: InterruptHandler::empty(),
            simd_floating_point: InterruptHandler::empty(),
            _reserved2: InterruptDescriptor::empty(),
            control_protection: InterruptHandlerWithErrorCode::empty(),
            _reserved3: [const { InterruptDescriptor::empty() }; 6],
            hypervisor_injection: InterruptHandler::empty(),
            vmm_communication: InterruptHandler::empty(),
            security_exception: InterruptHandler::empty(),
            _reserved4: InterruptDescriptor::empty(),
            user_defined: [const { InterruptHandler::empty() }; 224]
        }
    }

    pub fn load(&self) {
        let idtr = Idtr {
            limit: core::mem::size_of::<Idt>() as u16,
            address: self as *const Idt as usize as u64
        };
        let idtr_addr = &idtr as *const Idtr as usize;
        unsafe {
            asm!(
                "lidt [{idtr_mem_addr}]",
                idtr_mem_addr = in(reg) idtr_addr
            );
        }
    }
}

impl Index<usize> for Idt {
    type Output = InterruptHandler;

    fn index(&self, index: usize) -> &Self::Output {
        if index < 32 {
            panic!(
                "ERROR: Processor defined interrupt handlers should not be accessed by index, but rather \
                 throught the Idt struct's corresponding field."
            );
        }
        &self.user_defined[index - 32]
    }
}

impl IndexMut<usize> for Idt {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < 32 {
            panic!(
                "ERROR: Processor defined interrupt handlers should not be accessed by index, but rather \
                 throught the Idt struct's corresponding field."
            );
        }
        &mut self.user_defined[index - 32]
    }
}

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    address: u64
}

#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    return_ss: SegmentSelector,
    return_stack_pointer: u64,
    rflags: u64,
    return_cs: SegmentSelector,
    return_address: u64
}

#[repr(transparent)]
pub struct SegmentSelector(pub u16);
impl Debug for SegmentSelector {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0 >> 1 == 0 {
            return write!(f, "SegmentSelector(null selector)");
        }
        write!(
            f,
            "SegmentSelector(RPL = {}, descriptor_table = {}, descriptor_index = {})",
            self.0 & 0b11,
            if self.0 & 0b100 == 0 { "GDT" } else { "LDT" },
            self.0 >> 3
        )
    }
}

#[repr(transparent)]
pub struct SegmentSelectorErrorCode(pub u16);
impl Debug for SegmentSelectorErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "SegmentSelectorErrorCode(EXT = {}, descriptor_table = {}, descriptor_index = {})",
            match self.0 & 0b1 {
                0 => "internal",
                _ => "external"
            },
            match self.0 & 0b10 {
                0 => match self.0 & 0b100 {
                    0 => "GDT",
                    _ => "LDT"
                },
                _ => "IDT"
            },
            self.0 >> 3
        )
    }
}
