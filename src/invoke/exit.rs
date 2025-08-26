use crate::scheduler;

/// Handler for exit() and exit_group() system invoke
/// Terminates the current process/thread
pub extern "C" fn exit() {
	debug!("enter invoke exit.");
	
	scheduler::exit();
}