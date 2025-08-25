use crate::scheduler::*;

/// Handler for exit() and exit_group() system invoke
/// Terminates the current process/thread
pub extern "C" fn exit() {
	debug!("enter invoke exit.");
	// Call scheduler function to terminate current task
	do_exit();
}