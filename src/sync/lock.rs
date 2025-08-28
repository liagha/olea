use {
	crate::arch::kernel::{
		processor::utilities::pause,
		interrupts::{interrupt_nested_disable, interrupt_nested_enable},
	},
	core::{
		cell::UnsafeCell,
		marker::{PhantomData, Send, Sync},
		ops::{Deref, DerefMut},
		sync::atomic::{AtomicBool, AtomicUsize, Ordering},
	},
};

#[derive(Debug)]
pub struct RawWaitLock {
	queue: AtomicUsize,
	dequeue: AtomicUsize,
}

impl Default for RawWaitLock {
    fn default() -> Self {
        Self::new()
    }
}

impl RawWaitLock {
	pub const fn new() -> Self {
		Self {
			queue: AtomicUsize::new(0),
			dequeue: AtomicUsize::new(0),
		}
	}

	#[inline]
	pub fn lock(&self) {
		let ticket = self.queue.fetch_add(1, Ordering::Acquire);
		while self.dequeue.load(Ordering::Acquire) != ticket {
			pause();
		}
	}

	#[inline]
	pub fn try_lock(&self) -> bool {
		let current = self.dequeue.load(Ordering::Acquire);
		self.queue
			.compare_exchange_weak(current, current + 1, Ordering::Acquire, Ordering::Relaxed)
			.is_ok()
	}

	#[inline]
	pub fn unlock(&self) {
		self.dequeue.fetch_add(1, Ordering::Release);
	}

	#[inline]
	pub fn is_locked(&self) -> bool {
		let queue = self.queue.load(Ordering::Acquire);
		let dequeue = self.dequeue.load(Ordering::Acquire);
		queue != dequeue
	}
}

#[derive(Debug)]
pub struct WaitLock<T> {
	lock: RawWaitLock,
	data: UnsafeCell<T>,
}

impl<T> WaitLock<T> {
	pub const fn new(data: T) -> Self {
		Self {
			lock: RawWaitLock::new(),
			data: UnsafeCell::new(data),
		}
	}

	pub fn lock(&self) -> WaitLockGuard<'_, T> {
		self.lock.lock();
		WaitLockGuard {
			lock: &self.lock,
			data: &self.data,
			_phantom: PhantomData,
		}
	}

	pub fn try_lock(&self) -> Option<WaitLockGuard<'_, T>> {
		if self.lock.try_lock() {
			Some(WaitLockGuard {
				lock: &self.lock,
				data: &self.data,
				_phantom: PhantomData,
			})
		} else {
			None
		}
	}

	pub fn into_inner(self) -> T {
		self.data.into_inner()
	}
}

unsafe impl<T: Send> Sync for WaitLock<T> {}
unsafe impl<T: Send> Send for WaitLock<T> {}

pub struct WaitLockGuard<'a, T> {
	lock: &'a RawWaitLock,
	data: &'a UnsafeCell<T>,
	_phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Deref for WaitLockGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.data.get() }
	}
}

impl<'a, T> DerefMut for WaitLockGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *self.data.get() }
	}
}

impl<'a, T> Drop for WaitLockGuard<'a, T> {
	fn drop(&mut self) {
		self.lock.unlock();
	}
}

pub struct RawWaitLockIrqSave {
	queue: AtomicUsize,
	dequeue: AtomicUsize,
	irq_state: AtomicBool,
}

impl Default for RawWaitLockIrqSave {
    fn default() -> Self {
        Self::new()
    }
}

impl RawWaitLockIrqSave {
	pub const fn new() -> Self {
		Self {
			queue: AtomicUsize::new(0),
			dequeue: AtomicUsize::new(0),
			irq_state: AtomicBool::new(false),
		}
	}

	#[inline]
	pub fn lock(&self) -> bool {
		let irq_was_enabled = interrupt_nested_disable();
		let ticket = self.queue.fetch_add(1, Ordering::Acquire);

		while self.dequeue.load(Ordering::Acquire) != ticket {
			interrupt_nested_enable(irq_was_enabled);
			pause();
			interrupt_nested_disable();
		}

		self.irq_state.store(irq_was_enabled, Ordering::Release);
		irq_was_enabled
	}

