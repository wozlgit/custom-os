use core::arch::asm;

pub fn read_msr(address: u32) -> u64 {
    let msr_value_low: u32;
    let msr_value_high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") address,
            out("eax") msr_value_low,
            out("edx") msr_value_high
        )
    }
    ((msr_value_high as u64) << 32) | (msr_value_low as u64)
}

pub fn read_msr_only_low_order_32bits(address: u32) -> u32 {
    let msr_value_low: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") address,
            out("eax") msr_value_low,
        )
    }
    msr_value_low
}
