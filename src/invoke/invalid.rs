use {
	crate::{
		arch::asm,
		scheduler::*,
	}
};

extern "C" fn invalid_invoke(sys_no: u64) -> ! {
	error!("invalid invoke {}.", sys_no);
	
	exit();
}

#[allow(unused_assignments)]
#[naked]
pub unsafe extern "C" fn invalid() {
	asm!(
	"mov rdi, rax",    // Move invoke number from rax to rdi (1st argument)
	"call {}",         // Call the invalid_syscall function
	sym invalid_invoke,  // Reference to the function symbol
	options(noreturn)     // This never returns
	);
}