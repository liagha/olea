use core::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
pub struct LinkedList {
	head: *mut usize,
}

unsafe impl Send for LinkedList {}

impl LinkedList {
	pub const fn new() -> Self {
		Self {
			head: core::ptr::null_mut(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.head.is_null()
	}

	pub unsafe fn push(&mut self, item: *mut usize) {
		*item = self.head as usize;
		self.head = item;
	}

	pub fn pop(&mut self) -> Option<*mut usize> {
		match self.is_empty() {
			true => None,
			false => {
				let item = self.head;
				self.head = unsafe { *item as *mut usize };
				Some(item)
			}
		}
	}

	pub fn iter(&self) -> Iter {
		Iter {
			curr: self.head,
			list: PhantomData,
		}
	}

	pub fn iter_mut(&mut self) -> IterMut {
		IterMut {
			prev: &mut self.head as *mut *mut usize as *mut usize,
			curr: self.head,
			list: PhantomData,
		}
	}
}

pub struct Iter<'a> {
	curr: *mut usize,
	list: PhantomData<&'a LinkedList>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = *mut usize;

	fn next(&mut self) -> Option<Self::Item> {
		if self.curr.is_null() {
			None
		} else {
			let item = self.curr;
			let next = unsafe { *item as *mut usize };
			self.curr = next;
			Some(item)
		}
	}
}

pub struct ListNode {
	prev: *mut usize,
	curr: *mut usize,
}

impl ListNode {
	pub fn remove(self) -> *mut usize {
		unsafe {
			*(self.prev) = *(self.curr);
		}
		self.curr
	}

	pub fn value(&self) -> *mut usize {
		self.curr
	}
}

pub struct IterMut<'a> {
	list: PhantomData<&'a mut LinkedList>,
	prev: *mut usize,
	curr: *mut usize,
}

impl<'a> Iterator for IterMut<'a> {
	type Item = ListNode;

	fn next(&mut self) -> Option<Self::Item> {
		if self.curr.is_null() {
			None
		} else {
			let res = ListNode {
				prev: self.prev,
				curr: self.curr,
			};
			self.prev = self.curr;
			self.curr = unsafe { *self.curr as *mut usize };
			Some(res)
		}
	}
}
