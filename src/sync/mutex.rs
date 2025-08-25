use {
	crate::{
		sync::spinlock::*,
		scheduler::{
			task::*,
			block_current_task, reschedule, wakeup_task,
		},
	},
	core::{
		marker::Sync,
		cell::UnsafeCell,
		ops::{Deref, DerefMut, Drop},
	},
};

pub struct Mutex<T: ?Sized> {
	value: SpinlockIrqSave<bool>,
	queue: SpinlockIrqSave<PriorityTaskQueue>,
	data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
	value: &'a SpinlockIrqSave<bool>,
	queue: &'a SpinlockIrqSave<PriorityTaskQueue>,
	data: &'a mut T,
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
	pub fn new(user_data: T) -> Mutex<T> {
		Mutex {
			value: SpinlockIrqSave::new(true),
			queue: SpinlockIrqSave::new(PriorityTaskQueue::new()),
			data: UnsafeCell::new(user_data),
		}
	}

	pub fn into_inner(self) -> T {
		let Mutex { data, .. } = self;
		data.into_inner()
	}
}

impl<T: ?Sized> Mutex<T> {
	fn obtain_lock(&self) {
		loop {
			let mut count = self.value.lock();

			if *count {
				*count = false;
				return;
			} else {
				self.queue.lock().push(block_current_task());
				
				drop(count);
				
				reschedule();
			}
		}
	}

	pub fn lock(&self) -> MutexGuard<T> {
		self.obtain_lock();
		MutexGuard {
			value: &self.value,
			queue: &self.queue,
			data: unsafe { &mut *self.data.get() },
		}
	}
}

impl<T: Default> Default for Mutex<T> {
	fn default() -> Mutex<T> {
		Mutex::new(Default::default())
	}
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		&*self.data
	}
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		&mut *self.data
	}
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
	fn drop(&mut self) {
		let mut count = self.value.lock();
		*count = true;

		if let Some(task) = self.queue.lock().pop() {
			wakeup_task(task);
		}
	}
}
