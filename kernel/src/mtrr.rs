use core::mem::transmute;

use crate::cpuid::{CPUBasicFeatureFlags, CPUInfo};
use crate::msr::{read_msr, read_msr_only_low_order_32bits};
use crate::{print, println};

#[repr(u8)]
#[derive(Debug)]
pub enum MTRRMemoryType {
    Uncacheable = 0x00,
    WriteCombining = 0x01,
    WriteThrough = 0x04,
    WriteProtected = 0x05,
    Writeback = 0x06
}

impl MTRRMemoryType {
    pub fn from_u8(value: u8) -> MTRRMemoryType {
        unsafe { transmute::<_, MTRRMemoryType>(value) }
    }
}

pub fn print_mtrr_fixed_range_reg(reg_value: u64) {
    for i in 0..8 {
        print!(
            "{:?}",
            MTRRMemoryType::from_u8(((reg_value >> i * 8) & 0xff) as u8)
        );
        if i != 7 {
            print!(", ");
        }
    }
    println!();
}

pub fn print_mtrr_memory_mappings(ci: CPUInfo) {
    if ci.feature_flags.contains(CPUBasicFeatureFlags::MSR)
        && ci.feature_flags.contains(CPUBasicFeatureFlags::MTRR)
    {
        let mtrr_cap_msr = read_msr_only_low_order_32bits(0xfe);
        println!("IA32_MTRR_CAP_MSR: {:b}", mtrr_cap_msr & 0x00001fff);
        let num_variable_range_mtrrs = mtrr_cap_msr & 0x000000ff;
        println!("Num variable-range MTRRs: {}", num_variable_range_mtrrs);
        let mtrr_def_type_msr = read_msr_only_low_order_32bits(0x2ff);
        println!("IA32_MTRR_DEF_TYPE_MSR: {:b}", mtrr_def_type_msr & 0x00000fff);
        println!(
            "Default memory type: {:?}",
            MTRRMemoryType::from_u8((mtrr_def_type_msr & 0x00000007) as u8)
        );
        let mtrr_fixed_64k_msr = read_msr(0x250);
        println!("Address range: 0x0 - 0x7ffff, in 64kB chunks");
        print_mtrr_fixed_range_reg(mtrr_fixed_64k_msr);
        println!("Address range: 0x80000 - 0xbffff, in 16kB chunks");
        for i in 0..2 {
            let mtrr_fixed_16k_msr = read_msr(0x258 + i);
            println!(
                "Address range: {:x} - {:x}",
                0x80000 + 128 * 1024 * i,
                0x80000 + 128 * 1024 * (i + 1)
            );
            print_mtrr_fixed_range_reg(mtrr_fixed_16k_msr);
        }
        println!("Address range: 0xc0000 - 0xfffff, in 4kB chunks");
        for i in 0..7 {
            let mtrr_fixed_4k_msr = read_msr(0x269 + i);
            println!(
                "Address range: {:x} - {:x}",
                0xc0000 + 4 * 1024 * i,
                0xc0000 + 4 * 1024 * (i + 1)
            );
            print_mtrr_fixed_range_reg(mtrr_fixed_4k_msr);
        }

        for i in 0..num_variable_range_mtrrs {
            let physbase = read_msr(0x200 + i * 2);
            let physmask = read_msr(0x201 + i * 2);
            let addr_or_mask_obtaining_mask =
                (u64::MAX >> (64 - ci.physical_adress_bit_width)) & 0xfffffffffffff000;
            let mask = physmask & addr_or_mask_obtaining_mask;
            let base_adress = physbase & addr_or_mask_obtaining_mask;
            let valid_flag = (physmask >> 11) & 0x1;
            print!("Variable range MTRR {}: ", i);
            if valid_flag == 0 {
                println!("Invalid");
                continue;
            }
            else {
                println!();
            }
            println!(
                "Memory type: {:?}",
                MTRRMemoryType::from_u8((physbase & 0xff) as u8)
            );
            println!("Base address: {:x}", base_adress);
            println!("Mask: {:x}", mask);
        }
    }
}
