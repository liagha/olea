use crate::scheduler;

pub extern "C" fn exit() {
	debug!("enter invoke exit.");
	
	scheduler::exit();
}