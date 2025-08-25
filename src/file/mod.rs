#![allow(dead_code)]

mod initial;
mod r#virtual;
pub mod descriptor;
pub mod standard;

use {
	crate::{
		format::Debug,
		file::r#virtual::Fs,
		io::{Write, Read, Error},
		scheduler::{insert_io_interface, remove_io_interface},
	},
	alloc::{
		string::{String, ToString},
		sync::Arc,
		vec::Vec
	},
	core::include_bytes,
	descriptor::{FileDescriptor, IoInterface, OpenOptions, SeekFrom},
};

static DEMO: &[u8] = include_bytes!("../../demo/hello");

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
	File,
	Directory,
}

trait VfsNode: Debug + Send + Sync {
	fn get_kind(&self) -> NodeKind;
}

trait VfsNodeFile: VfsNode + Debug + Send + Sync {
	fn get_handle(&self, _opt: OpenOptions) -> Result<Arc<dyn IoInterface>, Error>;
}

trait VfsNodeDirectory: VfsNode + Debug + Send + Sync {
	fn traverse_mkdir(&mut self, _components: &mut Vec<&str>) -> Result<(), Error>;

	fn traverse_lsdir(&self, _tabs: String) -> Result<(), Error>;

	fn traverse_open(
		&mut self,
		_components: &mut Vec<&str>,
		_flags: OpenOptions,
	) -> Result<Arc<dyn IoInterface>, Error>;

	fn traverse_mount(&mut self, _components: &mut Vec<&str>, slice: &'static [u8]) -> Result<(), Error>;
}

trait Vfs: Debug + Send + Sync {
	fn mkdir(&mut self, path: &String) -> Result<(), Error>;

	fn lsdir(&self) -> Result<(), Error>;

	fn open(&mut self, path: &str, flags: OpenOptions) -> Result<Arc<dyn IoInterface>, Error>;

	fn mount(&mut self, path: &String, slice: &'static [u8]) -> Result<(), Error>;
}

static mut VFS_ROOT: Option<Fs> = None;

pub fn lsdir() -> Result<(), Error> {
	unsafe { VFS_ROOT.as_mut().unwrap().lsdir() }
}

pub fn mkdir(path: &String) -> Result<(), Error> {
	unsafe { VFS_ROOT.as_mut().unwrap().mkdir(path) }
}

pub fn open(name: &str, flags: OpenOptions) -> Result<FileDescriptor, Error> {
	debug!("open {}, {:?}.", name, flags);

	let fs = unsafe { VFS_ROOT.as_mut().unwrap() };
	if let Ok(file) = fs.open(name, flags) {
		let fd = insert_io_interface(file)?;
		Ok(fd)
	} else {
		Err(Error::InvalidArgument)
	}
}

pub fn mount(path: &String, slice: &'static [u8]) -> Result<(), Error> {
	unsafe { VFS_ROOT.as_mut().unwrap().mount(path, slice) }
}

fn check_path(path: &str) -> bool {
	if let Some(pos) = path.find('/') {
		if pos == 0 {
			return true;
		}
	}

	false
}

#[derive(Debug)]
pub struct File {
	fd: FileDescriptor,
	path: String,
}

impl File {
	pub fn create(path: &str) -> Result<Self, Error> {
		let fd = open(path, OpenOptions::READ_WRITE | OpenOptions::CREATE)?;

		Ok(File {
			fd,
			path: path.to_string(),
		})
	}

	pub fn open(path: &str) -> Result<Self, Error> {
		let fd = open(path, OpenOptions::READ_WRITE)?;

		Ok(File {
			fd,
			path: path.to_string(),
		})
	}

	pub fn len(&self) -> Result<usize, Error> {
		let fstat = descriptor::fstat(self.fd)?;
		Ok(fstat.file_size)
	}
}

impl Read for File {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
		descriptor::read(self.fd, buf)
	}
}

impl Write for File {
	fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
		descriptor::write(self.fd, buf)
	}
}

impl Drop for File {
	fn drop(&mut self) {
		let _ = remove_io_interface(self.fd);
	}
}

pub fn init() {
	let mut root = Fs::new();

	root.mkdir(&String::from("/bin")).unwrap();
	root.mkdir(&String::from("/dev")).unwrap();

	if DEMO.len() > 0 {
		info!(
			"found mountable file at 0x{:x} (len 0x{:x}).",
			DEMO.as_ptr() as u64,
			DEMO.len()
		);
		root.mount(&String::from("/bin/demo"), &DEMO)
			.expect("Unable to mount file");
	}

	root.lsdir().unwrap();
	
	unsafe {
		VFS_ROOT = Some(root);
	}
}
