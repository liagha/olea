// src/arch/x86/kernel/processor/shutdown.rs
#[cfg(feature = "qemu-exit")]
use qemu_exit::QEMUExit;

#[allow(unused_variables)]
#[no_mangle]
pub(crate) extern "C" fn shutdown(error_code: i32) -> ! {
    #[cfg(feature = "qemu-exit")]
    {
        let code = if error_code == 0 { 5 } else { 1 };

        let qemu_exit_handle = qemu_exit::X86::new(0xf4, code);
        qemu_exit_handle.exit_success();
    }

    #[cfg(not(feature = "qemu-exit"))]
    loop {
        unsafe {
            x86::halt();
        }
    }
}