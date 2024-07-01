use core::arch::asm;
use core::mem::{size_of, transmute};
use core::ptr::slice_from_raw_parts;
use core::str::from_utf8;

use arrayvec::ArrayString;
use bitflags::bitflags;


/// Determine CPUID support by changing the ID bit in RFLAGS, and then reading RFLAGS to
/// determine if the CPU actually accepted the write.
pub fn is_cpuid_supported() -> bool {
    let mut modified_rflags: u64;
    unsafe {
        asm!(
            "pushfq",
            "pop {old_rflags:r}",
            old_rflags = out(reg) modified_rflags
        );
    }
    modified_rflags ^= 1u64 << 21; // Invert ID flag
    let new_rflags: u64;
    unsafe {
        asm!(
            "push {modified_rflags:r}",
            "popfq",
            "pushfq",
            "pop {new_rflags:r}",
            modified_rflags = in(reg) modified_rflags,
            new_rflags = out(reg) new_rflags
        );
    }
    // Can just compare them straight, without a bit mask, as the ID bit is the only one we changed
    modified_rflags == new_rflags
}

#[repr(C)]
struct CPUIDLeaf {
    r_eax: u32,
    r_ebx: u32,
    r_ecx: u32,
    r_edx: u32
}

impl CPUIDLeaf {
    #[inline]
    fn is_leaf_supported(&self) -> bool {
        !(self.r_eax == 0 && self.r_ebx == 0 && self.r_ecx == 0 && self.r_edx == 0)
    }
}

fn get_cpuid_leaf(input_eax: u32, input_ecx: u32) -> CPUIDLeaf {
    // Rust sadly doesn't support partial initialization of structs
    let mut output = CPUIDLeaf {
        r_eax: 0,
        r_ebx: 0,
        r_ecx: 0,
        r_edx: 0
    };
    unsafe {
        // EBX can not be used as a register operand, as it is used internally by LLVM. Since registers
        // which are not specified as output operands must have the same value at the end of the
        // asm block as they did at the start, I have to save RBX into a different register operand
        // at the start and then restore it from there at the end.
        asm!(
            "mov r10, rbx",
            "mov eax, {input_eax:e}",
            "mov ecx, {input_ecx:e}",
            "cpuid",
            "mov esi, ebx",
            "mov rbx, r10",
            out("eax") output.r_eax,
            out("ecx") output.r_ecx,
            out("edx") output.r_edx,
            out("r10") _,
            out("esi") output.r_ebx,
            input_eax = in(reg) input_eax,
            input_ecx = in(reg) input_ecx
        );
    }
    output
}

/// Actually any particular processor's CPUID is not guaranteed to support all the info here. For now I still
/// wont use any Option values here to make code simpler, as I'm quite confident only really old
/// processors don't support the information we need at the moment. So I'll just `panic!` if the
/// required information is not supported.
#[derive(Debug)]
pub struct CPUInfo {
    pub vendor_id_str: ArrayString<12>,
    highest_supported_basic_function: u32,
    stepping_id: u8,
    model: u8,
    family_id: u16,
    processor_type: ProcessorType,
    local_apic_id: u8,
    feature_flags: CPUBasicFeatureFlags
}

#[repr(u8)]
#[derive(Debug)]
enum ProcessorType {
    OriginalOEMProcessor = 0,
    IntelOverDriveProcessor = 1,
    DualProcessor = 2,
    IntelReserved = 3
}

