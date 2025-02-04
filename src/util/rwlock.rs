//! TODO: Doc comments


use core::cell::UnsafeCell;
use core::sync::atomic::{ AtomicU32, Ordering };
use core::ops::{ Deref, DerefMut };
use core::pin::Pin;
use core::task::{ Context, Poll };
use core::mem::ManuallyDrop;
use alloc::sync::Arc;


/// A [`RwLock`] which gives up fairness for speed.
pub struct RwLock<T> {

    /// The inner data and lock of this [`RwLock`].
    inner : Arc<RwLockInner<T>>

}

/// Private types.
mod __internal {
    use super::*;

    /// The inner data and lock of a [`RwLock`].
    pub struct RwLockInner<T> {

        /// The inner data.
        pub(super) value          : UnsafeCell<T>,

        /// The lock state.
        /// 
        /// [`u32::MAX`] means the [`RwLock`] is write-locked.
        /// Anything below is the number of read locks.
        pub(super) state          : AtomicU32,

        /// The number of write-locks that are waiting.
        pub(super) waiting_writes : AtomicU32

    }

}
use __internal::*;

impl<T> RwLock<T> {

    /// Create a new [`RwLock`] with a given value contained.
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

impl<T> RwLock<T> {

    /// Returns a [`Future`] which will eventually take ownership of the inner data.
    /// 
    /// The [`RwLock`] will be permanently locked.
    pub fn into_inner(self) -> PendingRwLockOwn<T> {
        PendingRwLockOwn { lock : Some(self.inner) }
    }

    /// Tries to take ownership of the inner data.
    /// 
    /// The [`RwLock`] will be permanently locked if this succeeds.
    pub fn try_into_inner(self) -> Poll<T> {
        self.inner.state.compare_exchange(0, u32::MAX, Ordering::Acquire, Ordering::Relaxed).is_ok()
            // SAFETY: Lock state was 0 (no locks), and is now `u32::MAX` (write-locked).
            //         The lock is never cleared, preventing access to the inner data again.
            .then(|| unsafe{ Arc::into_inner(self.inner).unwrap_unchecked() }.value.into_inner() )
            .map_or(Poll::Pending, |out| Poll::Ready(out))
    }

}

impl<T> RwLockInner<T> {

    /// Tries to obtain a read-lock to the inner data.
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

    /// Returns a [`Future`] which will eventually obtain a read-lock to the inner data.
    pub fn read(self : &Arc<Self>) -> PendingRwLockRead<T> {
        PendingRwLockRead { lock : self.clone() }
    }

    /// Tries to obtain a write-lock to the inner data.
    pub fn try_write(self : &Arc<Self>) -> Poll<RwLockWriteGuard<T>> {
        self.state.compare_exchange(0, u32::MAX, Ordering::Acquire, Ordering::Relaxed).is_ok()
            .then(|| RwLockWriteGuard { lock : ManuallyDrop::new(self.clone()) })
            .map_or(Poll::Pending, |out| Poll::Ready(out))
    }

    /// Returns a [`Future`] which will eventually obtain a write-lock to the inner data.
    pub fn write(self : &Arc<Self>) -> PendingRwLockWrite<T> {
        let _ = self.waiting_writes.fetch_add(1, Ordering::Relaxed);
        PendingRwLockWrite { lock : self.clone() }
    }

    /// Returns a write-lock to this [`RwLock`], without checking state or locking.
    ///
    /// # Safety
    /// The caller is responsible for ensuring the [`RwLock`] is write-locked, but has no guards to it.
    /// See [`RwLock::new_writing`].
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



/// TODO: Doc comments
pub struct PendingRwLockOwn<T> {

    /// TODO: Doc comments
    lock : Option<Arc<RwLockInner<T>>>

}

impl<T> Unpin for PendingRwLockOwn<T> { }

impl<T> PendingRwLockOwn<T> {

    /// TODO: Doc comments
    pub fn try_into_inner(&mut self) -> Poll<T> {
        self.lock.as_ref().unwrap().state.compare_exchange(1, u32::MAX, Ordering::Acquire, Ordering::Relaxed).is_ok()
            // SAFETY: TODO
            .then(|| unsafe{ Arc::into_inner(self.lock.take().unwrap_unchecked()).unwrap_unchecked() }.value.into_inner() )
            .map_or(Poll::Pending, |out| Poll::Ready(out))
    }

}

impl<T> Future for PendingRwLockOwn<T> {
    type Output = T;

    fn poll(mut self : Pin<&mut Self>, _ctx : &mut Context<'_>) -> Poll<Self::Output> {
        self.try_into_inner()
    }
}
