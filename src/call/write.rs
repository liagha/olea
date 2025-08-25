use crate::file::descriptor::Descriptor;

/// I/O Vector structure for vectored I/O operations
/// Represents a single buffer in a scatter-gather I/O operation
#[repr(C)]  // Use C layout for compatibility with userspace
pub struct BufferSegment {
	pub base: *const u8,  // Pointer to buffer
	pub length: usize,       // Length of buffer
}

/// Handler for writev() system call - write data from multiple buffers
/// Implements scatter-gather write operation
///
/// Arguments:
/// - descriptor: File descriptor to write to
/// - pointer: Pointer to array of BufferSegment structures
/// - count: Number of BufferSegment structures in array
///
/// Returns: Number of bytes written, or negative error code
pub unsafe extern "C" fn write_vector(
    descriptor: Descriptor,
    pointer: *const BufferSegment,
    count: i32,
) -> isize {
	debug!("enter call writev.");
	let mut len: isize = 0;

	// Convert raw pointer and count to safe Rust slice
	let iovec = core::slice::from_raw_parts(pointer, count as usize);

	// Process each I/O vector in the array
	for i in iovec {
		// Convert each buffer to a Rust slice
		let slice = core::slice::from_raw_parts(i.base, i.length);

		// Attempt to write this buffer
		let tmp: isize = crate::file::descriptor::write(descriptor, slice).map_or_else(
			// On error: return negative error code
			|e| -(e as isize),
			// On success: return number of bytes written
			|v| v.try_into().unwrap(),
		);

		len += tmp;

		// If we wrote fewer bytes than requested, stop processing
		// This indicates the output buffer is full or an error occurred
		if tmp < i.length as isize {
			break;
		}
	}

	len  // Return total bytes written
}

/// Handler for write() system call - write data from single buffer
///
/// Arguments:
/// - fd: File descriptor to write to
/// - buffer: Pointer to data buffer
/// - length: Number of bytes to write
///
/// Returns: Number of bytes written, or negative error code
pub unsafe extern "C" fn write(descriptor: Descriptor, buffer: *mut u8, length: usize) -> isize {
	debug!("enter call write.");

	// Convert raw pointer and length to safe Rust slice
	let slice = unsafe { core::slice::from_raw_parts(buffer, length) };

	// Call the file descriptor write function
	crate::file::descriptor::write(descriptor, slice).map_or_else(
		|e| -(e as isize),
		|v| v.try_into().unwrap(),
	)
}