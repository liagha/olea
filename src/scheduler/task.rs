#![allow(dead_code)]

use {
	crate::{
		arch::{
			kernel::processor::utilities::most_significant_bit,
			memory::{
				get_boot_stack, paging::{
					get_kernel_root_page_table,
					BasePageSize, PageSize,
				},
				physical::deallocate,
				PhysicalAddress,
				VirtualAddress,
			},
		},
		consts::*,
		file::{
			vfs::{
				standard::{GenericStandardError, GenericStandardInput, GenericStandardOutput},
				
				descriptor::{
					Descriptor, Interface, STANDARD_ERROR, STANDARD_INPUT, STANDARD_OUTPUT,
				},
			},
		},
		format,
	},
	alloc::{
		boxed::Box, collections::{BTreeMap, VecDeque}, rc::Rc,
		sync::Arc,
	},
	core::cell::RefCell,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TaskStatus {
	Invalid,
	Ready,
	Running,
	Blocked,
	Finished,
	Idle,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskId(u32);

impl TaskId {
	pub const fn into(self) -> u32 {
		self.0
	}

	pub const fn from(x: u32) -> Self {
		TaskId(x)
	}
}

impl format::Display for TaskId {
	fn fmt(&self, f: &mut format::Formatter) -> format::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskPriority(u8);

impl TaskPriority {
	pub const fn into(self) -> u8 {
		self.0
	}

	pub const fn from(x: u8) -> Self {
		TaskPriority(x)
	}
}

impl format::Display for TaskPriority {
	fn fmt(&self, f: &mut format::Formatter) -> format::Result {
		write!(f, "{}", self.0)
	}
}

pub const REALTIME_PRIORITY: TaskPriority = TaskPriority::from(NO_PRIORITIES as u8 - 1);
pub const HIGH_PRIORITY: TaskPriority = TaskPriority::from(24);
pub const NORMAL_PRIORITY: TaskPriority = TaskPriority::from(16);
pub const LOW_PRIORITY: TaskPriority = TaskPriority::from(0);

pub struct PriorityTaskQueue {
	queues: [VecDeque<Rc<RefCell<Task>>>; NO_PRIORITIES],
	priority_bitmap: usize,
}

impl PriorityTaskQueue {
	pub const fn new() -> PriorityTaskQueue {
		const VALUE: VecDeque<Rc<RefCell<Task>>> = VecDeque::new();

		PriorityTaskQueue {
			queues: [VALUE; NO_PRIORITIES],
			priority_bitmap: 0,
		}
	}

	pub fn push(&mut self, task: Rc<RefCell<Task>>) {
		let i: usize = task.borrow().priority.into().into();
		self.priority_bitmap |= 1 << i;
		self.queues[i].push_back(task.clone());
	}

	fn pop_from_queue(&mut self, queue_index: usize) -> Option<Rc<RefCell<Task>>> {
		let task = self.queues[queue_index].pop_front();
		if self.queues[queue_index].is_empty() {
			self.priority_bitmap &= !(1 << queue_index);
		}

		task
	}

	pub fn pop(&mut self) -> Option<Rc<RefCell<Task>>> {
		if let Some(i) = most_significant_bit(self.priority_bitmap) {
			return self.pop_from_queue(i);
		}

		None
	}

	pub fn pop_with_priority(&mut self, priority: TaskPriority) -> Option<Rc<RefCell<Task>>> {
		if let Some(i) = most_significant_bit(self.priority_bitmap) {
			if i >= priority.into().into() {
				return self.pop_from_queue(i);
			}
		}

		None
	}
}

#[allow(dead_code)]
pub trait Stack {
	fn top(&self) -> VirtualAddress;
	fn bottom(&self) -> VirtualAddress;
	fn interrupt_top(&self) -> VirtualAddress;
	fn interrupt_bottom(&self) -> VirtualAddress;
}

#[derive(Copy, Clone)]
#[repr(C, align(64))]
pub struct TaskStack {
	buffer: [u8; STACK_SIZE],
	ist_buffer: [u8; INTERRUPT_STACK_SIZE],
}

impl Default for TaskStack {
	fn default() -> Self {
		Self::new()
	}
}

impl TaskStack {
	pub const fn new() -> TaskStack {
		TaskStack {
			buffer: [0; STACK_SIZE],
			ist_buffer: [0; INTERRUPT_STACK_SIZE],
		}
	}
}

impl Stack for TaskStack {
	fn top(&self) -> VirtualAddress {
		VirtualAddress::from(self.buffer.as_ptr() as usize + STACK_SIZE - 16)
	}

	fn bottom(&self) -> VirtualAddress {
		VirtualAddress::from(self.buffer.as_ptr() as usize)
	}

	fn interrupt_top(&self) -> VirtualAddress {
		VirtualAddress::from(self.ist_buffer.as_ptr() as usize + INTERRUPT_STACK_SIZE - 16)
	}

	fn interrupt_bottom(&self) -> VirtualAddress {
		VirtualAddress::from(self.ist_buffer.as_ptr() as usize)
	}
}

#[repr(align(64))]
pub struct Task {
	pub id: TaskId,
	pub priority: TaskPriority,
	pub status: TaskStatus,
	pub last_stack_pointer: VirtualAddress,
	pub stack: Box<dyn Stack>,
	pub root_page_table: PhysicalAddress,
	pub fd_map: BTreeMap<Descriptor, Arc<dyn Interface>>,
}

impl Task {
	pub fn new_idle(id: TaskId) -> Task {
		Task {
			id,
			priority: LOW_PRIORITY,
			status: TaskStatus::Idle,
			last_stack_pointer: VirtualAddress::zero(),
			stack: Box::new(get_boot_stack()),
			root_page_table: get_kernel_root_page_table(),
			fd_map: BTreeMap::new(),
		}
	}

	pub fn new(id: TaskId, status: TaskStatus, priority: TaskPriority) -> Task {
		let mut fd_map: BTreeMap<Descriptor, Arc<dyn Interface>> = BTreeMap::new();
		fd_map
			.try_insert(STANDARD_INPUT, Arc::new(GenericStandardInput::new()))
			.unwrap();
		fd_map
			.try_insert(STANDARD_OUTPUT, Arc::new(GenericStandardOutput::new()))
			.unwrap();
		fd_map
			.try_insert(STANDARD_ERROR, Arc::new(GenericStandardError::new()))
			.unwrap();

		Task {
			id,
			priority,
			status,
			last_stack_pointer: VirtualAddress::zero(),
			stack: Box::new(TaskStack::new()),
			root_page_table: get_kernel_root_page_table(),
			fd_map,
		}
	}
}

pub trait TaskFrame {
	fn create_stack_frame(&mut self, func: extern "C" fn());
}

impl Drop for Task {
	fn drop(&mut self) {
		if self.root_page_table != get_kernel_root_page_table() {
			debug!(
				"deallocate page table 0x{:x} of task {}.",
				self.root_page_table, self.id
			);
			deallocate(self.root_page_table, BasePageSize::SIZE);
		}
	}
}
