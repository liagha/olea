/// Handler for system calls that should do nothing but succeed
/// Used for syscalls that are not implemented but should not cause errors
/// Examples: close(), ioctl(), arch_prctl(), set_tid_address()
pub(crate) extern "C" fn nothing() -> i32 {
	// Return 0 (success) without doing anything
	0
}