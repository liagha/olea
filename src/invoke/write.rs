use {
	crate::{
		file::{
			vfs::{
				descriptor::{
					self, Descriptor
				},
			},
		},
	},
};

/// I/O Vector structure for vectored I/O operations
/// Represents a single buffer in a scatter-gather I/O operation
#[repr(C)]  
pub struct BufferSegment {
	pub base: *const u8,  
	pub length: usize,  
}

/// Handler for writev() system invoke - write data from multiple buffers
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
	debug!("enter invoke writev.");
	let mut length: isize = 0;

	let segment = core::slice::from_raw_parts(pointer, count as usize);

	for buffer in segment {
		let slice = core::slice::from_raw_parts(buffer.base, buffer.length);

		let temporary: isize = descriptor::write(descriptor, slice).map_or_else(
			|error| -(error as isize),
			|vector| vector.try_into().unwrap(),
		);

		length += temporary;

		if temporary < buffer.length as isize {
			break;
		}
	}

	length  
}

/// Handler for write() system invoke - write data from single buffer
///
/// Arguments:
/// - fd: File descriptor to write to
/// - buffer: Pointer to data buffer
/// - length: Number of bytes to write
///
/// Returns: Number of bytes written, or negative error code
pub unsafe extern "C" fn write(descriptor: Descriptor, buffer: *mut u8, length: usize) -> isize {
	debug!("enter invoke write.");

	let slice = unsafe { core::slice::from_raw_parts(buffer, length) };

	descriptor::write(descriptor, slice).map_or_else(
		|error| -(error as isize),
		|vector| vector.try_into().unwrap(),
	)
}