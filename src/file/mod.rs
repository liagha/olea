#![allow(dead_code)]

pub mod vfs;
pub mod handle;
pub mod standard;
pub mod descriptor;
pub mod error;

use alloc::format;
pub use error::Error;

use {
	crate::{
		format::Debug,
		file::vfs::VirtualSystem,
		io::{self, Write, Read},
		scheduler::{insert_io_interface, remove_io_interface},
	},
	alloc::{
		string::{String, ToString},
		vec::Vec,
	},
};

static DEMO: &[u8] = include_bytes!("../../demo/hello");

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
	File,
	Directory,
	Symlink,
}

pub fn lsdir() -> Result<(), Error> {
	unsafe { vfs::ROOT.as_mut().unwrap().lsdir() }
}

pub fn mkdir(path: &String) -> Result<(), Error> {
	let path = normalize_path(path)?;
	unsafe { vfs::ROOT.as_mut().unwrap().mkdir(&path) }
}

pub fn open(name: &str, flags: descriptor::OpenOptions) -> Result<descriptor::Descriptor, Error> {
	debug!("open {}, {:?}.", name, flags);
	let name = normalize_path(&name.to_string())?;
	let fs = unsafe { vfs::ROOT.as_mut().unwrap() };
	if let Ok(file) = fs.open(&name, flags) {
		let fd = insert_io_interface(file).map_err(|_| Error::IoError)?;
		Ok(fd)
	} else {
		Err(Error::InvalidArgument)
	}
}

pub fn mount(path: &String, slice: &'static [u8]) -> Result<(), Error> {
	let path = normalize_path(path)?;
	unsafe { vfs::ROOT.as_mut().unwrap().mount(&path, slice) }
}

pub fn symlink(target: &String, link: &String) -> Result<(), Error> {
	let target = normalize_path(target)?;
	let link = normalize_path(link)?;
	unsafe { vfs::ROOT.as_mut().unwrap().symlink(&target, &link) }
}

pub fn unlink(path: &String) -> Result<(), Error> {
	let path = normalize_path(path)?;
	unsafe { vfs::ROOT.as_mut().unwrap().unlink(&path) }
}

pub fn rename(old_path: &String, new_path: &String) -> Result<(), Error> {
	let old_path = normalize_path(old_path)?;
	let new_path = normalize_path(new_path)?;
	unsafe { vfs::ROOT.as_mut().unwrap().rename(&old_path, &new_path) }
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
		let fstat = descriptor::fstat(self.fd)?;
		Ok(fstat.file_size)
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
	let mut root = vfs::FileSystem::new();
	root.mkdir(&String::from("/bin")).unwrap();
	root.mkdir(&String::from("/dev")).unwrap();
	if DEMO.len() > 0 {
		root.mount(&String::from("/bin/demo"), &DEMO)
			.expect("Unable to mount file");
	}
	root.lsdir().unwrap();
	unsafe {
		vfs::ROOT = Some(root);
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