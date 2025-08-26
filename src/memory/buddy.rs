use crate::memory::linked_list;
use crate::sync::spinlock::Spinlock;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::cmp::{max, min};
use core::fmt;
use core::ptr::NonNull;

const MIN_ALLOC_SIZE: usize = 128;

#[derive(Debug)]
pub enum AllocatorError {
	OutOfMemory,
	TooBig,
}

#[repr(align(64))]
pub struct BuddySystem<const ORDER: usize> {
	free_list: [linked_list::LinkedList; ORDER],
}

impl<const ORDER: usize> BuddySystem<ORDER> {
	pub const fn new() -> Self {
		Self {
			free_list: [linked_list::LinkedList::new(); ORDER],
		}
	}

	pub unsafe fn init(&mut self, start: *mut u8, len: usize) {
		assert!(
			len.is_power_of_two(),
			"heap size must be a power of two, but got `{}`.",
			len
		);

		let order: usize = len.trailing_zeros().try_into().unwrap();

		assert!(
			order <= ORDER,
			"heap order `{}` exceeds maximum supported `ORDER` `{}`, increase `ORDER`.",
			order,
			ORDER
		);

		unsafe {
			self.free_list[order].push(start as *mut usize);
		}
	}

	pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocatorError> {
		let size = max(
			layout.size().next_power_of_two(),
			max(layout.align(), MIN_ALLOC_SIZE),
		);
		let order: usize = size.trailing_zeros().try_into().unwrap();

		if order >= ORDER {
			return Err(AllocatorError::TooBig);
		}

		for i in order..self.free_list.len() {
			if !self.free_list[i].is_empty() {
				for j in (order + 1..i + 1).rev() {
					if let Some(block) = self.free_list[j].pop() {
						unsafe {
							self.free_list[j - 1]
								.push((block as usize + (1 << (j - 1))) as *mut usize);
							self.free_list[j - 1].push(block);
						}
					} else {
						return Err(AllocatorError::OutOfMemory);
					}
				}

				return if let Some(addr) = self.free_list[order].pop() {
					Ok(NonNull::new(addr as *mut u8).unwrap())
				} else {
					Err(AllocatorError::OutOfMemory)
				}
			}
		}

		Err(AllocatorError::OutOfMemory)
	}

	pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
		let size = max(
			layout.size().next_power_of_two(),
			max(layout.align(), MIN_ALLOC_SIZE),
		);
		let order: usize = size.trailing_zeros().try_into().unwrap();

		unsafe {
			self.free_list[order].push(ptr.as_ptr() as *mut usize);

			let mut current_ptr = ptr.as_ptr() as usize;
			let mut current_order = order;

			'outer: while current_order < self.free_list.len() - 1 {
				let block_size = 1 << current_order;

				for block in self.free_list[current_order].iter_mut() {
					let buddy = block.value() as usize;
					if buddy == current_ptr + block_size || buddy == current_ptr - block_size {
						block.remove();
						self.free_list[current_order].pop().unwrap();
						current_ptr = min(current_ptr, buddy);
						current_order += 1;
						self.free_list[current_order].push(current_ptr as *mut usize);
						continue 'outer;
					}
				}

				break;
			}
		}
	}
}

impl<const ORDER: usize> fmt::Debug for BuddySystem<ORDER> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for order in (0..self.free_list.len()).rev() {
			if !self.free_list[order].is_empty() {
				write!(f, "Block size {:>8}: ", 1 << order)?;

				for block in self.free_list[order].iter() {
					write!(f, "0x{:x} ", block as usize)?;
				}

				writeln!(f)?;
			}
		}

		Ok(())
	}
}

pub struct LockedHeap<const ORDER: usize>(Spinlock<BuddySystem<ORDER>>);

impl<const ORDER: usize> LockedHeap<ORDER> {
	pub const fn new() -> Self {
		LockedHeap(Spinlock::new(BuddySystem::<ORDER>::new()))
	}

	pub unsafe fn init(&self, start: *mut u8, len: usize) {
		unsafe {
			self.0.lock().init(start, len);
		}
	}
}

impl<const ORDER: usize> fmt::Debug for LockedHeap<ORDER> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.lock().fmt(f)
	}
}

unsafe impl<const ORDER: usize> GlobalAlloc for LockedHeap<ORDER> {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		self.0
			.lock()
			.alloc(layout)
			.ok()
			.map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
	}
}