bitflags! {
    #[derive(Debug)]
    struct CPUBasicFeatureFlags: u64 {
        const SSE3 = 1u64 << 0;
        const PCLMULQDQ = 1u64 << 1;
        const DTES64 = 1u64 << 2;
        const MONITOR = 1u64 << 3;
        const DS_CPL = 1u64 << 4;
        const VMX = 1u64 << 5;
        const SMX = 1u64 << 6;
        const EIST = 1u64 << 7;
        const TM2  = 1u64 << 8;
        const SSSE3 = 1u64 << 9;
        const CNXT_ID = 1u64 << 10;
        const SDBG  = 1u64 << 11;
        const FMA = 1u64 << 12;
        const CMPXCHG16B = 1u64 << 13;
        const xTPR_Update_Control = 1u64 << 14;
        const PDCM = 1u64 << 15;
        const PCID = 1u64 << 17;
        const DCA = 1u64 << 18;
        const SSE4_1 = 1u64 << 19;
        const SSE4_2 = 1u64 << 20;
        const x2APIC = 1u64 << 21;
        const MOVBE = 1u64 << 22;
        const POPCNT = 1u64 << 23;
        const TSC_Deadline = 1u64 << 24;
        const AESNI = 1u64 << 25;
        const XSAVE = 1u64 << 26;
        const OSXSAVE = 1u64 << 27;
        const AVX = 1u64 << 28;
        const F16C = 1u64 << 29;
        const RDRAND = 1u64 << 30;
        const FPU = 1u64 << 32;
        const VME = 1u64 << 33;
        const DE = 1u64 << 34;
        const PSE = 1u64 << 35;
        const TSC = 1u64 << 36;
        const MSR = 1u64 << 37;
        const PAE = 1u64 << 38;
        const MCE = 1u64 << 39;
        const CX8 = 1u64 << 40;
        const APIC = 1u64 << 41;
        const SEP = 1u64 << 43;
        const MTRR = 1u64 << 44;
        const PGE = 1u64 << 45;
        const MCA = 1u64 << 46;
        const CMOV = 1u64 << 47;
        const PAT = 1u64 << 48;
        const PSE_36 = 1u64 << 49;
        const PSN = 1u64 << 50;
        const CLFSH = 1u64 << 51;
        const DS = 1u64 << 53;
        const ACPI = 1u64 << 54;
        const MMX = 1u64 << 55;
        const FXSR = 1u64 << 56;
        const SSE = 1u64 << 57;
        const SSE2 = 1u64 << 58;
        const SS = 1u64 << 59;
        const HTT = 1u64 << 60;
        const TM = 1u64 << 61;
        const PBE = 1u64 << 63;
    }
}

impl AsSliceOf<u8> for CPUIDLeaf {}
impl AsSliceOf<u32> for CPUIDLeaf {}

pub fn get_cpu_info() -> CPUInfo {
    let leaf_0 = get_cpuid_leaf(0, 0);
    if !leaf_0.is_leaf_supported() {
        panic!("Required CPUID leaf not supported");
    }
    let highest_supported_basic_function = leaf_0.r_eax;
    let leaf_bytes: &[u8; 16] = leaf_0.as_slice_of();
    let s = from_utf8(&leaf_bytes[4..16]).unwrap();
    let mut vendor_id_str: ArrayString<12> = ArrayString::new_const();
    vendor_id_str.push_str(&s[0..4]);
    vendor_id_str.push_str(&s[8..12]);
    vendor_id_str.push_str(&s[4..8]);

    let leaf_1;
    if 1 <= highest_supported_basic_function {
        leaf_1 = get_cpuid_leaf(1, 0);
        if !leaf_1.is_leaf_supported() {
            panic!("Required CPUID leaf not supported");
        }
    }
    else {
        panic!("Required CPUID leaf not supported");
    }
    let stepping_id = (leaf_1.r_eax & 0x0000000f) as u8;
    let mut family_id = ((leaf_1.r_eax >> 8) & 0x0000000f) as u16;
    // model-family-id order here on purpose, the if-statement here should depend on just the
    // family id field, not the "full" family id obtained by adding the extended family id field to
    // it
    let mut model = ((leaf_1.r_eax >> 4) & 0x0000000f) as u8;
    if family_id == 0x0f || family_id == 0x06 {
        model += ((leaf_1.r_eax >> 12) & 0x000000f0) as u8;
    }
    if family_id == 0x0f {
        family_id += ((leaf_1.r_eax >> 20) & 0x000000ff) as u16;
    }
    let processor_type: ProcessorType = unsafe { transmute(((leaf_1.r_eax >> 12) & 0x00000003) as u8) };
    let local_apic_id = ((leaf_1.r_ebx >> 24) & 0x000000ff) as u8;
    let feature_flags =
        CPUBasicFeatureFlags::from_bits_retain(((leaf_1.r_edx as u64) << 32) | (leaf_1.r_ecx as u64));
    CPUInfo {
        vendor_id_str,
        highest_supported_basic_function,
        stepping_id,
        model,
        family_id,
        processor_type,
        local_apic_id,
        feature_flags
    }
}

pub trait AsSliceOf<R>
where
    Self: Sized
{
    const ELEMENT_COUNT: usize = size_of::<Self>() / size_of::<R>();

    /// `mem::size_of::<Self>()` must not be larger than isize::MAX, nor can the sum of
    /// that size and the address of `self`, as per the safety requirements of `slice::from_raw_parts`.
    fn as_slice_of(&self) -> &[R; Self::ELEMENT_COUNT] {
        let sp =
            slice_from_raw_parts(self as *const Self, Self::ELEMENT_COUNT) as *const [R; Self::ELEMENT_COUNT];
        unsafe { &*sp }
    }
}
