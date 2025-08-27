use {
	crate::{
		scheduler::{
			block_current_task, reschedule, wakeup_task,
			task::{PriorityTaskQueue},
		},
		sync::lock::WaitLockIrqSave,
	},
	core::{
		cell::UnsafeCell,
		marker::{PhantomData, Send, Sync},
		ops::{Deref, DerefMut},
	},
};

pub struct Mutex<T> {
	is_locked: WaitLockIrqSave<bool>,
	wait_queue: WaitLockIrqSave<PriorityTaskQueue>,
	data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
	pub const fn new(data: T) -> Self {
		Self {
			is_locked: WaitLockIrqSave::new(false),
			wait_queue: WaitLockIrqSave::new(PriorityTaskQueue::new()),
			data: UnsafeCell::new(data),
		}
	}

	pub fn lock(&self) -> MutexGuard<'_, T> {
		self.acquire_lock();
		MutexGuard {
			mutex: self,
			_phantom: PhantomData,
		}
	}

	pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
		if self.try_acquire_lock() {
			Some(MutexGuard {
				mutex: self,
				_phantom: PhantomData,
			})
		} else {
			None
		}
	}

	pub fn into_inner(self) -> T {
		self.data.into_inner()
	}

	fn acquire_lock(&self) {
		loop {
			{
				let mut locked = self.is_locked.lock();
				if !*locked {
					*locked = true;
					return;
				}

				let task = block_current_task();
				self.wait_queue.lock().push(task);
			}

			reschedule();
		}
	}

	fn try_acquire_lock(&self) -> bool {
		let mut locked = self.is_locked.lock();
		if !*locked {
			*locked = true;
			true
		} else {
			false
		}
	}

	fn release_lock(&self) {
		let mut locked = self.is_locked.lock();
		*locked = false;

		if let Some(task) = self.wait_queue.lock().pop() {
			wakeup_task(task);
		}
	}
}

impl<T: Default> Default for Mutex<T> {
	fn default() -> Self {
		Self::new(T::default())
	}
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

pub struct MutexGuard<'a, T> {
	mutex: &'a Mutex<T>,
	_phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.mutex.data.get() }
	}
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *self.mutex.data.get() }
	}
}

impl<'a, T> Drop for MutexGuard<'a, T> {
	fn drop(&mut self) {
		self.mutex.release_lock();
	}
}