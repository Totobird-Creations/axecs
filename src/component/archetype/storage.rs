//! A wrapper around many [`Archetype`]s with a safe API for operating on them.


use crate::entity::{ Entity, Entities };
use crate::component::bundle::ComponentBundle;
use crate::component::query::{ ComponentQuery, ReadOnlyComponentQuery, ComponentFilter };
use crate::component::archetype::Archetype;
use crate::util::rwlock::{ RwLock, RwLockReadGuard, RwLockWriteGuard };
use crate::util::either::Either;
use crate::util::future::FunctionCallFuture;
use core::any::TypeId;
#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use core::any::type_name;
use core::task::Poll;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::vec::Vec;


/// A wrapper for several different [`Archetype`]s.
pub struct ArchetypeStorage {

    /// The raw data of this [`ArchetypeStorage`], behind a [`RwLock`].
    raw : RwLock<RawArchetypeStorage>

}

/// The raw data of an [`ArchetypeStorage`].
pub struct RawArchetypeStorage {

    /// The [`TypeId`] of a [`ComponentBundle`] implementor, to the index of the [`Archetype`] in [`RawArchetypeStorage::archetypes`].
    bundles    : BTreeMap<TypeId, usize>,

    /// The [`TypeId`]s of the [`Component`](crate::component::Component)s stored in an [`Archetype`], to the index of the [`Archetype`] in [`RawArchetypeStorage::archetypes`].
    components : BTreeMap<Box<[TypeId]>, usize>,

    /// The actual [`Archetype`]s. The ID of the archetype is its index in this Vec.
    archetypes : Vec<RwLock<Archetype>>

}

impl ArchetypeStorage {

    /// Attempts to acquire a read lock to the raw data, returning immediately if it can't.
    pub fn try_read_raw(&self) -> Poll<RwLockReadGuard<RawArchetypeStorage>> {
        self.raw.try_read()
    }

    /// Acquires a read lock to the raw data.
    pub async fn read_raw(&self) -> RwLockReadGuard<RawArchetypeStorage> {
        self.raw.read().await
    }

}

impl RawArchetypeStorage {

    /// Returns an [`Iterator`] over `TypeId`s of [`ComponentBundle`]s stored by the [`Archetype`]s.
    pub fn archetype_bundles(&self) -> impl Iterator<Item = (&[TypeId], usize)> {
        self.components.iter().map(|(k, v)| (&**k, *v))
    }

    /// Returns an [`Iterator`] over slices of `TypeId`s in [`ComponentBundle`]s stored by the [`Archetype`]s.
    pub fn archetype_components(&self) -> impl Iterator<Item = (&[TypeId], usize)> {
        self.components.iter().map(|(k, v)| (&**k, *v))
    }

    /// Returns an [`Iterator`] over [`RwLock`] wrapped [`Archetype`]s.
    pub fn archetypes(&self) -> impl Iterator<Item = &RwLock<Archetype>> {
        self.archetypes.iter()
    }

}

impl ArchetypeStorage {

    /// Creates an empty [`ArchetypeStorage`].
    pub fn new() -> Self { Self {
        raw : RwLock::new(RawArchetypeStorage {
            bundles    : BTreeMap::new(),
            components : BTreeMap::new(),
            archetypes : Vec::new()
        })
    } }

