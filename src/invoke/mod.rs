mod exit;
mod invalid;
mod nothing;
mod write;

use {
	exit::exit,
	invalid::invalid,
	nothing::nothing,
	write::{write, write_vector},
};

pub mod numbers {
	// These match the Linux x86-64 invoke numbers for compatibility

	/// System invoke number for write() - output data to file descriptor
	pub const WRITE: usize = 1;

	/// System invoke number for close() - close file descriptor
	pub const CLOSE: usize = 3;

	/// System invoke number for ioctl() - device-specific input/output control
	pub const IO_CONTROL: usize = 16;

	/// System invoke number for writev() - write data from multiple buffers
	pub const WRITE_VECTOR: usize = 20;

	/// System invoke number for exit() - terminate calling process
	pub const EXIT: usize = 60;

	/// System invoke number for arch_prctl() - set architecture-specific thread state
	pub const ARCH_PROCESS_CONTROL: usize = 158;

	/// System invoke number for set_tid_address() - set pointer to thread ID
	pub const SET_THREAD_ID_ADDRESS: usize = 218;

	/// System invoke number for exit_group() - exit all threads in a process
	pub const EXIT_GROUP: usize = 231;

	/// Total number of possible system invoke in the table
	pub const MAX_INVOKES: usize = 400;
}

#[repr(align(64))]
#[repr(C)]       
pub struct InvokeTable {
	handle: [*const usize; numbers::MAX_INVOKES],
}

impl InvokeTable {
	pub const fn default() -> Self {
		let mut table = InvokeTable {
			handle: [invalid as *const _; numbers::MAX_INVOKES],
		};

		table.handle[numbers::WRITE] = write as *const _;             
		table.handle[numbers::WRITE_VECTOR] = write_vector as *const _;  

		table.handle[numbers::CLOSE] = nothing as *const _;     
		table.handle[numbers::IO_CONTROL] = nothing as *const _;  

		table.handle[numbers::EXIT] = exit as *const _;  
		table.handle[numbers::EXIT_GROUP] = exit as *const _; 

		table.handle[numbers::ARCH_PROCESS_CONTROL] = nothing as *const _;  
		table.handle[numbers::SET_THREAD_ID_ADDRESS] = nothing as *const _;   

		table
	}
}

unsafe impl Send for InvokeTable {}

unsafe impl Sync for InvokeTable {}

impl Default for InvokeTable {
	fn default() -> Self {
		Self::default()
	}
}

pub static INVOKE_TABLE: InvokeTable = InvokeTable::default();