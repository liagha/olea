use core::intrinsics::write_bytes;
use x86::controlregs;
use x86::irq::PageFaultError;
use {
    crate::{
        scheduler::*,
        arch::{
            kernel::{
                interrupts::{
                    end_of_interrupt, MASTER, SLAVE,
                    exceptions::ExceptionStackFrame,
                },
            },
        },
    },
};
use crate::arch::memory::{physical, VirtualAddress};
use crate::arch::memory::paging::{map, BasePageSize, PageSize, PageTableEntryFlags};
use crate::consts::USER_ENTRY;
use crate::scheduler;

pub extern "x86-interrupt" fn unhandled_irq1(stack_frame: ExceptionStackFrame, irq: u64) {
    info!("task {} receive a unhandled IRQ: {} {:#?}.", get_current_taskid(), irq, stack_frame);
    end_of_interrupt(MASTER);
}

pub extern "x86-interrupt" fn unhandled_irq2(stack_frame: ExceptionStackFrame, irq: u64) {
    info!("task {} receive a unhandled IRQ: {} {:#?}.", get_current_taskid(), irq, stack_frame);
    end_of_interrupt(SLAVE);
    end_of_interrupt(MASTER);
}

pub extern "x86-interrupt" fn timer(stack_frame: ExceptionStackFrame) {
    debug!(
		"task {} receive timer interrupt!\n{:#?}.",
		get_current_taskid(),
		stack_frame
	);

    end_of_interrupt(MASTER);
    schedule();
}

pub extern "x86-interrupt" fn page_fault(stack_frame: ExceptionStackFrame, error_code: u64) {
    let mut virtual_address = unsafe { VirtualAddress::from_usize(controlregs::cr2()) };

    if virtual_address > USER_ENTRY + 0x400000u64 - 64u64 * 1024u64 {
        virtual_address = align_down!(virtual_address, BasePageSize::SIZE);

        let physical_address = physical::allocate_aligned(BasePageSize::SIZE, BasePageSize::SIZE);

        debug!("map 0x{:x} into the user space at 0x{:x}.", physical_address, virtual_address);

        map::<BasePageSize>(
            virtual_address,
            physical_address,
            1,
            PageTableEntryFlags::WRITABLE | PageTableEntryFlags::USER_ACCESSIBLE | PageTableEntryFlags::EXECUTE_DISABLE,
        );

        unsafe {
            write_bytes(virtual_address.as_mut_ptr::<u8>(), 0x00, BasePageSize::SIZE);
            controlregs::cr2_write(0);
        }

        end_of_interrupt(MASTER);
    } else {
        let pferror = PageFaultError::from_bits_truncate(error_code as u32);

        error!("page fault (#PF) Exception: {:#?}.", stack_frame);
        error!("virtual_address = {:#X}, page fault error = {}.", virtual_address, pferror);

        unsafe {
            controlregs::cr2_write(0);
        }

        end_of_interrupt(MASTER);
        scheduler::abort();
    }
}
