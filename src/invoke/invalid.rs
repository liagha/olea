use {
	crate::{
		arch::asm,
		scheduler::*,
	}
};

/// Helper function called by sys_invalid to handle unknown system invoke
/// Takes invoke number as parameter and terminates the process
extern "C" fn invalid_syscall(sys_no: u64) -> ! {
	error!("invalid invoke {}.", sys_no);
	
	exit();
}

/// Handler for invalid/unimplemented system invoke
/// This is the default handler in the invoke table
/// Extracts invoke number from rax and invoke error handler
#[allow(unused_assignments)]
#[naked]
pub unsafe extern "C" fn invalid() {
	asm!(
	"mov rdi, rax",    // Move invoke number from rax to rdi (1st argument)
	"call {}",         // Call the invalid_syscall function
	sym invalid_syscall,  // Reference to the function symbol
	options(noreturn)     // This never returns
	);
}