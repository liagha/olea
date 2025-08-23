use {
    crate::{
        arch::{
            memory::{get_boot_stack, VirtAddr}
        },
        scheduler::{self, task::Stack},
    },
    core::mem,
    x86::{
        Ring,
        bits64::{
            task::*,
            segmentation::*,
        },
        controlregs::cr3_write,
        dtables::{self, DescriptorTablePointer},
        segmentation::*,
    },
};

const NULL: usize = 0;
const KERNEL_CODE: usize = 1;
const KERNEL_DATA: usize = 2;
const USER32_CODE: usize = 3;
const USER32_DATA: usize = 4;
#[cfg(target_arch = "x86_64")]
const USER64_CODE: usize = 5;
#[cfg(target_arch = "x86_64")]
const FIRST_TSS: usize = 6;
#[cfg(target_arch = "x86")]
const FIRST_TSS: usize = 5;

#[cfg(target_arch = "x86_64")]
const TSS_ENTRIES: usize = 2;
#[cfg(target_arch = "x86")]
const TSS_ENTRIES: usize = 1;

const GDT_ENTRIES: usize = FIRST_TSS + TSS_ENTRIES;

static mut GlobalDescriptorTableArray: [Descriptor; GDT_ENTRIES] = [Descriptor::NULL; GDT_ENTRIES];
static mut TaskStateSegmentWrapper: Tss = Tss::from(TaskStateSegment::new());

#[repr(align(128))]
pub(crate) struct Tss(TaskStateSegment);

impl Tss {
    #[allow(dead_code)]
    pub const fn into(self) -> TaskStateSegment {
        self.0
    }

    pub const fn from(x: TaskStateSegment) -> Self {
        Tss(x)
    }
}

pub(crate) fn init() {
    #[cfg(target_arch = "x86_64")]
    let limit = 0;
    #[cfg(target_arch = "x86")]
    let limit = 0xFFFF_FFFF;

    unsafe {
        GlobalDescriptorTableArray[NULL] = Descriptor::NULL;

        #[cfg(target_arch = "x86_64")]
        {
            GlobalDescriptorTableArray[KERNEL_CODE] =
                DescriptorBuilder::code_descriptor(0, limit, CodeSegmentType::ExecuteRead)
                    .present()
                    .dpl(Ring::Ring0)
                    .l()
                    .finish();
        }
        #[cfg(target_arch = "x86")]
        {
            GlobalDescriptorTableArray[KERNEL_CODE] =
                DescriptorBuilder::code_descriptor(0, limit, CodeSegmentType::ExecuteRead)
                    .present()
                    .dpl(Ring::Ring0)
                    .db()
                    .limit_granularity_4kb()
                    .finish();
        }

        GlobalDescriptorTableArray[KERNEL_DATA] = DescriptorBuilder::data_descriptor(0, 0, DataSegmentType::ReadWrite)
            .present()
            .dpl(Ring::Ring0)
            .finish();

        GlobalDescriptorTableArray[USER32_CODE] =
            DescriptorBuilder::code_descriptor(0, 0, CodeSegmentType::ExecuteRead)
                .present()
                .dpl(Ring::Ring3)
                .finish();

        GlobalDescriptorTableArray[USER32_DATA] = DescriptorBuilder::data_descriptor(0, 0, DataSegmentType::ReadWrite)
            .present()
            .dpl(Ring::Ring3)
            .finish();

        #[cfg(target_arch = "x86_64")]
        {
            GlobalDescriptorTableArray[USER64_CODE] =
                DescriptorBuilder::code_descriptor(0, 0, CodeSegmentType::ExecuteRead)
                    .present()
                    .dpl(Ring::Ring3)
                    .l()
                    .finish();
        }

        #[cfg(target_arch = "x86_64")]
        {
            let base = &TaskStateSegmentWrapper.0 as *const _ as u64;
            let tss_descriptor: Descriptor64 =
                <DescriptorBuilder as GateDescriptorBuilder<u64>>::tss_descriptor(
                    base,
                    base + mem::size_of::<TaskStateSegment>() as u64 - 1,
                    true,
                )
                    .present()
                    .dpl(Ring::Ring0)
                    .finish();

            GlobalDescriptorTableArray[FIRST_TSS..FIRST_TSS + TSS_ENTRIES]
                .copy_from_slice(&mem::transmute::<Descriptor64, [Descriptor; 2]>(
                    tss_descriptor,
                ));

            TaskStateSegmentWrapper.0.rsp[0] = get_boot_stack().interrupt_top().into();
        }
        #[cfg(target_arch = "x86")]
        {
            let base = &TaskStateSegmentWrapper.0 as *const _ as u64;
            let tss_descriptor: Descriptor =
                <DescriptorBuilder as GateDescriptorBuilder<u32>>::tss_descriptor(
                    base,
                    base + mem::size_of::<TaskStateSegment>() as u64 - 1,
                    true,
                )
                    .present()
                    .dpl(Ring::Ring0)
                    .finish();

            TaskStateSegmentWrapper.0.eflags = 0x1202;
            TaskStateSegmentWrapper.0.ss0 = 0x10;
            TaskStateSegmentWrapper.0.esp0 = get_boot_stack().interrupt_top().into();
            TaskStateSegmentWrapper.0.cs = 0x0b;

            GlobalDescriptorTableArray[FIRST_TSS] = tss_descriptor;
        }

        let gdtr = DescriptorTablePointer::new(&GlobalDescriptorTableArray);
        dtables::lgdt(&gdtr);

        load_cs(SegmentSelector::new(KERNEL_CODE as u16, Ring::Ring0));
        load_ss(SegmentSelector::new(KERNEL_DATA as u16, Ring::Ring0));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn set_kernel_stack(stack: VirtAddr) {
    TaskStateSegmentWrapper.0.rsp[0] = stack.as_u64();
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn set_kernel_stack(stack: VirtAddr) {
    TaskStateSegmentWrapper.0.esp = stack.as_u32();
}

pub(crate) unsafe extern "C" fn set_current_kernel_stack() {
    cr3_write(scheduler::get_root_page_table().as_u64());
    set_kernel_stack(scheduler::get_current_interrupt_stack());
}