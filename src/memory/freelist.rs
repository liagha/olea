use {
	crate::{
		arch::memory::{PhysicalAddress, VirtualAddress},
	},
	alloc::collections::linked_list::LinkedList,
	core::cmp::Ordering,
};

#[derive(Copy, Clone, Debug)]
pub enum FreeListError {
	NoValidEntry,
}

pub struct FreeListEntry<
	T: Copy
		+ PartialEq
		+ PartialOrd
		+ core::fmt::UpperHex
		+ core::ops::Sub
		+ core::ops::BitAnd<u64>
		+ core::ops::Add<usize>
		+ core::ops::AddAssign<u64>
		+ core::ops::Add<usize, Output = T>,
> {
	pub start: T,
	pub end: T,
}

impl<
		T: Copy
			+ PartialEq
			+ PartialOrd
			+ core::fmt::UpperHex
			+ core::ops::Sub
			+ core::ops::BitAnd<u64>
			+ core::ops::Add<usize>
			+ core::ops::AddAssign<u64>
			+ core::ops::Add<usize, Output = T>,
	> FreeListEntry<T>
{
	pub const fn new(start: T, end: T) -> Self {
		FreeListEntry { start, end }
	}
}

pub struct FreeList<
	T: Copy
		+ PartialEq
		+ PartialOrd
		+ core::fmt::UpperHex
		+ core::ops::Sub
		+ core::ops::BitAnd<u64>
		+ core::ops::Add<usize>
		+ core::ops::AddAssign<u64>
		+ core::ops::Add<usize, Output = T>,
> {
	pub list: LinkedList<FreeListEntry<T>>,
}

impl<
		T: Copy
			+ PartialEq
			+ PartialOrd
			+ core::fmt::UpperHex
			+ core::ops::Sub
			+ core::ops::BitAnd<u64>
			+ core::ops::Add<usize>
			+ core::ops::AddAssign<u64>
			+ core::ops::Add<usize, Output = T>,
	> FreeList<T>
{
	pub const fn new() -> Self {
		Self {
			list: LinkedList::new(),
		}
	}

	pub fn deallocate(&mut self, address: T, size: usize) {
		debug!(
			"deallocating {} bytes at {:#X} from Free List {:#X}.",
			size, address, self as *const Self as usize
		);

		let end = address + size;
		let mut cursor = self.list.cursor_front_mut();

		while let Some(node) = cursor.current() {
			let (region_start, region_end) = (node.start, node.end);

			if region_start == end {
				node.start = address;

				if let Some(prev_node) = cursor.peek_prev() {
					let prev_region_end = prev_node.end;

					if prev_region_end == address {
						prev_node.end = region_end;
						cursor.remove_current();
					}
				}

				return;
			} else if region_end == address {
				node.end = end;

				if let Some(next_node) = cursor.peek_next() {
					let next_region_start = next_node.start;

					if next_region_start == end {
						next_node.start = region_start;
						cursor.remove_current();
					}
				}

				return;
			} else if end < region_start {
				let new_entry = FreeListEntry::new(address, end);
				cursor.insert_before(new_entry);
				return;
			}

			cursor.move_next();
		}

		let new_element = FreeListEntry::new(address, end);
		self.list.push_back(new_element);
	}
}

#[cfg(not(target_os = "none"))]
#[test]
fn add_element() {
	let mut freelist = FreeList::new();
	let entry = FreeListEntry::new(0x10000, 0x100000);

	freelist.list.push_back(entry);

	let mut cursor = freelist.list.cursor_front_mut();

	while let Some(node) = cursor.peek_next() {
		assert!(node.start != 0x1000);
		assert!(node.end != 0x10000);

		cursor.move_next();
	}
}