	#[inline]
	pub fn try_lock(&self) -> Option<bool> {
		let irq_was_enabled = interrupt_nested_disable();
		let current = self.dequeue.load(Ordering::Acquire);

		if self.queue
			.compare_exchange_weak(current, current + 1, Ordering::Acquire, Ordering::Relaxed)
			.is_ok()
		{
			self.irq_state.store(irq_was_enabled, Ordering::Release);
			Some(irq_was_enabled)
		} else {
			interrupt_nested_enable(irq_was_enabled);
			None
		}
	}

	#[inline]
	pub fn unlock(&self, irq_was_enabled: bool) {
		self.dequeue.fetch_add(1, Ordering::Release);
		interrupt_nested_enable(irq_was_enabled);
	}

	#[inline]
	pub fn is_locked(&self) -> bool {
		let queue = self.queue.load(Ordering::Acquire);
		let dequeue = self.dequeue.load(Ordering::Acquire);
		queue != dequeue
	}
}

pub struct WaitLockIrqSave<T> {
	lock: RawWaitLockIrqSave,
	data: UnsafeCell<T>,
}

impl<T> WaitLockIrqSave<T> {
	pub const fn new(data: T) -> Self {
		Self {
			lock: RawWaitLockIrqSave::new(),
			data: UnsafeCell::new(data),
		}
	}

	pub fn lock(&self) -> WaitLockIrqSaveGuard<'_, T> {
		let irq_was_enabled = self.lock.lock();
		WaitLockIrqSaveGuard {
			lock: &self.lock,
			data: &self.data,
			irq_was_enabled,
			_phantom: PhantomData,
		}
	}

	pub fn try_lock(&self) -> Option<WaitLockIrqSaveGuard<'_, T>> {
		self.lock.try_lock().map(|irq_was_enabled| WaitLockIrqSaveGuard {
				lock: &self.lock,
				data: &self.data,
				irq_was_enabled,
				_phantom: PhantomData,
			})
	}

	pub fn into_inner(self) -> T {
		self.data.into_inner()
	}
}

unsafe impl<T: Send> Sync for WaitLockIrqSave<T> {}
unsafe impl<T: Send> Send for WaitLockIrqSave<T> {}

pub struct WaitLockIrqSaveGuard<'a, T> {
	lock: &'a RawWaitLockIrqSave,
	data: &'a UnsafeCell<T>,
	irq_was_enabled: bool,
	_phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Deref for WaitLockIrqSaveGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.data.get() }
	}
}

impl<'a, T> DerefMut for WaitLockIrqSaveGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *self.data.get() }
	}
}

impl<'a, T> Drop for WaitLockIrqSaveGuard<'a, T> {
	fn drop(&mut self) {
		self.lock.unlock(self.irq_was_enabled);
	}
}

const READER_MASK: usize = (1 << 30) - 1;
const WRITER_BIT: usize = 1 << 31;
const WRITER_WAITING_BIT: usize = 1 << 30;

#[derive(Debug)]
pub struct RawSharedWaitLock {
	state: AtomicUsize,
}

impl Default for RawSharedWaitLock {
    fn default() -> Self {
        Self::new()
    }
}

impl RawSharedWaitLock {
	pub const fn new() -> Self {
		Self {
			state: AtomicUsize::new(0),
		}
	}

	#[inline]
	pub fn read_lock(&self) {
		loop {
			let state = self.state.load(Ordering::Acquire);

			if (state & (WRITER_BIT | WRITER_WAITING_BIT)) != 0 {
				pause();
				continue;
			}

			let new_state = state + 1;
			if new_state & READER_MASK == 0 {
				pause();
				continue;
			}

			if self.state.compare_exchange_weak(
				state,
				new_state,
				Ordering::Acquire,
				Ordering::Relaxed,
			).is_ok() {
				break;
			}

			pause();
		}
	}

	#[inline]
	pub fn try_read_lock(&self) -> bool {
		let state = self.state.load(Ordering::Acquire);

		if (state & (WRITER_BIT | WRITER_WAITING_BIT)) != 0 {
			return false;
		}

		let new_state = state + 1;
		if new_state & READER_MASK == 0 {
			return false;
		}

		self.state.compare_exchange_weak(
			state,
			new_state,
			Ordering::Acquire,
			Ordering::Relaxed,
		).is_ok()
	}

