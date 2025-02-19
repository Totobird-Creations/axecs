//! TODO: Doc comments


use crate::resource::Resource;
use crate::util::rwlock::{ RwLock, RwLockReadGuard, RwLockWriteGuard };
use core::ops::{ Deref, DerefMut };
use core::any::TypeId;
use core::task::Poll;
use core::alloc::Layout;
use core::ptr::NonNull;
use core::marker::PhantomData;
use alloc::alloc::{ alloc, dealloc, handle_alloc_error };
use alloc::collections::BTreeMap;


/// TODO: Doc comments
pub struct ResourceStorage {

    /// TODO: Doc comments
    raw : RwLock<RawResourceStorage>

}

/// TODO: Doc comments
pub struct RawResourceStorage {

    /// TODO: Doc comments
    resources : BTreeMap<TypeId, RwLock<ResourceCell>>

}


impl ResourceStorage {

    /// TODO: Doc comment
    pub fn new_with(raw : RawResourceStorage) -> Self { Self {
        raw : RwLock::new(raw)
    } }


    /// Attempts to acquire a read lock to the raw data, returning immediately if it can't.
    pub fn try_read_raw(&self) -> Poll<RwLockReadGuard<RawResourceStorage>> {
        self.raw.try_read()
    }

    /// Acquires a read lock to the raw data.
    pub async fn read_raw(&self) -> RwLockReadGuard<RawResourceStorage> {
        self.raw.read().await
    }

}

impl RawResourceStorage {

    /// Creates an empty [`RawResourceStorage`].
    pub fn new() -> Self { Self {
        resources : BTreeMap::new()
    } }

    /// Returns an [`Iterator`] over [`RwLock`] wrapped [`ResourceCell`]s.
    pub fn resources(&self) -> impl Iterator<Item = (TypeId, &RwLock<ResourceCell>)> {
        self.resources.iter().map(|(type_id, resource)| (*type_id, resource))
    }

    /// TODO: Doc comments
    pub fn insert<R : Resource + 'static>(&mut self, resource : R) -> bool {
        self.resources.try_insert(TypeId::of::<R>(), RwLock::new(ResourceCell::new(resource))).is_ok()
    }

}

impl ResourceStorage {

    /// Creates an empty [`ResourceStorage`].
    pub fn new() -> Self { Self {
        raw : RwLock::new(RawResourceStorage {
            resources : BTreeMap::new()
        })
    } }

    /// TODO: Doc comments
    pub async fn insert<R : Resource + 'static>(&self, resource : R) {
        self.raw.write().await.resources.insert(TypeId::of::<R>(), RwLock::new(ResourceCell::new(resource)));
    }

    /// TODO: Doc comments
    pub async fn replace<R : Resource + 'static>(&self, resource : R) -> Option<R> {
        let lock = self.raw.write().await.resources.insert(TypeId::of::<R>(), RwLock::new(ResourceCell::new(resource)))?;
        // SAFETY: TODO
        Some(unsafe{ lock.into_inner().await.read() })
    }

    /// TODO: Doc comments
    pub async fn remove<R : Resource + 'static>(&self) {
        self.raw.write().await.resources.remove(&TypeId::of::<R>());
    }

    /// TODO: Doc comments
    pub async fn take<R : Resource + 'static>(&self) -> Option<R> {
        let lock = self.raw.write().await.resources.remove(&TypeId::of::<R>())?;
        // SAFETY: TODO
        Some(unsafe{ lock.into_inner().await.read() })
    }

    /// Acquires a read lock to a [`ResourceCell`] by [`Resource`] type, if it exists.
    pub async fn get_ref<R : Resource + 'static>(&self) -> Option<ResourceCellReadGuard<'_, R>> {
        let raw = self.raw.read().await;
        let lock = raw.resources.get(&TypeId::of::<R>())?;
        Some(ResourceCellReadGuard {
            guard  : lock.read().await,
            marker : PhantomData
        })
    }

    /// TODO: Doc comment
    pub(crate) fn try_get_ref<R : Resource + 'static>(&self) -> Poll<Option<ResourceCellReadGuard<'_, R>>> {
        let Poll::Ready(raw) = self.raw.try_read() else { return Poll::Pending; };
        let Some(lock) = raw.resources.get(&TypeId::of::<R>()) else { return Poll::Ready(None); };
        let Poll::Ready(guard) = lock.try_read() else { return Poll::Pending; };
        Poll::Ready(Some(ResourceCellReadGuard {
            guard,
            marker : PhantomData
        }))
    }

    /// Acquires a write lock to a [`ResourceCell`] by [`Resource`] type, if it exists.
    pub async fn get_mut<R : Resource + 'static>(&self) -> Option<ResourceCellWriteGuard<'_, R>> {
        let raw = self.raw.read().await;
        let lock = raw.resources.get(&TypeId::of::<R>())?;
        Some(ResourceCellWriteGuard {
            guard  : lock.write().await,
            marker : PhantomData
        })
    }

    /// Acquires a write lock to a [`ResourceCell`] by [`Resource`] type, creating it if needed.
    pub async fn get_mut_or_insert<R : Resource + 'static>(&self, f : impl FnOnce() -> R) -> ResourceCellWriteGuard<'_, R> {
        let mut raw = self.raw.write().await;
        let type_id = TypeId::of::<R>();
        if let Some(lock) = raw.resources.get(&type_id) {
            ResourceCellWriteGuard {
                guard  : lock.write().await,
                marker : PhantomData
            }
        } else {
            let lock = unsafe{ RwLock::new_writing(ResourceCell::new(f())) };
            let guard = ResourceCellWriteGuard {
                guard  : unsafe{ lock.write_unchecked() },
                marker : PhantomData
            };
            raw.resources.insert(type_id, lock);
            guard
        }
    }

}


