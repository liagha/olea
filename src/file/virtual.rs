use {
	crate::{
		format,
		file::{
			check_path, 
			NodeKind, SeekFrom, Vfs, VfsNode, VfsNodeDirectory, VfsNodeFile,
			error::Error,
			initial::{RamHandle, RomHandle},
			descriptor::{
				OpenOptions, FileStatus, IoInterface,
			},
		},
		sync::spinlock::*,
	},
	core::any::Any,
	alloc::{
		boxed::Box,
		collections::BTreeMap,
		string::String,
		sync::Arc,
		vec::Vec,
	},
};

#[derive(Debug)]
struct VfsDirectory {
	children: BTreeMap<String, Box<dyn Any + Send + Sync>>,
}

impl VfsDirectory {
	pub fn new() -> Self {
		VfsDirectory {
			children: BTreeMap::new(),
		}
	}

	fn get_mut<T: VfsNode + Any>(&mut self, name: &String) -> Option<&mut T> {
		if let Some(b) = self.children.get_mut(name) {
			return b.downcast_mut::<T>();
		}
		None
	}

	fn get<T: VfsNode + Any>(&mut self, name: &String) -> Option<&T> {
		if let Some(b) = self.children.get_mut(name) {
			return b.downcast_ref::<T>();
		}
		None
	}
}

impl VfsNode for VfsDirectory {
	fn get_kind(&self) -> NodeKind {
		NodeKind::Directory
	}
}

impl VfsNodeDirectory for VfsDirectory {
	fn traverse_mkdir(&mut self, components: &mut Vec<&str>) -> Result<(), Error> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			{
				if let Some(directory) = self.get_mut::<VfsDirectory>(&node_name) {
					return directory.traverse_mkdir(components);
				}
			}

			let mut directory = Box::new(VfsDirectory::new());
			let result = directory.traverse_mkdir(components);
			self.children.insert(node_name, directory);

			result
		} else {
			Ok(())
		}
	}

	fn traverse_lsdir(&self, mut tabs: String) -> Result<(), Error> {
		tabs.push_str("  ");
		for (name, node) in self.children.iter() {
			if let Some(directory) = node.downcast_ref::<VfsDirectory>() {
				info!("{}{} ({:?})", tabs, name, self.get_kind());
				directory.traverse_lsdir(tabs.clone())?;
			} else if let Some(file) = node.downcast_ref::<VfsFile>() {
				info!("{}{} ({:?})", tabs, name, file.get_kind());
			} else {
				info!("{}{} (Unknown))", tabs, name);
			}
		}

		Ok(())
	}

	fn traverse_open(
		&mut self,
		components: &mut Vec<&str>,
		flags: OpenOptions,
	) -> Result<Arc<dyn IoInterface>, Error> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			if components.is_empty() == true {
				if let Some(file) = self.get_mut::<VfsFile>(&node_name) {
					return file.get_handle(flags);
				}
			}

			if components.is_empty() == true {
				if flags.contains(OpenOptions::CREATE) {
					let file = Box::new(VfsFile::new());
					let result = file.get_handle(flags);
					self.children.insert(node_name, file);

					result
				} else {
					Err(Error::InvalidArgument)
				}
			} else {
				if let Some(directory) = self.get_mut::<VfsDirectory>(&node_name) {
					directory.traverse_open(components, flags)
				} else {
					Err(Error::InvalidArgument)
				}
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}

	fn traverse_mount(&mut self, components: &mut Vec<&str>, slice: &'static [u8]) -> Result<(), Error> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			if components.is_empty() == true {
				let file = Box::new(VfsFile::new_from_rom(slice));
				self.children.insert(node_name, file);

				Ok(())
			} else {
				if let Some(directory) = self.get_mut::<VfsDirectory>(&node_name) {
					directory.traverse_mount(components, slice)
				} else {
					Err(Error::InvalidArgument)
				}
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}
}

#[derive(Debug, Clone)]
enum DataHandle {
	RAM(RamHandle),
	ROM(RomHandle),
}

#[derive(Debug, Clone)]
struct VfsFile {
	data: DataHandle,
}

impl VfsFile {
	pub fn new() -> Self {
		VfsFile {
			data: DataHandle::RAM(RamHandle::new(true)),
		}
	}

	pub fn new_from_rom(slice: &'static [u8]) -> Self {
		VfsFile {
			data: DataHandle::ROM(RomHandle::new(slice)),
		}
	}
}

impl VfsNode for VfsFile {
	fn get_kind(&self) -> NodeKind {
		NodeKind::File
	}
}

impl VfsNodeFile for VfsFile {
	fn get_handle(&self, opt: OpenOptions) -> Result<Arc<dyn IoInterface>, Error> {
		match self.data {
			DataHandle::RAM(ref data) => Ok(Arc::new(VfsFile {
				data: DataHandle::RAM(data.get_handle(opt)),
			})),
			DataHandle::ROM(ref data) => Ok(Arc::new(VfsFile {
				data: DataHandle::ROM(data.get_handle(opt)),
			})),
		}
	}
}

impl format::Write for VfsFile {
	fn write_str(&mut self, s: &str) -> format::Result {
		match self.data {
			DataHandle::RAM(ref mut data) => data.write_str(s),
			_ => Err(format::Error),
		}
	}
}

impl IoInterface for VfsFile {
	fn read(&self, buf: &mut [u8]) -> Result<usize, Error> {
		match self.data {
			DataHandle::RAM(ref data) => data.read(buf),
			DataHandle::ROM(ref data) => data.read(buf),
		}
	}

	fn write(&self, buf: &[u8]) -> Result<usize, Error> {
		match self.data {
			DataHandle::RAM(ref data) => data.write(buf),
			_ => Err(Error::BadFileDescriptor),
		}
	}

	fn seek(&self, style: SeekFrom) -> Result<usize, Error> {
		match self.data {
			DataHandle::RAM(ref data) => data.seek(style),
			DataHandle::ROM(ref data) => data.seek(style),
		}
	}

	fn fstat(&self) -> Result<FileStatus, Error> {
		let file_size = match self.data {
			DataHandle::RAM(ref data) => data.len(),
			DataHandle::ROM(ref data) => data.len(),
		};

		Ok(FileStatus { file_size })
	}
}

#[derive(Debug)]
pub struct Fs {
	handle: Spinlock<VfsDirectory>,
}

impl Fs {
	pub fn new() -> Fs {
		Fs {
			handle: Spinlock::new(VfsDirectory::new()),
		}
	}
}

impl Vfs for Fs {
	fn mkdir(&mut self, path: &String) -> Result<(), Error> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_mkdir(&mut components)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	fn lsdir(&self) -> Result<(), Error> {
		info!("/");

		self.handle.lock().traverse_lsdir(String::from(""))
	}

	fn open(&mut self, path: &str, flags: OpenOptions) -> Result<Arc<dyn IoInterface>, Error> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_open(&mut components, flags)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	fn mount(&mut self, path: &String, slice: &'static [u8]) -> Result<(), Error> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_mount(&mut components, slice)
		} else {
			Err(Error::InvalidFsPath)
		}
	}
}
