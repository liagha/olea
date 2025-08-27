use {
	super::save_interrupt,
	crate::{
		scheduler::error::Error,
		consts::*,
		scheduler::task::*,
		file::{
			vfs::{
				descriptor::{Descriptor, Interface},
			},
		},
		arch::{
			memory::{
				PhysicalAddress, VirtualAddress,
				paging::drop_user_space,
			},
			kernel::scheduling::switch,
		},
	},
	core::{
		cell::RefCell,
		sync::atomic::{AtomicU32, Ordering},
	},
	alloc::{
		rc::Rc,
		sync::Arc,
		collections::{BTreeMap, VecDeque}
	},
};

static TID_COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct Scheduler {
	current: Rc<RefCell<Task>>,
	idle: Rc<RefCell<Task>>,
	ready: PriorityTaskQueue,
	finished: VecDeque<TaskId>,
	tasks: BTreeMap<TaskId, Rc<RefCell<Task>>>,
}

impl Scheduler {
	pub fn new() -> Scheduler {
		let task_id = TaskId::from(TID_COUNTER.fetch_add(1, Ordering::SeqCst));
		let idle = Rc::new(RefCell::new(Task::new_idle(task_id)));
		let mut tasks = BTreeMap::new();

		tasks.insert(task_id, idle.clone());

		Scheduler {
			current: idle.clone(),
			idle: idle.clone(),
			ready: PriorityTaskQueue::new(),
			finished: VecDeque::<TaskId>::new(),
			tasks,
		}
	}

	fn get_tid(&self) -> TaskId {
		loop {
			let id = TaskId::from(TID_COUNTER.fetch_add(1, Ordering::SeqCst));

			if !self.tasks.contains_key(&id) {
				return id;
			}
		}
	}

	pub fn spawn(&mut self, func: extern "C" fn(), priority: TaskPriority) -> Result<TaskId, Error> {
		let closure = || {
			let priority_number: usize = priority.into().into();

			if priority_number >= NO_PRIORITIES {
				return Err(Error::BadPriority);
			}

			let tid = self.get_tid();
			let task = Rc::new(RefCell::new(Task::new(tid, TaskStatus::Ready, priority)));

			task.borrow_mut().create_stack_frame(func);

			self.ready.push(task.clone());
			self.tasks.insert(tid, task);

			info!("creating task {}.", tid);

			Ok(tid)
		};

		save_interrupt(closure)
	}

	fn cleanup(&mut self) {
		drop_user_space();

		self.current.borrow_mut().status = TaskStatus::Finished;
	}

	pub fn exit(&mut self) -> ! {
		let closure = || {
			if self.current.borrow().status != TaskStatus::Idle {
				info!("finished task with id {}.", self.current.borrow().id);
				self.cleanup();
			} else {
				panic!("unable to terminate idle task.");
			}
		};

		save_interrupt(closure);

		self.reschedule();

		panic!("exit failed!");
	}

	pub fn abort(&mut self) -> ! {
		let closure = || {
			if self.current.borrow().status != TaskStatus::Idle {
				info!("abort task with id {}.", self.current.borrow().id);
				self.cleanup();
			} else {
				panic!("unable to terminate idle task.");
			}
		};

		save_interrupt(closure);

		self.reschedule();

		panic!("abort failed!");
	}

	pub fn block_current_task(&mut self) -> Rc<RefCell<Task>> {
		let closure = || {
			if self.current.borrow().status == TaskStatus::Running {
				debug!("block task {}.", self.current.borrow().id);

				self.current.borrow_mut().status = TaskStatus::Blocked;
				self.current.clone()
			} else {
				panic!("unable to block task {}.", self.current.borrow().id);
			}
		};

		save_interrupt(closure)
	}

	pub fn wakeup_task(&mut self, task: Rc<RefCell<Task>>) {
		let closure = || {
			if task.borrow().status == TaskStatus::Blocked {
				debug!("wakeup task {}.", task.borrow().id);

				task.borrow_mut().status = TaskStatus::Ready;
				self.ready.push(task.clone());
			}
		};

		save_interrupt(closure);
	}

