mod scheduler;
pub mod task;

use {
	crate::{
		io::Error,
		scheduler::task::{Task, TaskPriority},
		file::descriptor::{Descriptor, IoInterface},
		arch::{
			memory::{PhysAddr, VirtAddr},
			kernel::{
				register_task,
				interrupts::{interrupt_nested_disable, interrupt_nested_enable},
			},
		},
	},
	core::cell::RefCell,
	alloc::{
		rc::Rc,
		sync::Arc,
	},
};

static mut SCHEDULER: Option<scheduler::Scheduler> = None;

pub fn init() {
	unsafe {
		SCHEDULER = Some(scheduler::Scheduler::new());
	}

	register_task();
}

pub fn spawn(func: extern "C" fn(), priority: TaskPriority) -> Result<task::TaskId, Error> {
	unsafe { SCHEDULER.as_mut().unwrap().spawn(func, priority) }
}

pub fn reschedule() {
	unsafe { SCHEDULER.as_mut().unwrap().reschedule() }
}

pub fn schedule() {
	unsafe { SCHEDULER.as_mut().unwrap().schedule() }
}

pub fn do_exit() -> ! {
	unsafe {
		SCHEDULER.as_mut().unwrap().exit();
	}
}

pub fn abort() -> ! {
	unsafe { SCHEDULER.as_mut().unwrap().abort() }
}

pub fn get_current_interrupt_stack() -> VirtAddr {
	unsafe { SCHEDULER.as_mut().unwrap().get_current_interrupt_stack() }
}

pub fn get_root_page_table() -> PhysAddr {
	unsafe { SCHEDULER.as_mut().unwrap().get_root_page_table() }
}

pub fn set_root_page_table(addr: PhysAddr) {
	unsafe {
		SCHEDULER.as_mut().unwrap().set_root_page_table(addr);
	}
}

pub fn block_current_task() -> Rc<RefCell<Task>> {
	unsafe { SCHEDULER.as_mut().unwrap().block_current_task() }
}

pub fn wakeup_task(task: Rc<RefCell<Task>>) {
	unsafe { SCHEDULER.as_mut().unwrap().wakeup_task(task) }
}

pub fn get_io_interface(fd: Descriptor) -> Result<Arc<dyn IoInterface>, Error> {
	let _preemption = DisabledPreemption::new();

	unsafe { SCHEDULER.as_mut().unwrap().get_io_interface(fd) }
}

pub fn insert_io_interface(obj: Arc<dyn IoInterface>) -> Result<Descriptor, Error> {
	let _preemption = DisabledPreemption::new();

	unsafe { SCHEDULER.as_mut().unwrap().insert_io_interface(obj) }
}

pub fn remove_io_interface(fd: Descriptor) -> Result<Arc<dyn IoInterface>, Error> {
	let _preemption = DisabledPreemption::new();

	unsafe { SCHEDULER.as_mut().unwrap().remove_io_interface(fd) }
}

pub fn get_current_taskid() -> task::TaskId {
	unsafe { SCHEDULER.as_ref().unwrap().get_current_taskid() }
}

pub struct DisabledPreemption {
	irq_enabled: bool,
}

impl DisabledPreemption {
	pub fn new() -> Self {
		DisabledPreemption {
			irq_enabled: interrupt_nested_disable(),
		}
	}
}

impl Drop for DisabledPreemption {
	fn drop(&mut self) {
		interrupt_nested_enable(self.irq_enabled);
	}
}
