use crate::file::descriptor::FileDescriptor;
use crate::logging::*;

/// I/O Vector structure for vectored I/O operations
/// Represents a single buffer in a scatter-gather I/O operation
#[repr(C)]  // Use C layout for compatibility with userspace
pub struct IoVec {
	pub iov_base: *const u8,  // Pointer to buffer
	pub iov_len: usize,       // Length of buffer
}

/// Handler for writev() system call - write data from multiple buffers
/// Implements scatter-gather write operation
///
/// Arguments:
/// - fd: File descriptor to write to
/// - ptr: Pointer to array of IoVec structures
/// - cnt: Number of IoVec structures in array
///
/// Returns: Number of bytes written, or negative error code
pub(crate) unsafe extern "C" fn write_vector(
	fd: FileDescriptor,
	ptr: *const IoVec,
	cnt: i32,
) -> isize {
	debug!("enter call writev.");
	let mut len: isize = 0;

	// Convert raw pointer and count to safe Rust slice
	let iovec = core::slice::from_raw_parts(ptr, cnt as usize);

	// Process each I/O vector in the array
	for i in iovec {
		// Convert each buffer to a Rust slice
		let slice = core::slice::from_raw_parts(i.iov_base, i.iov_len);

		// Attempt to write this buffer
		let tmp: isize = crate::file::descriptor::write(fd, slice).map_or_else(
			// On error: return negative error code
			|e| -num::ToPrimitive::to_isize(&e).unwrap(),
			// On success: return number of bytes written
			|v| v.try_into().unwrap(),
		);

		len += tmp;

		// If we wrote fewer bytes than requested, stop processing
		// This indicates the output buffer is full or an error occurred
		if tmp < i.iov_len as isize {
			break;
		}
	}

	len  // Return total bytes written
}

/// Handler for write() system call - write data from single buffer
///
/// Arguments:
/// - fd: File descriptor to write to
/// - buf: Pointer to data buffer
/// - len: Number of bytes to write
///
/// Returns: Number of bytes written, or negative error code
pub(crate) unsafe extern "C" fn write(fd: FileDescriptor, buf: *mut u8, len: usize) -> isize {
	debug!("enter call write.");

	// Convert raw pointer and length to safe Rust slice
	let slice = unsafe { core::slice::from_raw_parts(buf, len) };

	// Call the file descriptor write function
	crate::file::descriptor::write(fd, slice).map_or_else(
		// On error: return negative error code
		|e| -num::ToPrimitive::to_isize(&e).unwrap(),
		// On success: return number of bytes written
		|v| v.try_into().unwrap(),
	)
}