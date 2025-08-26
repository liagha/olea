#![allow(dead_code)]

pub mod system;
pub mod handle;
pub mod standard;
pub mod descriptor;
pub mod error;
pub mod types;

pub use error::Error;

use {
	super::{
		vfs::system::VirtualSystem,
	},
	crate::{
		format::Debug,
		io::{self, Write, Read},
		scheduler::{insert_io_interface, remove_io_interface},
	},
	alloc::{
		format,
		vec::Vec,
		string::{String, ToString},
	},
};

static DEMO: &[u8] = include_bytes!("../../../demo/hello_kernel");

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
	File,
	Directory,
	Symlink,
}

pub fn list() -> Result<(), Error> {
	unsafe { system::ROOT.as_mut().unwrap().list() }
}

pub fn make(path: &String) -> Result<(), Error> {
	let path = normalize_path(path)?;
	unsafe { system::ROOT.as_mut().unwrap().make(&path) }
}

pub fn open(name: &str, flags: descriptor::OpenOptions) -> Result<descriptor::Descriptor, Error> {
	debug!("open {}, {:?}.", name, flags);
	let name = normalize_path(&name.to_string())?;
	let fs = unsafe { system::ROOT.as_mut().unwrap() };
	if let Ok(file) = fs.open(&name, flags) {
		let fd = insert_io_interface(file).map_err(|_| Error::IoError)?;
		Ok(fd)
	} else {
		Err(Error::InvalidArgument)
	}
}

pub fn mount(path: &String, slice: &'static [u8]) -> Result<(), Error> {
	let path = normalize_path(path)?;
	unsafe { system::ROOT.as_mut().unwrap().mount(&path, slice) }
}

pub fn symlink(target: &String, link: &String) -> Result<(), Error> {
	let target = normalize_path(target)?;
	let link = normalize_path(link)?;
	unsafe { system::ROOT.as_mut().unwrap().symlink(&target, &link) }
}

pub fn unlink(path: &String) -> Result<(), Error> {
	let path = normalize_path(path)?;
	unsafe { system::ROOT.as_mut().unwrap().unlink(&path) }
}

pub fn rename(old_path: &String, new_path: &String) -> Result<(), Error> {
	let old_path = normalize_path(old_path)?;
	let new_path = normalize_path(new_path)?;
	unsafe { system::ROOT.as_mut().unwrap().rename(&old_path, &new_path) }
}

#[derive(Debug)]
pub struct File {
	fd: descriptor::Descriptor,
	path: String,
}

impl File {
	pub fn create(path: &str) -> Result<Self, Error> {
		let path = normalize_path(&path.to_string())?;
		let fd = open(&path, descriptor::OpenOptions::READ_WRITE | descriptor::OpenOptions::CREATE)?;
		Ok(File {
			fd,
			path: path.to_string(),
		})
	}

	pub fn open(path: &str) -> Result<Self, Error> {
		let path = normalize_path(&path.to_string())?;
		let fd = open(&path, descriptor::OpenOptions::READ_WRITE)?;
		Ok(File {
			fd,
			path: path.to_string(),
		})
	}

	pub fn len(&self) -> Result<usize, Error> {
		let st = descriptor::fstat(self.fd)?;
		Ok(st.size)
	}
}

impl Read for File {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
		descriptor::read(self.fd, buf).map_err(|_| io::Error::FsError)
	}
}

impl Write for File {
	fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
		descriptor::write(self.fd, buf).map_err(|_| io::Error::FsError)
	}
}

impl Drop for File {
	fn drop(&mut self) {
		let _ = remove_io_interface(self.fd);
	}
}

pub fn initialize() {
	let mut root = system::FileSystem::new();
	root.make(&String::from("/bin")).unwrap();
	root.make(&String::from("/dev")).unwrap();
	if DEMO.len() > 0 {
		root.mount(&String::from("/bin/demo"), &DEMO)
			.expect("Unable to mount file");
	}
	root.list().unwrap();
	unsafe {
		system::ROOT = Some(root);
	}
}

fn normalize_path(path: &String) -> Result<String, Error> {
	if path.is_empty() {
		return Err(Error::InvalidFsPath);
	}
	let is_absolute = path.starts_with('/');
	let components: Vec<&str> = path.split('/').filter(|&s| !s.is_empty()).collect();
	let mut result = Vec::new();
	for component in components {
		match component {
			"." => continue,
			".." => {
				if result.is_empty() && !is_absolute {
					result.push("..");
				} else if !result.is_empty() {
					result.pop();
				}
			}
			_ => result.push(component),
		}
	}
	let normalized = if is_absolute {
		format!("/{}", result.join("/"))
	} else {
		result.join("/")
	};
	Ok(if normalized.is_empty() { "/".to_string() } else { normalized })
}