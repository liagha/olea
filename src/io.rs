use alloc::string::String;
use alloc::vec::Vec;
use core::{fmt, result};
use num_derive::{FromPrimitive, ToPrimitive};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum Error {
	FileNotFound = crate::errno::FILE_NOT_FOUND as isize,
	NotImplemented = crate::errno::NOT_IMPLEMENTED as isize,
	IoError = crate::errno::IO_ERROR as isize,
	BadFileDescriptor = crate::errno::BAD_FILE_DESCRIPTOR as isize,
	IsADirectory = crate::errno::IS_A_DIRECTORY as isize,
	InvalidArgument = crate::errno::INVALID_ARGUMENT as isize,
	TimerExpired = crate::errno::TIMER_EXPIRED as isize,
	TryAgain = crate::errno::TRY_AGAIN as isize,
	BadAddress = crate::errno::BAD_ADDRESS as isize,
	NoBufferSpace = crate::errno::NO_BUFFER_SPACE as isize,
	NotConnected = crate::errno::NOT_CONNECTED as isize,
	NotADirectory = crate::errno::NOT_A_DIRECTORY as isize,
	TooManyOpenFiles = crate::errno::TOO_MANY_OPEN_FILES as isize,
	FileExists = crate::errno::FILE_EXISTS as isize,
	AddressInUse = crate::errno::ADDRESS_IN_USE as isize,
	ValueOverflow = crate::errno::VALUE_OVERFLOW as isize,
	NotASocket = crate::errno::NOT_A_SOCKET as isize,
}

pub type Result<T> = result::Result<T, Error>;

/// The Read trait allows for reading bytes from a source.
///
/// The Read trait is derived from Rust's std library.
pub trait Read {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

	/// Read all bytes until EOF in this source, placing them into buf.
	fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
		let start_len = buf.len();

		loop {
			let mut probe = [0u8; 512];

			match self.read(&mut probe) {
				Ok(0) => return Ok(buf.len() - start_len),
				Ok(n) => {
					buf.extend_from_slice(&probe[..n]);
				}
				Err(e) => return Err(e),
			}
		}
	}

	/// Read all bytes until EOF in this source, appending them to `buf`.
	///
	/// If successful, this function returns the number of bytes which were read
	/// and appended to `buf`.
	fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
		unsafe { self.read_to_end(buf.as_mut_vec()) }
	}
}

/// The Write trait allows for reading bytes from a source.
///
/// The Write trait is derived from Rust's std library.
pub trait Write {
	fn write(&mut self, buf: &[u8]) -> Result<usize>;

	/// Attempts to write an entire buffer into this writer.
	fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
		while !buf.is_empty() {
			match self.write(buf) {
				Ok(0) => {
					return Err(Error::IoError);
				}
				Ok(n) => buf = &buf[n..],
				Err(e) => return Err(e),
			}
		}

		Ok(())
	}

	/// Writes a formatted string into this writer, returning any error encountered.
	fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<()> {
		// Create a shim which translates a Write to a fmt::Write and saves
		// off I/O errors. instead of discarding them
		struct Adapter<'a, T: ?Sized> {
			inner: &'a mut T,
			error: Result<()>,
		}

		impl<T: Write + ?Sized> fmt::Write for Adapter<'_, T> {
			fn write_str(&mut self, s: &str) -> fmt::Result {
				match self.inner.write_all(s.as_bytes()) {
					Ok(()) => Ok(()),
					Err(e) => {
						self.error = Err(e);
						Err(fmt::Error)
					}
				}
			}
		}

		let mut output = Adapter {
			inner: self,
			error: Ok(()),
		};
		match fmt::write(&mut output, fmt) {
			Ok(()) => Ok(()),
			Err(..) => {
				// check if the error came from the underlying `Write` or not
				if output.error.is_err() {
					output.error
				} else {
					Err(Error::InvalidArgument)
				}
			}
		}
	}
}