    /// Acquires a read lock to an [`Archetype`] by [`ComponentBundle`], if it exists.
    pub async fn get_ref<C : ComponentBundle + 'static>(&self) -> Option<RwLockReadGuard<Archetype>> {
        let raw = self.raw.read().await;
        // Try checking by TypeId (Fastest lookup).
        if let Some(&archetype_id) = raw.bundles.get(&TypeId::of::<C>()) {
            // SAFETY: A bundle of type `C` was previously inserted and must exist.
            return Some(unsafe{ raw.archetypes.get_unchecked(archetype_id) }.read().await);
        }
        // Try checking by sorted ComponentTypeInfo.
        {
            let ctis = C::type_info().into_iter().map(|cti| cti.type_id()).collect::<Vec<_>>();
            for (component_group, &archetype_id) in &raw.components {
                if (component_group.len() == ctis.len() && ctis.iter().all(|cti| component_group.contains(cti))) {
                    let mut raw = RwLockReadGuard::upgrade(raw).await;
                    raw.bundles.insert(TypeId::of::<C>(), archetype_id);
                    // SAFETY: A bundle with the same components as `C` was previously inserted and must exist.
                    return Some(unsafe{ raw.archetypes.get_unchecked(archetype_id) }.read().await);
                }
            }
        }
        // No matching archetypes found.
        None
    }

    /// Tries to acquire a read lock to an [`Archetype`] by ID.
    pub fn get_ref_by_id(&self, archetype_id : usize) -> Poll<Option<RwLockReadGuard<Archetype>>> {
        let Poll::Ready(raw) = self.raw.try_read() else { return Poll::Pending };
        let Some(archetype) = raw.archetypes.get(archetype_id) else { return Poll::Ready(None) };
        match (archetype.try_read()) {
            Poll::Ready(out) => Poll::Ready(Some(out)),
            Poll::Pending    => Poll::Pending
        }
    }

    /// Tries to acquire a read lock to an [`Archetype`] by ID, without checking if it exists.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that this [`ArchetypeStorage`] actually has an [`Archetype`] by this ID.
    pub unsafe fn get_ref_by_id_unchecked(&self, archetype_id : usize) -> Poll<RwLockReadGuard<Archetype>> {
        let Poll::Ready(raw) = self.raw.try_read() else { return Poll::Pending };
        // SAFETY: The caller is responsible for ensuring that the archetype actually exists.
        unsafe{ raw.archetypes.get_unchecked(archetype_id) }.try_read()
    }

    /// Acquires a write lock to an [`Archetype`] by [`ComponentBundle`], if it exists.
    pub async fn get_mut<C : ComponentBundle + 'static>(&self) -> Option<RwLockWriteGuard<Archetype>> {
        self.get_mut_raw::<C>(self.raw.read().await).await.0
    }

    /// Tries to acquire a write lock to an [`Archetype`] by [`ComponentBundle`], if it exists.
    ///
    /// If the [`RwLockReadGuard`] had to be upgraded during this operation, that is returned. Otherwise, `raw` is returned.
    async fn get_mut_raw<C : ComponentBundle + 'static>(&self, raw : RwLockReadGuard<RawArchetypeStorage>)
        -> (Option<RwLockWriteGuard<Archetype>>, Either<RwLockReadGuard<RawArchetypeStorage>, RwLockWriteGuard<RawArchetypeStorage>>)
    {
        // Try checking by TypeId (Fastest lookup).
        if let Some(&archetype_id) = raw.bundles.get(&TypeId::of::<C>()) {
            // SAFETY: A bundle of type `C` was previously inserted and must exist.
            return (Some(unsafe{ raw.archetypes.get_unchecked(archetype_id) }.write().await), Either::A(raw));
        }
        // Try checking by sorted ComponentTypeInfo.
        {
            let ctis = C::type_info().into_iter().map(|cti| cti.type_id()).collect::<Vec<_>>();
            for (component_group, &archetype_id) in &raw.components {
                if (component_group.len() == ctis.len() && ctis.iter().all(|cti| component_group.contains(cti))) {
                    let mut raw = RwLockReadGuard::upgrade(raw).await;
                    raw.bundles.insert(TypeId::of::<C>(), archetype_id);
                    // SAFETY: A bundle with the same components as `C` was previously inserted and must exist.
                    return (Some(unsafe{ raw.archetypes.get_unchecked(archetype_id) }.write().await), Either::B(raw));
                }
            }
        }
        // No matching archetypes found.
        (None, Either::A(raw))
    }

    /// Tries to acquire a write lock to an [`Archetype`] by ID.
    pub fn get_mut_by_id(&self, archetype_id : usize) -> Poll<Option<RwLockWriteGuard<Archetype>>> {
        let Poll::Ready(raw) = self.raw.try_read() else { return Poll::Pending };
        let Some(archetype) = raw.archetypes.get(archetype_id) else { return Poll::Ready(None) };
        match (archetype.try_write()) {
            Poll::Ready(out) => Poll::Ready(Some(out)),
            Poll::Pending    => Poll::Pending
        }
    }

    /// Tries to acquire a write lock to an [`Archetype`] by ID, without checking if it exists.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that this [`ArchetypeStorage`] actually has an [`Archetype`] by this ID.
    pub unsafe fn get_mut_by_id_unchecked(&self, archetype_id : usize) -> Poll<RwLockWriteGuard<Archetype>> {
        let Poll::Ready(raw) = self.raw.try_read() else { return Poll::Pending };
        // SAFETY: The caller is responsible for ensuring that the archetype actually exists.
        unsafe{ raw.archetypes.get_unchecked(archetype_id) }.try_write()
    }

    /// Acquires a write lock to an [`Archetype`] by [`ComponentBundle`] if it exists, or creates one.
    pub async fn get_mut_or_create<C : ComponentBundle + 'static>(&self) -> RwLockWriteGuard<Archetype> {
        let raw = self.raw.read().await;
        let (maybe_archetype, raw) = self.get_mut_raw::<C>(raw).await;
        if let Some(archetype) = maybe_archetype {
            return archetype;
        }
        let mut raw = match (raw) {
            Either::A(read  ) => { RwLockReadGuard::upgrade(read).await },
            Either::B(write ) => { write }
        };
        // No matching archetypes found. Create a new one.
        let archetype_id = raw.archetypes.len();
        raw.bundles.insert(TypeId::of::<C>(), archetype_id);
        raw.components.insert(<C as ComponentBundle>::type_info().into_iter().map(|cti| cti.type_id()).collect::<Box<[_]>>(), archetype_id);
        // SAFETY: `write_unchecked` is called below and returned. The caller will eventually drop it.
        raw.archetypes.push(unsafe{ RwLock::new_writing(Archetype::new::<C>(
            archetype_id,
            #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
            type_name::<C>()
        )) });
        // SAFETY: The `RwLock` was created above using `new_writing`, ensuring that it is already
        //         locked, but has no locks to it.
        return unsafe{ raw.archetypes.get_unchecked(archetype_id).write_unchecked() };
    }

    /// Gets the corresponding [`Archetype`] (creating it if needed), then adds a row, "spawning" an entity.
    ///
    /// # Panics
    /// Panics if the given [`ComponentBundle`] is not valid.
    /// See [`BundleValidator`](crate::component::bundle::BundleValidator).
    #[track_caller]
    pub async fn spawn<C : ComponentBundle + 'static>(&self, bundle : C) -> Entity {
        C::validate().panic_on_violation();
        // SAFETY: The archetype rules were checked in the line above.
        unsafe{ self.spawn_unchecked::<C>(bundle).await }
    }

    /// Gets the corresponding [`Archetype`] (creating it if needed), then adds a row, "spawning an entity", without checking if the given [`ComponentBundle`] is valid.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ComponentBundle`] does not violate the archetype rules. See [`BundleValidator`](crate::component::bundle::BundleValidator).
    pub async unsafe fn spawn_unchecked<C : ComponentBundle + 'static>(&self, bundle : C) -> Entity {
        let mut archetype = self.get_mut_or_create::<C>().await;
        Entity::new(
            archetype.archetype_id(),
            #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
            archetype.archetype_name(),
            // SAFETY: The caller is responsible for ensuring that the archetype rules are not violated.
            unsafe{ archetype.spawn_unchecked(bundle) }
        )
    }

    /// Gets the corresponding [`Archetype`] (creating it if needed), then adds several rows, "spawning" entities.
    ///
    /// # Panics
    /// Panics if the given [`ComponentBundle`]s are not valid.
    /// See [`BundleValidator`](crate::component::bundle::BundleValidator).
    ///
    /// This is more efficient than [`ArchetypeStorage::spawn`], but has the downside of only being able to spawn entities with the same [`ComponentBundle`] type.
    #[track_caller]
    pub async fn spawn_batch<C : ComponentBundle + 'static>(&self, bundles : impl IntoIterator<Item = C>) -> impl Iterator<Item = Entity> {
        C::validate().panic_on_violation();
        // SAFETY: The archetype rules were checked in the line above.
        unsafe{ self.spawn_batch_unchecked::<C>(bundles).await }
    }

    /// Gets the corresponding [`Archetype`] (creating it if needed), then adds several rows, "spawning" entities, without checking if the given [`ComponentBundle`] is valid.
    ///
    /// This is more efficient than [`ArchetypeStorage::spawn`], but has the downside of only being able to spawn entities with the same [`ComponentBundle`] type.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ComponentBundle`] does not violate the archetype rules. See [`BundleValidator`](crate::component::bundle::BundleValidator).
    pub async unsafe fn spawn_batch_unchecked<C : ComponentBundle + 'static>(&self, bundles : impl IntoIterator<Item = C>) -> impl Iterator<Item = Entity> {
        let mut archetype = self.get_mut_or_create::<C>().await;
        let mut entities  = Vec::new();
        for bundle in bundles {
            entities.push(Entity::new(
                archetype.archetype_id(),
                #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                archetype.archetype_name(),
                // SAFETY: The caller is responsible for ensuring that the archetype rules are not violated.
                unsafe{ archetype.spawn_unchecked::<C>(bundle) }
            ));
        }
        entities.into_boxed_slice().into_iter()
    }

    /// Removes a row from an [`Archetype`], if it exists.
    pub async fn despawn(&self, entity : Entity) {
        let archetype_id = entity.archetype_id();
        let Some(mut archetype) = FunctionCallFuture::new(|| self.get_mut_by_id(archetype_id)).await else { return };
        let row = entity.archetype_row();
        if (archetype.has_row(row)) {
            // SAFETY: It was checked in the line above that the row exists.
            unsafe{ archetype.despawn_unchecked(row); }
        }
    }

    /// Resmoves a row from an [`Archetype`] without checking if it exists.
    /// 
    /// # Safety:
    /// The caller is responsible for ensuring that the [`Archetype`] and row exist.
    pub async fn despawn_unchecked(&self, entity : Entity) {
        let archetype_id = entity.archetype_id();
        // SAFETY: The caller is responsible for ensuring that the archetype exists.
        let mut archetype = FunctionCallFuture::new(|| unsafe{ self.get_mut_by_id_unchecked(archetype_id) }).await;
        // SAFETY: The caller is responsible for ensuring that the row exists.
        unsafe{ archetype.despawn_unchecked(entity.archetype_row()); }
    }

    /// Returns [`Entities`] that match the given [`ReadOnlyComponentQuery`] and [`ComponentFilter`].
    ///
    /// # Panics
    /// Panics if the given [`ReadOnlyComponentQuery`] is not valid.
    /// See [`QueryValidator`](crate::query::QueryValidator).
    #[track_caller]
    pub async fn query<'l, Q : ReadOnlyComponentQuery + 'l, F : ComponentFilter>(&'l self) -> Entities<'l, Q, F> {
        Q::validate().panic_on_violation();
        // SAFETY: The archetype rules were checked in the line above.
        unsafe{ self.query_unchecked::<'l, Q, F>().await }
    }

    /// Returns [`Entities`] that match the given filter, without checking if the given [`ReadOnlyComponentQuery`] is valid.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ReadOnlyComponentQuery`] does not violate the borrow checker rules. See [`BundleValidator`](crate::component::bundle::BundleValidator).
    pub async unsafe fn query_unchecked<'l, Q : ReadOnlyComponentQuery + 'l, F : ComponentFilter>(&'l self) -> Entities<'l, Q, F> {
        // SAFETY: The caller is responsible for ensuring that the archetype rules are not violated.
        FunctionCallFuture::new(|| unsafe{ Entities::<Q, F>::acquire_archetypes_unchecked(self) }).await
    }

    /// Returns [`Entities`] that match the given [`ComponentQuery`] and [`ComponentFilter`].
    ///
    /// # Panics
    /// Panics if the given [`ReadOnlyComponentQuery`] is not valid.
    /// See [`QueryValidator`](crate::query::QueryValidator).
    #[track_caller]
    pub async fn query_mut<'l, Q : ComponentQuery + 'l, F : ComponentFilter>(&'l self) -> Entities<'l, Q, F> {
        Q::validate().panic_on_violation();
        // SAFETY: The archetype rules were checked in the line above.
        unsafe{ self.query_unchecked_mut::<'l, Q, F>().await }
    }

    /// Returns [`Entities`] that match the given filter, without checking if the given [`ComponentQuery`] is valid.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ComponentQuery`] does not violate the borrow checker rules. See [`BundleValidator`](crate::component::bundle::BundleValidator).
    pub async unsafe fn query_unchecked_mut<'l, Q : ComponentQuery + 'l, F : ComponentFilter>(&'l self) -> Entities<'l, Q, F> {
        // SAFETY: The caller is responsible for ensuring that the archetype rules are not violated.
        FunctionCallFuture::new(|| unsafe{ Entities::<Q, F>::acquire_archetypes_unchecked(self) }).await
    }

}
