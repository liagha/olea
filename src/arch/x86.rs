pub use {
    x86::{
        Ring,
        controlregs::cr3_write,
        dtables::{self, DescriptorTablePointer},
        segmentation::*,
        bits64::{
            segmentation::*,
            task::*,
        },
    }
};