	pub fn insert_io_interface(
        &mut self,
        io_interface: Arc<dyn Interface>,
	) -> Result<Descriptor, Error> {
		let new_fd = || -> Result<Descriptor, Error> {
			let mut fd: Descriptor = 0;
			loop {
				if !self.current.borrow().fd_map.contains_key(&fd) {
					break Ok(fd);
				} else if fd == Descriptor::MAX {
					break Err(Error::ValueOverflow);
				}

				fd = fd.saturating_add(1);
			}
		};

		let fd = new_fd()?;
		self.current
			.borrow_mut()
			.fd_map
			.insert(fd, io_interface.clone());

		Ok(fd)
	}

	pub fn remove_io_interface(&self, fd: Descriptor) -> Result<Arc<dyn Interface>, Error> {
		self.current
			.borrow_mut()
			.fd_map
			.remove(&fd)
			.ok_or(Error::BadFileDescriptor)
	}

	pub fn get_io_interface(
		&self,
		fd: Descriptor,
	) -> Result<Arc<dyn Interface>, Error> {
		let closure = || {
			if let Some(io_interface) = self.current.borrow().fd_map.get(&fd) {
				Ok(io_interface.clone())
			} else {
				Err(Error::FileNotFound)
			}
		};

		save_interrupt(closure)
	}

	pub fn get_current_taskid(&self) -> TaskId {
		save_interrupt(|| self.current.borrow().id)
	}

	#[no_mangle]
	pub fn get_current_interrupt_stack(&self) -> VirtualAddress {
		save_interrupt(|| (*self.current.borrow().stack).interrupt_top())
	}

	pub fn get_root_page_table(&self) -> PhysicalAddress {
		self.current.borrow().root_page_table
	}

	pub fn set_root_page_table(&self, addr: PhysicalAddress) {
		self.current.borrow_mut().root_page_table = addr;
	}

	pub fn schedule(&mut self) {
		if let Some(id) = self.finished.pop_front() {
			if self.tasks.remove(&id).is_none() {
				info!("unable to drop task {}.", id);
			}
		}

		let (current_id, current_stack_pointer, current_priority, current_status) = {
			let mut borrowed = self.current.borrow_mut();
			(
				borrowed.id,
				&mut borrowed.last_stack_pointer as *mut VirtualAddress,
				borrowed.priority,
				borrowed.status,
			)
		};

		let mut next_task;
		if current_status == TaskStatus::Running {
			next_task = self.ready.pop_with_priority(current_priority);
		} else {
			next_task = self.ready.pop();
		}

		if next_task.is_none()
			&& current_status != TaskStatus::Running
			&& current_status != TaskStatus::Idle
		{
			debug!("switch to idle task.");
			next_task = Some(self.idle.clone());
		}

		if let Some(new_task) = next_task {
			let (new_id, new_stack_pointer) = {
				let mut borrowed = new_task.borrow_mut();
				borrowed.status = TaskStatus::Running;
				(borrowed.id, borrowed.last_stack_pointer)
			};

			if current_status == TaskStatus::Running {
				debug!("add task {} to ready queue.", current_id);
				self.current.borrow_mut().status = TaskStatus::Ready;
				self.ready.push(self.current.clone());
			} else if current_status == TaskStatus::Finished {
				debug!("task {} finished.", current_id);
				self.current.borrow_mut().status = TaskStatus::Invalid;
				self.finished.push_back(current_id);
			}

			debug!(
				"switching task from {} to {} (stack {:#X} => {:#X}).",
				current_id,
				new_id,
				unsafe { *current_stack_pointer },
				new_stack_pointer
			);

			self.current = new_task;

			unsafe {
				switch::perform_context_switch(current_stack_pointer, new_stack_pointer);
			}
		}
	}

	pub fn reschedule(&mut self) {
		save_interrupt(|| self.schedule());
	}
}
