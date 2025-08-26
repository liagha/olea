use {
	super::{
		error::Error, types::Metadata
	},
	crate::{
		format::Debug,
		scheduler::get_io_interface,
	}
};

pub type Descriptor = i32;

pub const STANDARD_INPUT: Descriptor = 0;
pub const STANDARD_OUTPUT: Descriptor = 1;
pub const STANDARD_ERROR: Descriptor = 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SeekFrom {
	Start(usize),
	End(isize),
	Current(isize),
}

pub struct State {
	pub size: usize,
}

pub trait Interface: Sync + Send + Debug {
	fn read(&self, _buf: &mut [u8]) -> Result<usize, Error> {
		Err(Error::NotImplemented)
	}
	fn write(&self, _buf: &[u8]) -> Result<usize, Error> {
		Err(Error::NotImplemented)
	}
	fn seek(&self, _offset: SeekFrom) -> Result<usize, Error> {
		Err(Error::NotImplemented)
	}
	fn fstat(&self) -> Result<State, Error> {
		Err(Error::NotImplemented)
	}
	fn metadata(&self) -> Result<Metadata, Error> {
		Err(Error::NotImplemented)
	}
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct OpenOptions: i32 {
        const READ_ONLY = 0o0000;
        const WRITE_ONLY = 0o0001;
        const READ_WRITE = 0o0002;
        const CREATE = 0o0100;
        const EXCLUSIVE = 0o0200;
        const TRUNCATE = 0o1000;
        const APPEND = 0o2000;
        const NONBLOCK = 0o4000;
        const DIRECT_IO = 0o40000;
        const DIRECTORY = 0o200_000;
    }
}

pub fn read(descriptor: Descriptor, buffer: &mut [u8]) -> Result<usize, Error> {
	let object = get_io_interface(descriptor).map_err(|_| Error::IoError)?;
	if buffer.is_empty() {
		return Ok(0);
	}
	object.read(buffer)
}

pub fn write(descriptor: Descriptor, buffer: &[u8]) -> Result<usize, Error> {
	let object = get_io_interface(descriptor).map_err(|_| Error::IoError)?;
	if buffer.is_empty() {
		return Ok(0);
	}
	object.write(buffer)
}

pub fn fstat(descriptor: Descriptor) -> Result<State, Error> {
	get_io_interface(descriptor).map_err(|_| Error::IoError)?.fstat()
}

pub fn metadata(descriptor: Descriptor) -> Result<Metadata, Error> {
	get_io_interface(descriptor).map_err(|_| Error::IoError)?.metadata()
}