	#[inline]
	pub fn read_unlock(&self) {
		self.state.fetch_sub(1, Ordering::Release);
	}

	#[inline]
	pub fn write_lock(&self) {
		loop {
			let state = self.state.load(Ordering::Acquire);
			if (state & WRITER_WAITING_BIT) == 0 && self.state.compare_exchange_weak(
					state,
					state | WRITER_WAITING_BIT,
					Ordering::Acquire,
					Ordering::Relaxed,
				).is_ok() {
   					break;
   				}
			pause();
		}

		loop {
			let state = self.state.load(Ordering::Acquire);
			if (state & READER_MASK) == 0 && (state & WRITER_BIT) == 0 && self.state.compare_exchange_weak(
					state,
					WRITER_BIT,
					Ordering::Acquire,
					Ordering::Relaxed,
				).is_ok() {
   					break;
   				}
			pause();
		}
	}

	#[inline]
	pub fn try_write_lock(&self) -> bool {
		let state = self.state.load(Ordering::Acquire);

		if state != 0 {
			return false;
		}

		self.state.compare_exchange_weak(
			0,
			WRITER_BIT,
			Ordering::Acquire,
			Ordering::Relaxed,
		).is_ok()
	}

	#[inline]
	pub fn write_unlock(&self) {
		self.state.store(0, Ordering::Release);
	}

	#[inline]
	pub fn is_locked(&self) -> bool {
		self.state.load(Ordering::Acquire) != 0
	}
}

#[derive(Debug)]
pub struct SharedWaitLock<T> {
	lock: RawSharedWaitLock,
	data: UnsafeCell<T>,
}

impl<T> SharedWaitLock<T> {
	pub const fn new(data: T) -> Self {
		Self {
			lock: RawSharedWaitLock::new(),
			data: UnsafeCell::new(data),
		}
	}

	pub fn read(&self) -> SharedWaitLockReadGuard<'_, T> {
		self.lock.read_lock();
		SharedWaitLockReadGuard {
			lock: &self.lock,
			data: &self.data,
			_phantom: PhantomData,
		}
	}

	pub fn try_read(&self) -> Option<SharedWaitLockReadGuard<'_, T>> {
		if self.lock.try_read_lock() {
			Some(SharedWaitLockReadGuard {
				lock: &self.lock,
				data: &self.data,
				_phantom: PhantomData,
			})
		} else {
			None
		}
	}

	pub fn write(&self) -> SharedWaitLockWriteGuard<'_, T> {
		self.lock.write_lock();
		SharedWaitLockWriteGuard {
			lock: &self.lock,
			data: &self.data,
			_phantom: PhantomData,
		}
	}

	pub fn try_write(&self) -> Option<SharedWaitLockWriteGuard<'_, T>> {
		if self.lock.try_write_lock() {
			Some(SharedWaitLockWriteGuard {
				lock: &self.lock,
				data: &self.data,
				_phantom: PhantomData,
			})
		} else {
			None
		}
	}

	pub fn into_inner(self) -> T {
		self.data.into_inner()
	}
}

unsafe impl<T: Send> Sync for SharedWaitLock<T> {}
unsafe impl<T: Send> Send for SharedWaitLock<T> {}

pub struct SharedWaitLockReadGuard<'a, T> {
	lock: &'a RawSharedWaitLock,
	data: &'a UnsafeCell<T>,
	_phantom: PhantomData<&'a T>,
}

impl<'a, T> Deref for SharedWaitLockReadGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.data.get() }
	}
}

impl<'a, T> Drop for SharedWaitLockReadGuard<'a, T> {
	fn drop(&mut self) {
		self.lock.read_unlock();
	}
}

pub struct SharedWaitLockWriteGuard<'a, T> {
	lock: &'a RawSharedWaitLock,
	data: &'a UnsafeCell<T>,
	_phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Deref for SharedWaitLockWriteGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.data.get() }
	}
}

impl<'a, T> DerefMut for SharedWaitLockWriteGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *self.data.get() }
	}
}

impl<'a, T> Drop for SharedWaitLockWriteGuard<'a, T> {
	fn drop(&mut self) {
		self.lock.write_unlock();
	}
}