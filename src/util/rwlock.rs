//! TODO: Doc comments


use core::cell::UnsafeCell;
use core::sync::atomic::{ AtomicU32, Ordering };
use core::ops::{ Deref, DerefMut };
use core::pin::Pin;
use core::task::{ Context, Poll };
use core::mem::ManuallyDrop;
use alloc::sync::Arc;


/// TODO: Doc comments
pub struct RwLock<T> {

    /// TODO: Doc comments
    inner : Arc<RwLockInner<T>>

}

/// TODO: Doc comments
mod __internal {
    use super::*;

    /// TODO: Doc comments
    pub struct RwLockInner<T> {

        /// TODO: Doc comments
        pub(super) value          : UnsafeCell<T>,

        /// TODO: Doc comments
        pub(super) state          : AtomicU32,

        /// TODO: Doc comments
        pub(super) waiting_writes : AtomicU32

    }

}
use __internal::*;

impl<T> RwLock<T> {

    /// TODO: Doc comments
    pub fn new(value : T) -> Self { Self {
        inner : Arc::new(RwLockInner {
            value          : UnsafeCell::new(value),
            state          : AtomicU32::new(0),
            waiting_writes : AtomicU32::new(0)
        })
    } }

    /// Creates a new [`RwLock`], already write-locked.
    ///
    /// # Safety
    /// The caller is responsible for obtaining a [`RwLockWriteGuard`] through `[RwLockInner::write_unchecked]`, and eventually dropping it.
    pub(crate) unsafe fn new_writing(value : T) -> Self { Self {
        inner : Arc::new(RwLockInner {
            value          : UnsafeCell::new(value),
            state          : AtomicU32::new(u32::MAX),
            waiting_writes : AtomicU32::new(0)
        })
    } }

}

impl<T> Deref for RwLock<T> {
    type Target = Arc<RwLockInner<T>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> RwLockInner<T> {

    /// TODO: Doc comments
    pub fn try_read(self : &Arc<Self>) -> Poll<RwLockReadGuard<T>> {
        if (self.waiting_writes.load(Ordering::Relaxed) > 0) {
            return Poll::Pending;
        }
        let s = self.state.load(Ordering::Relaxed);
        if (s < u32::MAX) {
            assert!(s < (u32::MAX - 1), "maximum reader count exceeded");
            self.state.compare_exchange_weak(s, s + 1, Ordering::Acquire, Ordering::Relaxed).is_ok()
                .then(|| RwLockReadGuard { lock : ManuallyDrop::new(self.clone()) })
                .map_or(Poll::Pending, |out| Poll::Ready(out))
        } else { Poll::Pending }
    }

    /// TODO: Doc comments
    pub fn read(self : &Arc<Self>) -> PendingRwLockRead<T> {
        PendingRwLockRead { lock : self.clone() }
    }

    /// TODO: Doc comments
    pub fn try_write(self : &Arc<Self>) -> Poll<RwLockWriteGuard<T>> {
        self.state.compare_exchange(0, u32::MAX, Ordering::Acquire, Ordering::Relaxed).is_ok()
            .then(|| RwLockWriteGuard { lock : ManuallyDrop::new(self.clone()) })
            .map_or(Poll::Pending, |out| Poll::Ready(out))
    }

    /// TODO: Doc comments
    pub fn write(self : &Arc<Self>) -> PendingRwLockWrite<T> {
        let _ = self.waiting_writes.fetch_add(1, Ordering::Relaxed);
        PendingRwLockWrite { lock : self.clone() }
    }

    /// Returns a write-lock to this [`RwLock`], without checking state or locking.
    ///
    /// # Safety
    /// The caller is responsible for ensuring the [`RwLock`] is write-locked, but has no locks to it. See [`RwLock::new_writing`].
    pub(crate) fn write_unchecked(self : &Arc<Self>) -> RwLockWriteGuard<T> {
        RwLockWriteGuard { lock : ManuallyDrop::new(self.clone()) }
    }

}



/// TODO: Doc comments
pub struct PendingRwLockRead<T> {

    /// TODO: Doc comments
    lock : Arc<RwLockInner<T>>

}

impl<T> PendingRwLockRead<T> {

    /// TODO: Doc comments
    pub fn try_read(&self) -> Poll<RwLockReadGuard<T>> {
        self.lock.try_read()
    }

}

impl<T> Future for PendingRwLockRead<T> {
    type Output = RwLockReadGuard<T>;

    fn poll(self : Pin<&mut Self>, _ctx : &mut Context<'_>) -> Poll<Self::Output> {
        self.lock.try_read()
    }
}



/// TODO: Doc comments
pub struct RwLockReadGuard<T> {

