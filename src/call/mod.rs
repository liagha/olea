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
	// These match the Linux x86-64 call numbers for compatibility

	/// System call number for write() - output data to file descriptor
	pub const WRITE: usize = 1;

	/// System call number for close() - close file descriptor
	pub const CLOSE: usize = 3;

	/// System call number for ioctl() - device-specific input/output control
	pub const IO_CONTROL: usize = 16;

	/// System call number for writev() - write data from multiple buffers
	pub const WRITE_VECTOR: usize = 20;

	/// System call number for exit() - terminate calling process
	pub const EXIT: usize = 60;

	/// System call number for arch_prctl() - set architecture-specific thread state
	pub const ARCH_PROCESS_CONTROL: usize = 158;

	/// System call number for set_tid_address() - set pointer to thread ID
	pub const SET_THREAD_ID_ADDRESS: usize = 218;

	/// System call number for exit_group() - exit all threads in a process
	pub const EXIT_GROUP: usize = 231;

	/// Total number of possible system calls in the table
	pub const MAX_CALLS: usize = 400;
}

/// System call dispatch table
/// Maps system call numbers to their handler functions
/// Aligned to 64-byte boundary for optimal cache performance
#[repr(align(64))]  // Cache line alignment for performance
#[repr(C)]          // C layout to ensure predictable memory layout
pub(crate) struct CallTable {
	/// Array of function pointers, indexed by system call number
	/// Each entry points to a system call handler function
	handle: [*const usize; numbers::MAX_CALLS],
}

impl CallTable {
	/// Create a new system call table with all entries initialized
	/// This must be const fn to allow static initialization
	pub const fn default() -> Self {
		// Initialize all entries to point to invalid call handler
		// This ensures that unimplemented syscalls are caught
		let mut table = CallTable {
			handle: [invalid as *const _; numbers::MAX_CALLS],
		};

		// Implemented Calls
		// Map specific call numbers to their handler functions

		// I/O operations
		table.handle[numbers::WRITE] = write as *const _;                  // Write to file descriptor
		table.handle[numbers::WRITE_VECTOR] = write_vector as *const _;    // Vectored write

		// File operations (stubbed out)
		table.handle[numbers::CLOSE] = nothing as *const _;         // Close file (no-op)
		table.handle[numbers::IO_CONTROL] = nothing as *const _;    // I/O control (no-op)

		// Process control
		table.handle[numbers::EXIT] = exit as *const _;        // Exit process
		table.handle[numbers::EXIT_GROUP] = exit as *const _;  // Exit all threads

		// Thread/architecture control (stubbed out)
		table.handle[numbers::ARCH_PROCESS_CONTROL] = nothing as *const _;       // Arch control (no-op)
		table.handle[numbers::SET_THREAD_ID_ADDRESS] = nothing as *const _;      // Set TID address (no-op)

		table
	}
}

// Safety Implementations
// These are required because the table contains raw function pointers
// We assert that the table can be safely shared between threads

/// Safe to send between threads because function pointers are immutable
unsafe impl Send for CallTable {}

/// Safe to share references between threads because table is read-only after init
unsafe impl Sync for CallTable {}

impl Default for CallTable {
	fn default() -> Self {
		Self::default()
	}
}

/// Global system call handler table
/// This is accessed by the assembly code in the call entry point
/// Static initialization ensures it's available at boot time
pub(crate) static CALL_TABLE: CallTable = CallTable::default();