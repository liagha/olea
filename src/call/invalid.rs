use crate::logging::*;
use crate::scheduler::*;
use core::arch::asm;

/// Helper function called by sys_invalid to handle unknown system calls
/// Takes call number as parameter and terminates the process
extern "C" fn invalid_syscall(sys_no: u64) -> ! {
	error!("invalid call {}.", sys_no);
	// Terminate process that made invalid call
	do_exit();
}

/// Handler for invalid/unimplemented system calls
/// This is the default handler in the call table
/// Extracts call number from rax and calls error handler
#[allow(unused_assignments)]
#[naked]
pub(crate) unsafe extern "C" fn invalid() {
	asm!(
	"mov rdi, rax",    // Move call number from rax to rdi (1st argument)
	"call {}",         // Call the invalid_syscall function
	sym invalid_syscall,  // Reference to the function symbol
	options(noreturn)     // This never returns
	);
}