    /// TODO: Doc comments
    lock : ManuallyDrop<Arc<RwLockInner<T>>>

}

impl<T> RwLockReadGuard<T> {

    /// TODO: Doc comments
    pub fn try_upgrade(guard : Self) -> Poll<RwLockWriteGuard<T>> {
        let mut guard = ManuallyDrop::new(guard);
        guard.lock.state.compare_exchange(1, u32::MAX, Ordering::Acquire, Ordering::Relaxed).is_ok()
            // SAFETY: TODO
            .then(|| RwLockWriteGuard { lock : ManuallyDrop::new(unsafe{ ManuallyDrop::take(&mut guard.lock) }) })
            .map_or(Poll::Pending, |out| Poll::Ready(out))
    }

    /// TODO: Doc comments
    pub fn upgrade(guard : Self) -> PendingRwLockUpgrade<T> {
        let mut guard = ManuallyDrop::new(guard);
        let _ = guard.lock.waiting_writes.fetch_add(1, Ordering::Relaxed);
        // SAFETY: TODO
        PendingRwLockUpgrade { lock : unsafe{ ManuallyDrop::take(&mut guard.lock) } }
    }

}

impl<T> Deref for RwLockReadGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: TODO
        unsafe{ &*self.lock.value.get() }
    }
}

impl<T> Drop for RwLockReadGuard<T> {
    fn drop(&mut self) {
        let _ = self.lock.state.fetch_sub(1, Ordering::Release);
        // SAFETY: TODO
        unsafe{ ManuallyDrop::drop(&mut self.lock); }
    }
}



/// TODO: Doc comments
pub struct PendingRwLockWrite<T> {

    /// TODO: Doc comments
    lock : Arc<RwLockInner<T>>

}

impl<T> PendingRwLockWrite<T> {

    /// TODO: Doc comments
    pub fn try_write(&self) -> Poll<RwLockWriteGuard<T>> {
        self.lock.try_write()
    }

}

impl<T> Future for PendingRwLockWrite<T> {
    type Output = RwLockWriteGuard<T>;

    fn poll(self : Pin<&mut Self>, _ctx : &mut Context<'_>) -> Poll<Self::Output> {
        self.lock.try_write()
    }
}

impl<T> Drop for PendingRwLockWrite<T> {
    fn drop(&mut self) {
        let _ = self.lock.waiting_writes.fetch_sub(1, Ordering::Relaxed);
    }
}



/// TODO: Doc comments
pub struct PendingRwLockUpgrade<T> {

    /// TODO: Doc comments
    lock : Arc<RwLockInner<T>>

}

impl<T> PendingRwLockUpgrade<T> {

    /// TODO: Doc comments
    pub fn try_write(&self) -> Poll<RwLockWriteGuard<T>> {
        self.lock.state.compare_exchange(1, u32::MAX, Ordering::Acquire, Ordering::Relaxed).is_ok()
            .then(|| RwLockWriteGuard { lock : ManuallyDrop::new(self.lock.clone()) })
            .map_or(Poll::Pending, |out| Poll::Ready(out))
    }

}

impl<T> Future for PendingRwLockUpgrade<T> {
    type Output = RwLockWriteGuard<T>;

    fn poll(self : Pin<&mut Self>, _ctx : &mut Context<'_>) -> Poll<Self::Output> {
        self.try_write()
    }
}

impl<T> Drop for PendingRwLockUpgrade<T> {
    fn drop(&mut self) {
        let _ = self.lock.waiting_writes.fetch_sub(1, Ordering::Relaxed);
    }
}



/// TODO: Doc comments
pub struct RwLockWriteGuard<T> {

    /// TODO: Doc comments
    lock : ManuallyDrop<Arc<RwLockInner<T>>>

}

impl<T> RwLockWriteGuard<T> {

    /// TODO: Doc comments
    pub fn downgrade(guard : Self) -> RwLockReadGuard<T> {
        let mut guard = ManuallyDrop::new(guard);
        guard.lock.state.store(1, Ordering::Relaxed);
        // SAFETY: TODO
        RwLockReadGuard { lock : ManuallyDrop::new(unsafe{ ManuallyDrop::take(&mut guard.lock) }) }
    }
}

impl<T> Deref for RwLockWriteGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: TODO
        unsafe{ &*self.lock.value.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: TODO
        unsafe{ &mut *self.lock.value.get() }
    }
}

impl<T> Drop for RwLockWriteGuard<T> {
    fn drop(&mut self) {
        let _ = self.lock.state.store(0, Ordering::Release);
        // SAFETY: TODO
        unsafe{ ManuallyDrop::drop(&mut self.lock); }
    }
}
