use alloc::string::ToString;
use {
	crate::{
		format::{
			self, Debug,
		},
		file::{
			NodeKind,
			handle::{RamHandle, RomHandle},
			descriptor::{OpenOptions, Status, Interface},
			error::Error,
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

#[derive(Debug, Clone, Copy)]
pub struct Permissions {
	owner_read: bool,
	owner_write: bool,
	owner_execute: bool,
	group_read: bool,
	group_write: bool,
	group_execute: bool,
	others_read: bool,
	others_write: bool,
	others_execute: bool,
}

impl Permissions {
	pub fn new(mode: u16) -> Self {
		Permissions {
			owner_read: mode & 0o400 != 0,
			owner_write: mode & 0o200 != 0,
			owner_execute: mode & 0o100 != 0,
			group_read: mode & 0o040 != 0,
			group_write: mode & 0o020 != 0,
			group_execute: mode & 0o010 != 0,
			others_read: mode & 0o004 != 0,
			others_write: mode & 0o002 != 0,
			others_execute: mode & 0o001 != 0,
		}
	}

	pub fn can_read(&self, _uid: u32, _gid: u32) -> bool {
		self.owner_read || self.group_read || self.others_read
	}

	pub fn can_write(&self, _uid: u32, _gid: u32) -> bool {
		self.owner_write || self.group_write || self.others_write
	}

	pub fn can_execute(&self, _uid: u32, _gid: u32) -> bool {
		self.owner_execute || self.group_execute || self.others_execute
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Metadata {
	permissions: Permissions,
	uid: u32,
	gid: u32,
	atime: u64,
	mtime: u64,
	ctime: u64,
}

impl Metadata {
	pub fn new() -> Self {
		Metadata {
			permissions: Permissions::new(0o644),
			uid: 0,
			gid: 0,
			atime: 0,
			mtime: 0,
			ctime: 0,
		}
	}
}

pub trait Node: Debug + Send + Sync {
	fn get_kind(&self) -> NodeKind;
	fn get_metadata(&self) -> Metadata;
	fn get_handle(&self, _opt: OpenOptions) -> Result<Arc<dyn Interface>, Error> { Err(Error::NotImplemented) }
	fn traverse_mkdir(&mut self, _components: &mut Vec<&str>, _metadata: Metadata) -> Result<(), Error> { Err(Error::NotImplemented) }
	fn traverse_lsdir(&self, _tabs: String) -> Result<(), Error> { Err(Error::NotImplemented) }
	fn traverse_open(&mut self, _components: &mut Vec<&str>, _flags: OpenOptions, _visited: &mut Vec<String>) -> Result<Arc<dyn Interface>, Error> { Err(Error::NotImplemented) }
	fn traverse_mount(&mut self, _components: &mut Vec<&str>, _slice: &'static [u8]) -> Result<(), Error> { Err(Error::NotImplemented) }
	fn traverse_unlink(&mut self, _components: &mut Vec<&str>) -> Result<(), Error> { Err(Error::NotImplemented) }
	fn traverse_rename(&mut self, _old_components: &mut Vec<&str>, _new_components: &mut Vec<&str>) -> Result<(), Error> { Err(Error::NotImplemented) }
	fn traverse_symlink(&mut self, _target_components: &mut Vec<&str>, _link_components: &mut Vec<&str>) -> Result<(), Error> { Err(Error::NotImplemented) }
}

pub trait VirtualSystem: Debug + Send + Sync {
	fn mkdir(&mut self, path: &String) -> Result<(), Error>;
	fn lsdir(&self) -> Result<(), Error>;
	fn open(&mut self, path: &str, flags: OpenOptions) -> Result<Arc<dyn Interface>, Error>;
	fn mount(&mut self, path: &String, slice: &'static [u8]) -> Result<(), Error>;
	fn symlink(&mut self, target: &String, link: &String) -> Result<(), Error>;
	fn unlink(&mut self, path: &String) -> Result<(), Error>;
	fn rename(&mut self, old_path: &String, new_path: &String) -> Result<(), Error>;
}

pub static mut ROOT: Option<FileSystem> = None;

#[derive(Debug)]
struct Directory {
	children: BTreeMap<String, Box<dyn Any + Send + Sync>>,
	metadata: Metadata,
}

impl Directory {
	pub fn new() -> Self {
		Directory {
			children: BTreeMap::new(),
			metadata: Metadata {
				permissions: Permissions::new(0o755),
				uid: 0,
				gid: 0,
				atime: 0,
				mtime: 0,
				ctime: 0,
			},
		}
	}

	fn get_mut<T: Node + Any>(&mut self, name: &String) -> Option<&mut T> {
		self.children.get_mut(name).and_then(|b| b.downcast_mut::<T>())
	}

	fn get<T: Node + Any>(&self, name: &String) -> Option<&T> {
		self.children.get(name).and_then(|b| b.downcast_ref::<T>())
	}
}

impl Node for Directory {
	fn get_kind(&self) -> NodeKind {
		NodeKind::Directory
	}

	fn get_metadata(&self) -> Metadata {
		self.metadata.clone()
	}
}

impl Directory {
	fn traverse_mkdir(&mut self, components: &mut Vec<&str>, metadata: Metadata) -> Result<(), Error> {
		if !self.metadata.permissions.can_write(0, 0) {
			return Err(Error::PermissionDenied);
		}
		if let Some(component) = components.pop() {
			let node_name = String::from(component);
			if let Some(directory) = self.get_mut::<Directory>(&node_name) {
				return directory.traverse_mkdir(components, metadata);
			}
			let mut directory = Box::new(Directory {
				children: BTreeMap::new(),
				metadata,
			});
			let result = directory.traverse_mkdir(components, metadata);
			self.children.insert(node_name, directory);
			result
		} else {
			Ok(())
		}
	}

	fn traverse_lsdir(&self, mut tabs: String) -> Result<(), Error> {
		tabs.push_str("  ");
		for (name, node) in self.children.iter() {
			if let Some(directory) = node.downcast_ref::<Directory>() {
				info!("{}{} ({:?})", tabs, name, directory.get_kind());
				directory.traverse_lsdir(tabs.clone())?;
			} else if let Some(file) = node.downcast_ref::<File>() {
				info!("{}{} ({:?})", tabs, name, file.get_kind());
			} else if let Some(symlink) = node.downcast_ref::<SymbolLink>() {
				info!("{}{} ({:?} -> {})", tabs, name, symlink.get_kind(), symlink.target);
			} else {
				info!("{}{} (Unknown)", tabs, name);
			}
		}
		Ok(())
	}

	fn traverse_open(&mut self, components: &mut Vec<&str>, flags: OpenOptions, visited: &mut Vec<String>) -> Result<Arc<dyn Interface>, Error> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);
			if visited.contains(&node_name) {
				return Err(Error::SymlinkLoop);
			}
			if components.is_empty() {
				if let Some(file) = self.get_mut::<File>(&node_name) {
					if !file.get_metadata().permissions.can_read(0, 0) {
						return Err(Error::PermissionDenied);
					}
					return file.get_handle(flags);
				}
				let symlink_target = self.get::<SymbolLink>(&node_name).map(|symlink| {
					if !symlink.get_metadata().permissions.can_read(0, 0) {
						None
					} else {
						Some(symlink.target.clone())
					}
				});
				if let Some(Some(target)) = symlink_target {
					visited.push(node_name);
					let mut target_components: Vec<&str> = target.split('/').filter(|&s| !s.is_empty()).collect();
					target_components.reverse();
					return self.traverse_open(&mut target_components, flags, visited);
				}
				if flags.contains(OpenOptions::CREATE) {
					if !self.metadata.permissions.can_write(0, 0) {
						return Err(Error::PermissionDenied);
					}
					let file = Box::new(File::new());
					let result = file.get_handle(flags);
					self.children.insert(node_name, file);
					return result;
				}
				Err(Error::FileNotFound)
			} else {
				let symlink_target = self.get::<SymbolLink>(&node_name).map(|symlink| {
					if !symlink.get_metadata().permissions.can_read(0, 0) {
						None
					} else {
						Some(symlink.target.clone())
					}
				});
				if let Some(Some(target)) = symlink_target {
					visited.push(node_name);
					let mut target_components: Vec<&str> = target.split('/').filter(|&s| !s.is_empty()).collect();
					target_components.reverse();
					return self.traverse_open(&mut target_components, flags, visited);
				}
				self.get_mut::<Directory>(&node_name)
					.ok_or(Error::NotADirectory)?
					.traverse_open(components, flags, visited)
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}

	fn traverse_mount(&mut self, components: &mut Vec<&str>, slice: &'static [u8]) -> Result<(), Error> {
		if !self.metadata.permissions.can_write(0, 0) {
			return Err(Error::PermissionDenied);
		}
		if let Some(component) = components.pop() {
			let node_name = String::from(component);
			if components.is_empty() {
				let file = Box::new(File::new_from_rom(slice));
				self.children.insert(node_name, file);
				Ok(())
			} else {
				self.get_mut::<Directory>(&node_name)
					.ok_or(Error::NotADirectory)?
					.traverse_mount(components, slice)
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}

	fn traverse_unlink(&mut self, components: &mut Vec<&str>) -> Result<(), Error> {
		if !self.metadata.permissions.can_write(0, 0) {
			return Err(Error::PermissionDenied);
		}
		if let Some(component) = components.pop() {
			let node_name = String::from(component);
			if components.is_empty() {
				if self.children.contains_key(&node_name) {
					if let Some(directory) = self.get::<Directory>(&node_name) {
						if !directory.children.is_empty() {
							return Err(Error::IsADirectory);
						}
					}
					self.children.remove(&node_name);
					Ok(())
				} else {
					Err(Error::FileNotFound)
				}
			} else {
				self.get_mut::<Directory>(&node_name)
					.ok_or(Error::NotADirectory)?
					.traverse_unlink(components)
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}

	fn traverse_rename(&mut self, old_components: &mut Vec<&str>, new_components: &mut Vec<&str>) -> Result<(), Error> {
		if !self.metadata.permissions.can_write(0, 0) {
			return Err(Error::PermissionDenied);
		}
		if let Some(old_component) = old_components.pop() {
			let old_name = String::from(old_component);
			if let Some(new_component) = new_components.pop() {
				let new_name = String::from(new_component);
				if old_components.is_empty() && new_components.is_empty() {
					if self.children.contains_key(&new_name) {
						return Err(Error::AlreadyExists);
					}
					if let Some(node) = self.children.remove(&old_name) {
						self.children.insert(new_name, node);
						Ok(())
					} else {
						Err(Error::FileNotFound)
					}
				} else {
					self.get_mut::<Directory>(&old_name)
						.ok_or(Error::NotADirectory)?
						.traverse_rename(old_components, new_components)
				}
			} else {
				Err(Error::InvalidArgument)
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}

	fn traverse_symlink(&mut self, target_components: &mut Vec<&str>, link_components: &mut Vec<&str>) -> Result<(), Error> {
		if !self.metadata.permissions.can_write(0, 0) {
			return Err(Error::PermissionDenied);
		}
		if let Some(link_component) = link_components.pop() {
			let link_name = String::from(link_component);
			if link_components.is_empty() {
				if self.children.contains_key(&link_name) {
					return Err(Error::AlreadyExists);
				}
				let target = target_components.iter().rev().map(|s| *s).collect::<Vec<&str>>().join("/");
				let symlink = Box::new(SymbolLink::new(&target));
				self.children.insert(link_name, symlink);
				Ok(())
			} else {
				self.get_mut::<Directory>(&link_name)
					.ok_or(Error::NotADirectory)?
					.traverse_symlink(target_components, link_components)
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
struct File {
	data: DataHandle,
	metadata: Metadata,
}

impl File {
	pub fn new() -> Self {
		File {
			data: DataHandle::RAM(RamHandle::new(true)),
			metadata: Metadata::new(),
		}
	}

	pub fn new_from_rom(slice: &'static [u8]) -> Self {
		File {
			data: DataHandle::ROM(RomHandle::new(slice)),
			metadata: Metadata::new(),
		}
	}
}

impl Node for File {
	fn get_kind(&self) -> NodeKind {
		NodeKind::File
	}

	fn get_metadata(&self) -> Metadata {
		self.metadata.clone()
	}
}

impl File {
	fn get_handle(&self, opt: OpenOptions) -> Result<Arc<dyn Interface>, Error> {
		match self.data {
			DataHandle::RAM(ref data) => Ok(Arc::new(File {
				data: DataHandle::RAM(data.get_handle(opt)),
				metadata: self.metadata.clone(),
			})),
			DataHandle::ROM(ref data) => Ok(Arc::new(File {
				data: DataHandle::ROM(data.get_handle(opt)),
				metadata: self.metadata.clone(),
			})),
		}
	}
}

impl format::Write for File {
	fn write_str(&mut self, s: &str) -> format::Result {
		match self.data {
			DataHandle::RAM(ref mut data) => data.write_str(s),
			_ => Err(format::Error),
		}
	}
}

impl Interface for File {
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

	fn seek(&self, style: super::descriptor::SeekFrom) -> Result<usize, Error> {
		match self.data {
			DataHandle::RAM(ref data) => data.seek(style),
			DataHandle::ROM(ref data) => data.seek(style),
		}
	}

 fn fstat(&self) -> Result<Status, Error> {
		let size = match self.data {
			DataHandle::RAM(ref data) => data.len(),
			DataHandle::ROM(ref data) => data.len(),
		};
		Ok(Status { size })
	}
}

#[derive(Debug)]
struct SymbolLink {
	target: String,
	metadata: Metadata,
}

impl SymbolLink {
	pub fn new(target: &str) -> Self {
		SymbolLink {
			target: target.to_string(),
			metadata: Metadata::new(),
		}
	}
}

impl Node for SymbolLink {
	fn get_kind(&self) -> NodeKind {
		NodeKind::Symlink
	}

	fn get_metadata(&self) -> Metadata {
		self.metadata.clone()
	}
}

#[derive(Debug)]
pub struct FileSystem {
	handle: Spinlock<Directory>,
}

impl FileSystem {
	pub fn new() -> FileSystem {
		FileSystem {
			handle: Spinlock::new(Directory::new()),
		}
	}
}

impl VirtualSystem for FileSystem {
	fn mkdir(&mut self, path: &String) -> Result<(), Error> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").filter(|&s| !s.is_empty()).collect();
			components.reverse();
			self.handle.lock().traverse_mkdir(&mut components, Metadata::new())
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	fn lsdir(&self) -> Result<(), Error> {
		info!("/");
		self.handle.lock().traverse_lsdir(String::from(""))
	}

	fn open(&mut self, path: &str, flags: OpenOptions) -> Result<Arc<dyn Interface>, Error> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").filter(|&s| !s.is_empty()).collect();
			components.reverse();
			let mut visited = Vec::new();
			self.handle.lock().traverse_open(&mut components, flags, &mut visited)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	fn mount(&mut self, path: &String, slice: &'static [u8]) -> Result<(), Error> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").filter(|&s| !s.is_empty()).collect();
			components.reverse();
			self.handle.lock().traverse_mount(&mut components, slice)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	fn symlink(&mut self, target: &String, link: &String) -> Result<(), Error> {
		if check_path(target) && check_path(link) {
			let mut target_components: Vec<&str> = target.split("/").filter(|&s| !s.is_empty()).collect();
			target_components.reverse();
			let mut link_components: Vec<&str> = link.split("/").filter(|&s| !s.is_empty()).collect();
			link_components.reverse();
			self.handle.lock().traverse_symlink(&mut target_components, &mut link_components)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	fn unlink(&mut self, path: &String) -> Result<(), Error> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").filter(|&s| !s.is_empty()).collect();
			components.reverse();
			self.handle.lock().traverse_unlink(&mut components)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	fn rename(&mut self, old_path: &String, new_path: &String) -> Result<(), Error> {
		if check_path(old_path) && check_path(new_path) {
			let mut old_components: Vec<&str> = old_path.split("/").filter(|&s| !s.is_empty()).collect();
			old_components.reverse();
			let mut new_components: Vec<&str> = new_path.split("/").filter(|&s| !s.is_empty()).collect();
			new_components.reverse();
			self.handle.lock().traverse_rename(&mut old_components, &mut new_components)
		} else {
			Err(Error::InvalidFsPath)
		}
	}
}

pub fn check_path(path: &str) -> bool {
	!path.is_empty()
}