pub mod stdio;

use {
	crate::{
		io,
		format,
		scheduler::get_io_interface,
	},
};

pub type FileDescriptor = i32;

pub const STDIN_FILENO: FileDescriptor = 0;
pub const STDOUT_FILENO: FileDescriptor = 1;
pub const STDERR_FILENO: FileDescriptor = 2;

/// Enumeration of possible methods to seek within an I/O object.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SeekFrom {
	/// Set the offset to the provided number of bytes.
	Start(usize),
	/// Set the offset to the size of this object plus the specified number of bytes.
	///
	/// It is possible to seek beyond the end of an object, but it's an error to
	/// seek before byte 0.
	End(isize),
	/// Set the offset to the current position plus the specified number of bytes.
	///
	/// It is possible to seek beyond the end of an object, but it's an error to
	/// seek before byte 0.
	Current(isize),
}

/// Describes information about a file.
pub struct FileStatus {
	/// Size of the file
	pub file_size: usize,
}

#[allow(dead_code)]
pub trait IoInterface: Sync + Send + format::Debug {
	/// `read` attempts to read `len` bytes from the object references
	/// by the descriptor
	fn read(&self, _buf: &mut [u8]) -> io::Result<usize> {
		Err(io::Error::NotImplemented)
	}

	/// `write` attempts to write `len` bytes to the object references
	/// by the descriptor
	fn write(&self, _buf: &[u8]) -> io::Result<usize> {
		Err(io::Error::NotImplemented)
	}

	fn seek(&self, _offset: SeekFrom) -> io::Result<usize> {
		Err(io::Error::NotImplemented)
	}

	fn fstat(&self) -> io::Result<FileStatus> {
		Err(io::Error::NotImplemented)
	}
}

bitflags! {
   /// Options for opening files
   #[derive(Debug, Copy, Clone)]
   pub struct OpenOptions: i32 {
       /// Open file for reading only
       const READ_ONLY = 0o0000;
       
       /// Open file for writing only
       const WRITE_ONLY = 0o0001;
       
       /// Open file for both reading and writing
       const READ_WRITE = 0o0002;
       
       /// Create the file if it doesn't exist
       const CREATE = 0o0100;
       
       /// Fail if file already exists (use with CREATE)
       const EXCLUSIVE = 0o0200;
       
       /// Truncate file to zero length when opening
       const TRUNCATE = 0o1000;
       
       /// All writes append to end of file
       const APPEND = 0o2000;
       
       /// Bypass kernel buffer cache for direct I/O
       const DIRECT_IO = 0o40000;
       
       /// Fail if path is not a directory
       const DIRECTORY = 0o200_000;
   }
}

pub fn read(descriptor: FileDescriptor, buffer: &mut [u8]) -> io::Result<usize> {
	let object = get_io_interface(descriptor)?;

	if buffer.is_empty() {
		return Ok(0);
	}

	object.read(buffer)
}

pub fn write(descriptor: FileDescriptor, buffer: &[u8]) -> io::Result<usize> {
	let object = get_io_interface(descriptor)?;

	if buffer.is_empty() {
		return Ok(0);
	}

	object.write(buffer)
}

pub fn fstat(descriptor: FileDescriptor) -> io::Result<FileStatus> {
	get_io_interface(descriptor)?.fstat()
}
