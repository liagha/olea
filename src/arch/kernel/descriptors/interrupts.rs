use {
    crate::{
        sync::spinlock::SpinlockIrqSave,
        arch::{
            x86::*,
            memory::paging::page_fault_handler,
            kernel::{
                interrupts::{
                    handlers::{timer_handler, unhandled_irq1, unhandled_irq2},
                    exceptions::{alignment_check, bad_tss, coprocessor_segment_overrun, debug, double_fault, floating_point, general_protection, divide_by_zero, int_zero, int_three, invalid_opcode, machine_check, non_maskable, no_coprocessor, out_of_bound, reserved, segment_not_present, stack_fault, ExceptionStackFrame}
                }
            },
        }
    }
};

const INTERRUPT_ENTRIES: usize = 256;
const KERNEL_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(1, Ring::Ring0);

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct InterruptEntry {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: u16,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl InterruptEntry {
    pub const MISSING: InterruptEntry = InterruptEntry {
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
    interrupts: [InterruptEntry; INTERRUPT_ENTRIES],
}

impl InterruptDescriptorTable {
    pub const fn new() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            interrupts: [InterruptEntry::MISSING; INTERRUPT_ENTRIES],
        }
    }

    #[allow(dead_code)]
    pub fn add_handler(
        &mut self,
        int_no: usize,
        func: extern "x86-interrupt" fn(ExceptionStackFrame),
    ) {
        if int_no < INTERRUPT_ENTRIES {
            self.interrupts[int_no] = InterruptEntry::new(
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
        if int_no < INTERRUPT_ENTRIES {
            if int_no < 40 {
                self.interrupts[int_no] = InterruptEntry::new(
                    VAddr::from_usize(unhandled_irq1 as usize),
                    KERNEL_CODE_SELECTOR,
                    Ring::Ring0,
                    SystemDescriptorTypes64::InterruptGate,
                    0,
                );
            } else {
                // send  eoi to the master and to the slave
                self.interrupts[int_no] = InterruptEntry::new(
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

    pub unsafe fn load_interrupts(&mut self) {
        self.interrupts[0] = InterruptEntry::new(VAddr::from_usize(divide_by_zero as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[1] = InterruptEntry::new(VAddr::from_usize(debug as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[2] = InterruptEntry::new(VAddr::from_usize(non_maskable as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[3] = InterruptEntry::new(VAddr::from_usize(int_three as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[4] = InterruptEntry::new(VAddr::from_usize(int_zero as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[5] = InterruptEntry::new(VAddr::from_usize(out_of_bound as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[6] = InterruptEntry::new(VAddr::from_usize(invalid_opcode as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[7] = InterruptEntry::new(VAddr::from_usize(no_coprocessor as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[8] = InterruptEntry::new(VAddr::from_usize(double_fault as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[9] = InterruptEntry::new(VAddr::from_usize(coprocessor_segment_overrun as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[10] = InterruptEntry::new(VAddr::from_usize(bad_tss as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[11] = InterruptEntry::new(VAddr::from_usize(segment_not_present as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[12] = InterruptEntry::new(VAddr::from_usize(stack_fault as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[13] = InterruptEntry::new(VAddr::from_usize(general_protection as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[14] = InterruptEntry::new(VAddr::from_usize(page_fault_handler as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[15] = InterruptEntry::new(VAddr::from_usize(reserved as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[16] = InterruptEntry::new(VAddr::from_usize(floating_point as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[17] = InterruptEntry::new(VAddr::from_usize(alignment_check as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        self.interrupts[18] = InterruptEntry::new(VAddr::from_usize(machine_check as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        for i in 19..32 {
            self.interrupts[i] = InterruptEntry::new(VAddr::from_usize(reserved as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        }
        self.interrupts[32] = InterruptEntry::new(VAddr::from_usize(timer_handler as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);

        for i in 33..40 {
            self.interrupts[i] = InterruptEntry::new(VAddr::from_usize(unhandled_irq1 as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        }
        for i in 40..INTERRUPT_ENTRIES {
            self.interrupts[i] = InterruptEntry::new(VAddr::from_usize(unhandled_irq2 as usize), KERNEL_CODE_SELECTOR, Ring::Ring0, SystemDescriptorTypes64::InterruptGate, 0);
        }

        let idtr = DescriptorTablePointer::new(&self.interrupts);
        lidt(&idtr);
    }
}