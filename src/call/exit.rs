use crate::logging::*;
use crate::scheduler::*;

/// Handler for exit() and exit_group() system calls
/// Terminates the current process/thread
pub(crate) extern "C" fn exit() {
	debug!("enter call exit.");
	// Call scheduler function to terminate current task
	do_exit();
}