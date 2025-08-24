use {
	core::result,
	crate::{
		format::{self, Arguments},
		error::numbers,
	},
	alloc::{
		string::String,
		vec::Vec,
	},
};

#[derive(Debug, PartialEq)]
pub enum Error {
	FileNotFound = numbers::FILE_NOT_FOUND as isize,
	NotImplemented = numbers::NOT_IMPLEMENTED as isize,
	IoError = numbers::IO_ERROR as isize,
	BadFileDescriptor = numbers::BAD_FILE_DESCRIPTOR as isize,
	IsADirectory = numbers::IS_A_DIRECTORY as isize,
	InvalidArgument = numbers::INVALID_ARGUMENT as isize,
	TimerExpired = numbers::TIMER_EXPIRED as isize,
	TryAgain = numbers::TRY_AGAIN as isize,
	BadAddress = numbers::BAD_ADDRESS as isize,
	NoBufferSpace = numbers::NO_BUFFER_SPACE as isize,
	NotConnected = numbers::NOT_CONNECTED as isize,
	NotADirectory = numbers::NOT_A_DIRECTORY as isize,
	TooManyOpenFiles = numbers::TOO_MANY_OPEN_FILES as isize,
	FileExists = numbers::FILE_EXISTS as isize,
	AddressInUse = numbers::ADDRESS_IN_USE as isize,
	ValueOverflow = numbers::VALUE_OVERFLOW as isize,
	NotASocket = numbers::NOT_A_SOCKET as isize,
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
	fn write_fmt(&mut self, fmt: Arguments<'_>) -> Result<()> {
		// Create a shim which translates a Write to a fmt::Write and saves
		// off I/O errors. instead of discarding them
		struct Adapter<'a, T: ?Sized> {
			inner: &'a mut T,
			error: Result<()>,
		}

		impl<T: Write + ?Sized> format::Write for Adapter<'_, T> {
			fn write_str(&mut self, s: &str) -> format::Result {
				match self.inner.write_all(s.as_bytes()) {
					Ok(()) => Ok(()),
					Err(e) => {
						self.error = Err(e);
						Err(format::Error)
					}
				}
			}
		}

		let mut output = Adapter {
			inner: self,
			error: Ok(()),
		};
		match format::write(&mut output, fmt) {
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