#[cfg(not(target_os = "none"))]
#[test]
fn allocate() {
	let mut freelist = FreeList::new();
	let entry = FreeListEntry::new(0x10000, 0x100000);

	freelist.list.push_back(entry);
	let addr = freelist.allocate(0x1000, None);

	assert_eq!(addr.unwrap(), 0x10000);

	let mut cursor = freelist.list.cursor_front_mut();
	while let Some(node) = cursor.current() {
		assert_eq!(node.start, 0x11000);
		assert_eq!(node.end, 0x100000);

		cursor.move_next();
	}

	let addr = freelist.allocate(0x1000, Some(0x2000));
	let mut cursor = freelist.list.cursor_front_mut();
	assert!(cursor.current().is_some());
	if let Some(node) = cursor.current() {
		assert_eq!(node.start, 0x11000);
	}

	cursor.move_next();
	assert!(cursor.current().is_some());
	if let Some(node) = cursor.current() {
		assert_eq!(node.start, 0x13000);
	}
}

#[cfg(not(target_os = "none"))]
#[test]
fn deallocate() {
	let mut freelist = FreeList::new();
	let entry = FreeListEntry::new(0x10000, 0x100000);

	freelist.list.push_back(entry);
	let addr = freelist.allocate(0x1000, None);
	freelist.deallocate(addr.unwrap(), 0x1000);

	let mut cursor = freelist.list.cursor_front_mut();
	while let Some(node) = cursor.current() {
		assert_eq!(node.start, 0x10000);
		assert_eq!(node.end, 0x100000);

		cursor.move_next();
	}
}

impl FreeList<PhysicalAddress> {
	pub fn allocate(
		&mut self,
		size: usize,
		alignment: Option<usize>,
	) -> Result<PhysicalAddress, FreeListError> {
		debug!(
			"allocating {} bytes from Free List {:#X}.",
			size, self as *const Self as usize
		);

		let new_size = if let Some(align) = alignment {
			size + align
		} else {
			size
		};

		let mut cursor = self.list.cursor_front_mut();
		while let Some(node) = cursor.current() {
			let (region_start, region_size) = (node.start, node.end - node.start);

			match region_size.as_usize().cmp(&new_size) {
				Ordering::Greater => {
					if let Some(align) = alignment {
						let new_addr = PhysicalAddress::from(align_up!(region_start.as_usize(), align));
						node.start += (new_addr - region_start) + size as u64;
						if new_addr != region_start {
							let new_entry = FreeListEntry::new(region_start, new_addr);
							cursor.insert_before(new_entry);
						}
						return Ok(new_addr);
					} else {
						node.start += size as u64;
						return Ok(region_start);
					}
				}
				Ordering::Equal => {
					if let Some(align) = alignment {
						let new_addr = PhysicalAddress::from(align_up!(region_start.as_usize(), align));
						if new_addr != region_start {
							node.end = new_addr;
						}
						return Ok(new_addr);
					} else {
						cursor.remove_current();
						return Ok(region_start);
					}
				}
				Ordering::Less => {}
			}

			cursor.move_next();
		}

		Err(FreeListError::NoValidEntry)
	}
}

impl FreeList<VirtualAddress> {
	pub fn allocate(
		&mut self,
		size: usize,
		alignment: Option<usize>,
	) -> Result<VirtualAddress, FreeListError> {
		debug!(
			"allocating {} bytes from Free List {:#X}.",
			size, self as *const Self as usize
		);

		let new_size = if let Some(align) = alignment {
			size + align
		} else {
			size
		};

		let mut cursor = self.list.cursor_front_mut();
		while let Some(node) = cursor.current() {
			let (region_start, region_size) = (node.start, node.end - node.start);

			match region_size.as_usize().cmp(&new_size) {
				Ordering::Greater => {
					if let Some(align) = alignment {
						let new_addr = VirtualAddress::from(align_up!(region_start.as_usize(), align));
						node.start += (new_addr - region_start) + size as u64;
						if new_addr != region_start {
							let new_entry = FreeListEntry::new(region_start, new_addr);
							cursor.insert_before(new_entry);
						}
						return Ok(new_addr);
					} else {
						node.start += size as u64;
						return Ok(region_start);
					}
				}
				Ordering::Equal => {
					return if let Some(align) = alignment {
						let new_addr = VirtualAddress::from(align_up!(region_start.as_usize(), align));
						if new_addr != region_start {
							node.end = new_addr;
						}
						Ok(new_addr)
					} else {
						cursor.remove_current();
						Ok(region_start)
					}
				}
				Ordering::Less => {}
			}

			cursor.move_next();
		}

		Err(FreeListError::NoValidEntry)
	}
}
