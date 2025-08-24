use crate::arch::LogLevel;
use crate::arch::LOGGER;
use x86::bits64::paging::VAddr;
use x86::dtables::{lidt, DescriptorTablePointer};
use x86::segmentation::{SegmentSelector, SystemDescriptorTypes64};
use x86::Ring;
use crate::arch::memory::paging::page_fault_handler;
use crate::arch::kernel::interrupts::exceptions::{alignment_check_exception, bad_tss_exception, coprocessor_segment_overrun_exception, debug_exception, double_fault_exception, floating_point_exception, general_protection_exception, handle_divide_by_zero_exception, int0_exception, int3_exception, invalid_opcode_exception, machine_check_exception, nmi_exception, no_coprocessor_exception, out_of_bound_exception, reserved_exception, segment_not_present_exception, stack_fault_exception, ExceptionStackFrame};
use crate::arch::kernel::interrupts::handlers::{timer_handler, unhandled_irq1, unhandled_irq2};
use crate::sync::spinlock::SpinlockIrqSave;

const IDT_ENTRIES: usize = 256;
const KERNEL_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(1, Ring::Ring0);

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: u16,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl IdtEntry {
    pub const MISSING: IdtEntry = IdtEntry {
        pointer_low: 0,
        gdt_selector: SegmentSelector::from_raw(0),
        options: 0,
        pointer_middle: 0,
        pointer_high: 0,
        reserved: 0,
    };

    fn new(base: VAddr, selector: SegmentSelector, ring: Ring, gate_type: SystemDescriptorTypes64, ist: u8) -> Self {
        let base = base.as_u64();
        let options = ((ring as u16) << 13) | (1 << 15) | ((gate_type as u16) << 8) | ((ist as u16) & 0x7);
        Self {
            pointer_low: (base & 0xFFFF) as u16,
            gdt_selector: selector,
            options,
            pointer_middle: ((base >> 16) & 0xFFFF) as u16,
            pointer_high: (base >> 32) as u32,
            reserved: 0,
        }
    }
}

pub static INTERRUPT_HANDLER: SpinlockIrqSave<InterruptDescriptorTable> =
    SpinlockIrqSave::new(InterruptDescriptorTable::new());

pub struct InterruptDescriptorTable {
    idt: [IdtEntry; IDT_ENTRIES],
}

impl InterruptDescriptorTable {
    pub const fn new() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            idt: [IdtEntry::MISSING; IDT_ENTRIES],
        }
    }

    #[allow(dead_code)]
    pub fn add_handler(
        &mut self,
        int_no: usize,
        func: extern "x86-interrupt" fn(ExceptionStackFrame),
    ) {
        if int_no < IDT_ENTRIES {
            self.idt[int_no] = IdtEntry::new(
                VAddr::from_usize(func as usize),
                KERNEL_CODE_SELECTOR,
                Ring::Ring0,
                SystemDescriptorTypes64::InterruptGate,
                0,
            );
        } else {
            info!("unable to add handler for interrupt {}.", int_no);
        }
    }

    #[allow(dead_code)]
    pub fn remove_handler(&mut self, int_no: usize) {
        if int_no < IDT_ENTRIES {
            if int_no < 40 {
                self.idt[int_no] = IdtEntry::new(
                    VAddr::from_usize(unhandled_irq1 as usize),
                    KERNEL_CODE_SELECTOR,
                    Ring::Ring0,
                    SystemDescriptorTypes64::InterruptGate,
                    0,
                );
            } else {
                // send  eoi to the master and to the slave
                self.idt[int_no] = IdtEntry::new(
                    VAddr::from_usize(unhandled_irq2 as usize),
                    KERNEL_CODE_SELECTOR,
                    Ring::Ring0,
                    SystemDescriptorTypes64::InterruptGate,
                    0,
                );
            }
        } else {
            info!("unable to remove handler for interrupt {}.", int_no);
        }
    }

    pub unsafe fn load_idt(&mut self) {
        self.idt[0] = IdtEntry::new(VAddr::from_usize(handle_divide_by_zero_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[1] = IdtEntry::new(VAddr::from_usize(debug_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[2] = IdtEntry::new(VAddr::from_usize(nmi_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[3] = IdtEntry::new(VAddr::from_usize(int3_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[4] = IdtEntry::new(VAddr::from_usize(int0_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[5] = IdtEntry::new(VAddr::from_usize(out_of_bound_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[6] = IdtEntry::new(VAddr::from_usize(invalid_opcode_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[7] = IdtEntry::new(VAddr::from_usize(no_coprocessor_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[8] = IdtEntry::new(VAddr::from_usize(double_fault_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[9] = IdtEntry::new(VAddr::from_usize(coprocessor_segment_overrun_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[10] = IdtEntry::new(VAddr::from_usize(bad_tss_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[11] = IdtEntry::new(VAddr::from_usize(segment_not_present_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[12] = IdtEntry::new(VAddr::from_usize(stack_fault_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[13] = IdtEntry::new(VAddr::from_usize(general_protection_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[14] = IdtEntry::new(VAddr::from_usize(page_fault_handler as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[15] = IdtEntry::new(VAddr::from_usize(reserved_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[16] = IdtEntry::new(VAddr::from_usize(floating_point_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[17] = IdtEntry::new(VAddr::from_usize(alignment_check_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.idt[18] = IdtEntry::new(VAddr::from_usize(machine_check_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        for i in 19..32 {
            self.idt[i] = IdtEntry::new(VAddr::from_usize(reserved_exception as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        }
        self.idt[32] = IdtEntry::new(VAddr::from_usize(timer_handler as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);

        for i in 33..40 {
            self.idt[i] = IdtEntry::new(VAddr::from_usize(unhandled_irq1 as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        }
        for i in 40..IDT_ENTRIES {
            self.idt[i] = IdtEntry::new(VAddr::from_usize(unhandled_irq2 as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        }

        let idtr = DescriptorTablePointer::new(&self.idt);
        lidt(&idtr);
    }
}