use core::arch::asm;
use core::panic;

use spin::Lazy;

use crate::interrupts_general::{Idt, InterruptStackFrame, SegmentSelectorErrorCode};
use crate::println;

static IDT: Lazy<Idt> = Lazy::new(|| {
    let mut idt = Idt::new();
    idt.breakpoint.set_to_handler(breakpoint_interrupt);
    idt.double_fault.set_to_handler(double_fault_interrupt);
    idt.general_protection
        .set_to_handler(general_protection_interrupt);
    idt.segment_not_present
        .set_to_handler(segment_not_present_interrupt);
    idt.page_fault.set_to_handler(page_fault);
    idt
});

pub fn load_idt() {
    IDT.load();
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

extern "x86-interrupt" fn general_protection_interrupt(stack_frame: InterruptStackFrame, error_code: u64) {
    match error_code {
        0 => panic!(
            "EXCEPTION: General protection occurred! Stack frame: \n{:#?}\nError not related to a segment \
             descriptor access.",
            stack_frame,
        ),
        _ => panic!(
            "EXCEPTION: General protection occurred! Stack frame: \n{:#?}\nError related to a segment \
             descriptor access. Segment descriptor in question: {:?}",
            stack_frame,
            SegmentSelectorErrorCode(error_code as u16)
        )
    }
}

extern "x86-interrupt" fn segment_not_present_interrupt(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!(
        "EXCEPTION: Segment not present occurred! Stack frame: \n{:#?}\nError code: {:?}",
        stack_frame,
        SegmentSelectorErrorCode(error_code as u16)
    );
}

extern "x86-interrupt" fn page_fault(stack_frame: InterruptStackFrame, error_code: u64) {
    let reg_cr2: u64;
    unsafe {
        asm!(
            "mov {cr2_contents}, CR2",
            cr2_contents = out(reg) reg_cr2
        );
    }
    panic!(
        "EXCEPTION: Page fault occurred! Stack frame:\n{:#?}\nError code: {:b}\nAdress of memory access \
         that generated the page fault: {:X}\n",
        stack_frame, error_code, reg_cr2
    );
}