/// TODO: Doc comments
pub struct ResourceCellReadGuard<'l, R : Resource> {

    /// TODO: Doc comments
    guard  : RwLockReadGuard<ResourceCell>,

    /// TODO: Doc comments
    marker : PhantomData<&'l R>

}

impl<'l, R : Resource> Deref for ResourceCellReadGuard<'l, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        // SAFETY: TODO
        unsafe{ self.guard.get_ref::<R>() }
    }
}


/// TODO: Doc comments
pub struct ResourceCellWriteGuard<'l, R : Resource> {

    /// TODO: Doc comments
    guard  : RwLockWriteGuard<ResourceCell>,

    /// TODO: Doc comments
    marker : PhantomData<&'l R>

}

impl<'l, R : Resource> Deref for ResourceCellWriteGuard<'l, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        // SAFETY: TODO
        unsafe{ self.guard.get_ref::<R>() }
    }
}

impl<'l, R : Resource> DerefMut for ResourceCellWriteGuard<'l, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: TODO
        unsafe{ self.guard.get_mut::<R>() }
    }
}


/// A single cell in a [`ResourceStorage`].
pub struct ResourceCell {

    /// TODO: Doc comments
    layout   : Layout,

    /// A pointer to the contained value.
    data_ptr : NonNull<u8>,

    /// TODO: Doc comments
    drop     : unsafe fn(NonNull<u8>) -> ()

}

impl ResourceCell {

    /// Creates a new cell with the given [`Resource`] type.
    pub fn new<R : Resource>(resource : R) -> Self {
        let layout = Layout::new::<R>();
        let data_ptr = unsafe{ alloc(layout) };
        if (data_ptr.is_null()) {
            handle_alloc_error(layout)
        }
        // SAFETY: An alloc error was emitted above if `data_ptr` `is_null`.
        unsafe{ data_ptr.cast::<R>().write(resource); }
        Self {
            layout,
            // SAFETY: An alloc error was emitted above if `data_ptr` `is_null`.
            data_ptr : unsafe{ NonNull::new_unchecked(data_ptr) },
            // SAFETY: TODO
            drop     : |data_ptr| { unsafe{ data_ptr.cast::<R>().drop_in_place(); } }
        }
    }

    /// Returns a reference to the value in the cell.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell **is occupied**.
    /// - `R` is the type stored in this cell.
    pub unsafe fn get_ref<R : Resource>(&self) -> &R {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.data_ptr.cast::<R>().as_ref() }
    }

    /// Returns a mutable reference to the value in the cell.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell **is occupied**.
    /// - `R` is the type stored in this cell.
    pub unsafe fn get_mut<R : Resource>(&mut self) -> &mut R {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.data_ptr.cast::<R>().as_mut() }
    }

    /// Returns a pointer to the value in the cell.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell **is occupied**.
    /// - `R` is the type stored in this cell.
    /// - the pointer is not used when the cell is unoccupied or has been dropped.
    /// - data-races are prevented.
    pub unsafe fn get_ptr<R : Resource>(&self) -> *mut R {
        self.data_ptr.cast::<R>().as_ptr()
    }

    /// TODO: Doc comment
    pub unsafe fn read<R : Resource>(self) -> R {
        // SAFETY: TODO
        unsafe{ self.data_ptr.cast::<R>().read() }
    }

}


impl Drop for ResourceCell {
    fn drop(&mut self) {
        // SAFETY: TODO
        unsafe{ (self.drop)(self.data_ptr); }
        // SAFETY: TODO
        unsafe{ dealloc(self.data_ptr.as_ptr(), self.layout) }
    }
}
