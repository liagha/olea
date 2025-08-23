use crate::arch::memory::get_boot_stack;
use crate::arch::x86::kernel::system_calls::handler::handle_system_call;
use crate::logging::*;
use core::arch::asm;
use x86::controlregs::*;
use x86::cpuid::*;
use x86::msr::*;
use crate::scheduler::task::Stack;

const EFER_SCE: u64 = 1 << 0;
const EFER_LME: u64 = 1 << 8;
const EFER_LMA: u64 = 1 << 10;
const EFER_NXE: u64 = 1 << 11;
const EFER_SVME: u64 = 1 << 12;
const EFER_LMSLE: u64 = 1 << 13;
const EFER_FFXSR: u64 = 1 << 14;
const EFER_TCE: u64 = 1 << 15;

static mut PHYSICAL_ADDRESS_BITS: u8 = 0;
static mut LINEAR_ADDRESS_BITS: u8 = 0;
static mut SUPPORTS_1GIB_PAGES: bool = false;

pub(crate) fn supports_1gib_pages() -> bool {
    unsafe { SUPPORTS_1GIB_PAGES }
}

pub(crate) fn get_linear_address_bits() -> u8 {
    unsafe { LINEAR_ADDRESS_BITS }
}

pub(crate) fn get_physical_address_bits() -> u8 {
    unsafe { PHYSICAL_ADDRESS_BITS }
}

pub(crate) fn enable_features() {
    let cpuid = CpuId::new();
    let mut cr0 = unsafe { cr0() };

    cr0 |= Cr0::CR0_ALIGNMENT_MASK;
    cr0 |= Cr0::CR0_NUMERIC_ERROR;
    cr0 |= Cr0::CR0_MONITOR_COPROCESSOR;
    cr0 &= !(Cr0::CR0_CACHE_DISABLE | Cr0::CR0_NOT_WRITE_THROUGH);

    unsafe { cr0_write(cr0) };

    let mut cr4 = unsafe { cr4() };

    let has_pge = match cpuid.get_feature_info() {
        Some(finfo) => finfo.has_pge(),
        None => false,
    };

    if has_pge {
        cr4 |= Cr4::CR4_ENABLE_GLOBAL_PAGES;
    }

    let has_fsgsbase = match cpuid.get_extended_feature_info() {
        Some(efinfo) => efinfo.has_fsgsbase(),
        None => false,
    };

    if has_fsgsbase {
        cr4 |= Cr4::CR4_ENABLE_FSGSBASE;
    } else {
        panic!("Olea-Base requires the CPU feature FSGSBASE");
    }

    let has_mce = match cpuid.get_feature_info() {
        Some(finfo) => finfo.has_mce(),
        None => false,
    };

    if has_mce {
        cr4 |= Cr4::CR4_ENABLE_MACHINE_CHECK;
    }

    cr4 &= !(Cr4::CR4_ENABLE_PPMC | Cr4::CR4_TIME_STAMP_DISABLE);

    unsafe { cr4_write(cr4) };

    let has_syscall = match cpuid.get_extended_processor_and_feature_identifiers() {
        Some(finfo) => finfo.has_syscall_sysret(),
        None => false,
    };

    if !has_syscall {
        panic!("syscall support is missing.");
    }

    unsafe {
        wrmsr(IA32_EFER, rdmsr(IA32_EFER) | EFER_LMA | EFER_SCE | EFER_NXE);
        wrmsr(IA32_STAR, (0x1Bu64 << 48) | (0x08u64 << 32));
        wrmsr(IA32_LSTAR, (handle_system_call as usize).try_into().unwrap());
        wrmsr(IA32_FMASK, 1 << 9);

        wrmsr(IA32_GS_BASE, 0);
        asm!("wrgsbase {}", in(reg) get_boot_stack().top().as_u64(), options(preserves_flags, nomem, nostack));
    }

    let extended_feature_info = cpuid
        .get_processor_capacity_feature_info()
        .expect("CPUID Capacity Feature Info is not available!");
    unsafe {
        PHYSICAL_ADDRESS_BITS = extended_feature_info.physical_address_bits();
        LINEAR_ADDRESS_BITS = extended_feature_info.linear_address_bits();
        SUPPORTS_1GIB_PAGES = cpuid
            .get_extended_processor_and_feature_identifiers()
            .expect("CPUID Extended Processor and Feature Info is not available!")
            .has_1gib_pages();
    }

    if supports_1gib_pages() {
        info!("system supports 1GiB pages.");
    }
    debug!("physical address bits {}.", get_physical_address_bits());
    debug!("linear address bits {}.", get_linear_address_bits());
    debug!("CR0: {:?}.", cr0);
    debug!("CR4: {:?}.", cr4);
}