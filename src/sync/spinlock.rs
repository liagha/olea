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
pub struct RawSpinlock {
	queue: AtomicUsize,
	dequeue: AtomicUsize,
}

impl Default for RawSpinlock {
    fn default() -> Self {
        Self::new()
    }
}

impl RawSpinlock {
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
pub struct Spinlock<T> {
	lock: RawSpinlock,
	data: UnsafeCell<T>,
}

impl<T> Spinlock<T> {
	pub const fn new(data: T) -> Self {
		Self {
			lock: RawSpinlock::new(),
			data: UnsafeCell::new(data),
		}
	}

	pub fn lock(&self) -> SpinlockGuard<'_, T> {
		self.lock.lock();
		SpinlockGuard {
			lock: &self.lock,
			data: &self.data,
			_phantom: PhantomData,
		}
	}

	pub fn try_lock(&self) -> Option<SpinlockGuard<'_, T>> {
		if self.lock.try_lock() {
			Some(SpinlockGuard {
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

unsafe impl<T: Send> Sync for Spinlock<T> {}
unsafe impl<T: Send> Send for Spinlock<T> {}

pub struct SpinlockGuard<'a, T> {
	lock: &'a RawSpinlock,
	data: &'a UnsafeCell<T>,
	_phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Deref for SpinlockGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.data.get() }
	}
}

impl<'a, T> DerefMut for SpinlockGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *self.data.get() }
	}
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
	fn drop(&mut self) {
		self.lock.unlock();
	}
}

pub struct RawSpinlockIrqSave {
	queue: AtomicUsize,
	dequeue: AtomicUsize,
	irq_state: AtomicBool,
}

impl Default for RawSpinlockIrqSave {
    fn default() -> Self {
        Self::new()
    }
}

impl RawSpinlockIrqSave {
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

pub struct SpinlockIrqSave<T> {
	lock: RawSpinlockIrqSave,
	data: UnsafeCell<T>,
}

impl<T> SpinlockIrqSave<T> {
	pub const fn new(data: T) -> Self {
		Self {
			lock: RawSpinlockIrqSave::new(),
			data: UnsafeCell::new(data),
		}
	}

	pub fn lock(&self) -> SpinlockIrqSaveGuard<'_, T> {
		let irq_was_enabled = self.lock.lock();
		SpinlockIrqSaveGuard {
			lock: &self.lock,
			data: &self.data,
			irq_was_enabled,
			_phantom: PhantomData,
		}
	}

	pub fn try_lock(&self) -> Option<SpinlockIrqSaveGuard<'_, T>> {
		self.lock.try_lock().map(|irq_was_enabled| SpinlockIrqSaveGuard {
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

unsafe impl<T: Send> Sync for SpinlockIrqSave<T> {}
unsafe impl<T: Send> Send for SpinlockIrqSave<T> {}

pub struct SpinlockIrqSaveGuard<'a, T> {
	lock: &'a RawSpinlockIrqSave,
	data: &'a UnsafeCell<T>,
	irq_was_enabled: bool,
	_phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Deref for SpinlockIrqSaveGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.data.get() }
	}
}

impl<'a, T> DerefMut for SpinlockIrqSaveGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *self.data.get() }
	}
}

impl<'a, T> Drop for SpinlockIrqSaveGuard<'a, T> {
	fn drop(&mut self) {
		self.lock.unlock(self.irq_was_enabled);
	}
}

const READER_MASK: usize = (1 << 30) - 1;
const WRITER_BIT: usize = 1 << 31;
const WRITER_WAITING_BIT: usize = 1 << 30;

#[derive(Debug)]
pub struct RawRwSpinlock {
	state: AtomicUsize,
}

impl Default for RawRwSpinlock {
    fn default() -> Self {
        Self::new()
    }
}

impl RawRwSpinlock {
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

			// Try to increment reader count
			let new_state = state + 1;
			if new_state & READER_MASK == 0 {
				// Reader count would overflow, spin
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

		// If writer is active or waiting, fail
		if (state & (WRITER_BIT | WRITER_WAITING_BIT)) != 0 {
			return false;
		}

		// Try to increment reader count
		let new_state = state + 1;
		if new_state & READER_MASK == 0 {
			// Reader count would overflow
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
		// Set writer waiting bit
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

		// Wait for all readers to finish and acquire write lock
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

		// Check if we can acquire write lock immediately
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
pub struct RwSpinlock<T> {
	lock: RawRwSpinlock,
	data: UnsafeCell<T>,
}

impl<T> RwSpinlock<T> {
	pub const fn new(data: T) -> Self {
		Self {
			lock: RawRwSpinlock::new(),
			data: UnsafeCell::new(data),
		}
	}

	pub fn read(&self) -> RwSpinlockReadGuard<'_, T> {
		self.lock.read_lock();
		RwSpinlockReadGuard {
			lock: &self.lock,
			data: &self.data,
			_phantom: PhantomData,
		}
	}

	pub fn try_read(&self) -> Option<RwSpinlockReadGuard<'_, T>> {
		if self.lock.try_read_lock() {
			Some(RwSpinlockReadGuard {
				lock: &self.lock,
				data: &self.data,
				_phantom: PhantomData,
			})
		} else {
			None
		}
	}

	pub fn write(&self) -> RwSpinlockWriteGuard<'_, T> {
		self.lock.write_lock();
		RwSpinlockWriteGuard {
			lock: &self.lock,
			data: &self.data,
			_phantom: PhantomData,
		}
	}

	pub fn try_write(&self) -> Option<RwSpinlockWriteGuard<'_, T>> {
		if self.lock.try_write_lock() {
			Some(RwSpinlockWriteGuard {
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

unsafe impl<T: Send> Sync for RwSpinlock<T> {}
unsafe impl<T: Send> Send for RwSpinlock<T> {}

pub struct RwSpinlockReadGuard<'a, T> {
	lock: &'a RawRwSpinlock,
	data: &'a UnsafeCell<T>,
	_phantom: PhantomData<&'a T>,
}

impl<'a, T> Deref for RwSpinlockReadGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.data.get() }
	}
}

impl<'a, T> Drop for RwSpinlockReadGuard<'a, T> {
	fn drop(&mut self) {
		self.lock.read_unlock();
	}
}

pub struct RwSpinlockWriteGuard<'a, T> {
	lock: &'a RawRwSpinlock,
	data: &'a UnsafeCell<T>,
	_phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Deref for RwSpinlockWriteGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.data.get() }
	}
}

impl<'a, T> DerefMut for RwSpinlockWriteGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *self.data.get() }
	}
}

impl<'a, T> Drop for RwSpinlockWriteGuard<'a, T> {
	fn drop(&mut self) {
		self.lock.write_unlock();
	}
}