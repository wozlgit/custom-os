use core::panic;

use spin::Lazy;

use crate::interrupts_general::{Idt, InterruptStackFrame};
use crate::println;

static IDT: Lazy<Idt> = Lazy::new(|| {
    let mut idt = Idt::new();
    idt.breakpoint.set_to_handler(breakpoint_interrupt);
    idt.double_fault.set_to_handler(double_fault_interrupt);
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
