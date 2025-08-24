use {
    crate::{
        arch::{
            x86::*,
            memory::{get_boot_stack, VirtAddr}
        },
        scheduler::{self, task::Stack},
    },
    core::mem,
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

static mut GLOBAL_DESCRIPTOR_TABLE_ARRAY: [Descriptor; GDT_ENTRIES] = [Descriptor::NULL; GDT_ENTRIES];
static mut TASK_STATE_SEGMENT_WRAPPER: Tss = Tss::from(TaskStateSegment::new());

#[repr(align(128))]
pub struct Tss(TaskStateSegment);

impl Tss {
    #[allow(dead_code)]
    pub const fn into(self) -> TaskStateSegment {
        self.0
    }

    pub const fn from(x: TaskStateSegment) -> Self {
        Tss(x)
    }
}

pub fn init() {
    #[cfg(target_arch = "x86_64")]
    let limit = 0;
    #[cfg(target_arch = "x86")]
    let limit = 0xFFFF_FFFF;

    unsafe {
        GLOBAL_DESCRIPTOR_TABLE_ARRAY[NULL] = Descriptor::NULL;

        #[cfg(target_arch = "x86_64")]
        {
            GLOBAL_DESCRIPTOR_TABLE_ARRAY[KERNEL_CODE] =
                DescriptorBuilder::code_descriptor(0, limit, CodeSegmentType::ExecuteRead)
                    .present()
                    .dpl(Ring::Ring0)
                    .l()
                    .finish();
        }
        #[cfg(target_arch = "x86")]
        {
            GLOBAL_DESCRIPTOR_TABLE_ARRAY[KERNEL_CODE] =
                DescriptorBuilder::code_descriptor(0, limit, CodeSegmentType::ExecuteRead)
                    .present()
                    .dpl(Ring::Ring0)
                    .db()
                    .limit_granularity_4kb()
                    .finish();
        }

        GLOBAL_DESCRIPTOR_TABLE_ARRAY[KERNEL_DATA] = DescriptorBuilder::data_descriptor(0, 0, DataSegmentType::ReadWrite)
            .present()
            .dpl(Ring::Ring0)
            .finish();

        GLOBAL_DESCRIPTOR_TABLE_ARRAY[USER32_CODE] =
            DescriptorBuilder::code_descriptor(0, 0, CodeSegmentType::ExecuteRead)
                .present()
                .dpl(Ring::Ring3)
                .finish();

        GLOBAL_DESCRIPTOR_TABLE_ARRAY[USER32_DATA] = DescriptorBuilder::data_descriptor(0, 0, DataSegmentType::ReadWrite)
            .present()
            .dpl(Ring::Ring3)
            .finish();

        #[cfg(target_arch = "x86_64")]
        {
            GLOBAL_DESCRIPTOR_TABLE_ARRAY[USER64_CODE] =
                DescriptorBuilder::code_descriptor(0, 0, CodeSegmentType::ExecuteRead)
                    .present()
                    .dpl(Ring::Ring3)
                    .l()
                    .finish();
        }

        #[cfg(target_arch = "x86_64")]
        {
            let base = &TASK_STATE_SEGMENT_WRAPPER.0 as *const _ as u64;
            let tss_descriptor: Descriptor64 =
                <DescriptorBuilder as GateDescriptorBuilder<u64>>::tss_descriptor(
                    base,
                    base + mem::size_of::<TaskStateSegment>() as u64 - 1,
                    true,
                )
                    .present()
                    .dpl(Ring::Ring0)
                    .finish();

            GLOBAL_DESCRIPTOR_TABLE_ARRAY[FIRST_TSS..FIRST_TSS + TSS_ENTRIES]
                .copy_from_slice(&mem::transmute::<Descriptor64, [Descriptor; 2]>(
                    tss_descriptor,
                ));

            TASK_STATE_SEGMENT_WRAPPER.0.rsp[0] = get_boot_stack().interrupt_top().into();
        }
        #[cfg(target_arch = "x86")]
        {
            let base = &TASK_STATE_SEGMENT_WRAPPER.0 as *const _ as u64;
            let tss_descriptor: Descriptor =
                <DescriptorBuilder as GateDescriptorBuilder<u32>>::tss_descriptor(
                    base,
                    base + mem::size_of::<TaskStateSegment>() as u64 - 1,
                    true,
                )
                    .present()
                    .dpl(Ring::Ring0)
                    .finish();

            TASK_STATE_SEGMENT_WRAPPER.0.eflags = 0x1202;
            TASK_STATE_SEGMENT_WRAPPER.0.ss0 = 0x10;
            TASK_STATE_SEGMENT_WRAPPER.0.esp0 = get_boot_stack().interrupt_top().into();
            TASK_STATE_SEGMENT_WRAPPER.0.cs = 0x0b;

            GLOBAL_DESCRIPTOR_TABLE_ARRAY[FIRST_TSS] = tss_descriptor;
        }

        let gdtr = DescriptorTablePointer::new(&GLOBAL_DESCRIPTOR_TABLE_ARRAY);
        lgdt(&gdtr);

        load_cs(SegmentSelector::new(KERNEL_CODE as u16, Ring::Ring0));
        load_ss(SegmentSelector::new(KERNEL_DATA as u16, Ring::Ring0));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn set_kernel_stack(stack: VirtAddr) {
    TASK_STATE_SEGMENT_WRAPPER.0.rsp[0] = stack.as_u64();
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn set_kernel_stack(stack: VirtAddr) {
    TASK_STATE_SEGMENT_WRAPPER.0.esp = stack.as_u32();
}

pub unsafe extern "C" fn set_current_kernel_stack() {
    cr3_write(scheduler::get_root_page_table().as_u64());
    set_kernel_stack(scheduler::get_current_interrupt_stack());
}