//! A wrapper around many [`Archetype`]s with a safe API for operating on them.


use crate::archetype::Archetype;
use crate::entity::{ Entity, Entities };
use crate::component::ComponentBundle;
use crate::component::query::{ ComponentQuery, ReadOnlyComponentQuery, ComponentFilter };
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

    /// The inner data of this [`ArchetypeStorage`], behind a [`RwLock`].
    inner : RwLock<ArchetypeStorageInner>

}

/// The inner data of an [`ArchetypeStorage`].
pub struct ArchetypeStorageInner {

    /// The [`TypeId`] of a [`ComponentBundle`] implementor, to the index of the [`Archetype`] in [`ArchetypeStorageInner::archetypes`].
    bundles    : BTreeMap<TypeId, usize>,

    /// The [`TypeId`]s of the [`Component`](crate::component::Component)s stored in an [`Archetype`], to the index of the [`Archetype`] in [`ArchetypeStorageInner::archetypes`].
    components : BTreeMap<Box<[TypeId]>, usize>,

    /// The actual [`Archetype`]s. The ID of the archetype is its index in this Vec.
    archetypes : Vec<RwLock<Archetype>>

}

impl ArchetypeStorage {

    /// Attempts to acquire a read lock to the inner data, returning immediately if it can't.
    pub fn try_read_inner(&self) -> Poll<RwLockReadGuard<ArchetypeStorageInner>> {
        self.inner.try_read()
    }

    /// Acquires a read lock to the inner data.
    pub async fn read_inner(&self) -> RwLockReadGuard<ArchetypeStorageInner> {
        self.inner.read().await
    }

}

impl ArchetypeStorageInner {

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
        inner : RwLock::new(ArchetypeStorageInner {
            bundles    : BTreeMap::new(),
            components : BTreeMap::new(),
            archetypes : Vec::new()
        })
    } }

    /// Acquires a read lock to an [`Archetype`] by [`ComponentBundle`], if it exists.
    pub async fn get_ref<C : ComponentBundle + 'static>(&self) -> Option<RwLockReadGuard<Archetype>> {
        let inner = self.inner.read().await;
        // Try checking by TypeId (Fastest lookup).
        if let Some(&archetype_id) = inner.bundles.get(&TypeId::of::<C>()) {
            // SAFETY: A bundle of type `C` was previously inserted and must exist.
            return Some(unsafe{ inner.archetypes.get_unchecked(archetype_id) }.read().await);
        }
        // Try checking by sorted ComponentTypeInfo.
        {
            let ctis = C::type_info().into_iter().map(|cti| cti.type_id()).collect::<Vec<_>>();
            for (component_group, &archetype_id) in &inner.components {
                if (component_group.len() == ctis.len() && ctis.iter().all(|cti| component_group.contains(cti))) {
                    let mut inner = RwLockReadGuard::upgrade(inner).await;
                    inner.bundles.insert(TypeId::of::<C>(), archetype_id);
                    // SAFETY: A bundle with the same components as `C` was previously inserted and must exist.
                    return Some(unsafe{ inner.archetypes.get_unchecked(archetype_id) }.read().await);
                }
            }
        }
        // No matching archetypes found.
        None
    }

    /// Acquires a read lock to an [`Archetype`] by ID.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that this [`ArchetypeStorage`] actually has an [`Archetype`] by this ID.
    pub unsafe fn get_ref_by_id_unchecked(&self, archetype_id : usize) -> Poll<RwLockReadGuard<Archetype>> {
        let Poll::Ready(inner) = self.inner.try_read() else { return Poll::Pending };
        // SAFETY: The caller is responsible for ensuring that the archetype actually exists.
        unsafe{ inner.archetypes.get_unchecked(archetype_id) }.try_read()
    }

    /// Acquires a write lock to an [`Archetype`] by [`ComponentBundle`], if it exists.
    pub async fn get_mut<C : ComponentBundle + 'static>(&self) -> Option<RwLockWriteGuard<Archetype>> {
        self.get_mut_inner::<C>(self.inner.read().await).await.0
    }

    /// Acquires a write lock to an [`Archetype`] by [`ComponentBundle`], if it exists.
    ///
    /// If the [`RwLockReadGuard`] had to be upgraded during this operation, that is returned. Otherwise, `inner` is returned.
    async fn get_mut_inner<C : ComponentBundle + 'static>(&self, inner : RwLockReadGuard<ArchetypeStorageInner>)
        -> (Option<RwLockWriteGuard<Archetype>>, Either<RwLockReadGuard<ArchetypeStorageInner>, RwLockWriteGuard<ArchetypeStorageInner>>)
    {
        // Try checking by TypeId (Fastest lookup).
        if let Some(&archetype_id) = inner.bundles.get(&TypeId::of::<C>()) {
            // SAFETY: A bundle of type `C` was previously inserted and must exist.
            return (Some(unsafe{ inner.archetypes.get_unchecked(archetype_id) }.write().await), Either::A(inner));
        }
        // Try checking by sorted ComponentTypeInfo.
        {
            let ctis = C::type_info().into_iter().map(|cti| cti.type_id()).collect::<Vec<_>>();
            for (component_group, &archetype_id) in &inner.components {
                if (component_group.len() == ctis.len() && ctis.iter().all(|cti| component_group.contains(cti))) {
                    let mut inner = RwLockReadGuard::upgrade(inner).await;
                    inner.bundles.insert(TypeId::of::<C>(), archetype_id);
                    // SAFETY: A bundle with the same components as `C` was previously inserted and must exist.
                    return (Some(unsafe{ inner.archetypes.get_unchecked(archetype_id) }.write().await), Either::B(inner));
                }
            }
        }
        // No matching archetypes found.
        (None, Either::A(inner))
    }

    /// Acquires a write lock to an [`Archetype`] by ID.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that this [`ArchetypeStorage`] actually has an [`Archetype`] by this ID.
    pub unsafe fn get_mut_by_id_unchecked(&self, archetype_id : usize) -> Poll<RwLockWriteGuard<Archetype>> {
        let Poll::Ready(inner) = self.inner.try_read() else { return Poll::Pending };
        // SAFETY: The caller is responsible for ensuring that the archetype actually exists.
        unsafe{ inner.archetypes.get_unchecked(archetype_id) }.try_write()
    }

    /// Acquires a write lock to an [`Archetype`] by [`ComponentBundle`] if it exists, or creates one.
    pub async fn get_mut_or_create<C : ComponentBundle + 'static>(&self) -> RwLockWriteGuard<Archetype> {
        let inner = self.inner.read().await;
        let (maybe_archetype, inner) = self.get_mut_inner::<C>(inner).await;
        if let Some(archetype) = maybe_archetype {
            return archetype;
        }
        let mut inner = match (inner) {
            Either::A(read  ) => { RwLockReadGuard::upgrade(read).await },
            Either::B(write ) => { write }
        };
        // No matching archetypes found. Create a new one.
        let archetype_id = inner.archetypes.len();
        inner.bundles.insert(TypeId::of::<C>(), archetype_id);
        inner.components.insert(<C as ComponentBundle>::type_info().into_iter().map(|cti| cti.type_id()).collect::<Box<[_]>>(), archetype_id);
        // SAFETY: `write_unchecked` is called below and returned. The caller will eventually drop it.
        inner.archetypes.push(unsafe{ RwLock::new_writing(Archetype::new::<C>(
            archetype_id,
            #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
            type_name::<C>()
        )) });
        // SAFETY: The `RwLock` was created above using `new_writing`, ensuring that it is already
        //         locked, but has no locks to it.
        return unsafe{ inner.archetypes.get_unchecked(archetype_id).write_unchecked() };
    }

    /// Gets the corresponding [`Archetype`] (creating it if needed), then adds a row, "spawning" an entity.
    ///
    /// # Panics
    /// Panics if the given [`ComponentBundle`] is not valid.
    /// See [`BundleValidator`](crate::component::BundleValidator).
    #[track_caller]
    pub async fn spawn<C : ComponentBundle + 'static>(&self, bundle : C) -> Entity {
        C::validate().panic_on_violation();
        // SAFETY: The archetype rules were checked in the line above.
        unsafe{ self.spawn_unchecked::<C>(bundle).await }
    }

    /// Gets the corresponding [`Archetype`] (creating it if needed), then adds a row, "spawning an entity", without checking if the given [`ComponentBundle`] is valid.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ComponentBundle`] does not violate the archetype rules. See [`BundleValidator`](crate::component::BundleValidator).
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
    /// See [`BundleValidator`](crate::component::BundleValidator).
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
    /// The caller is responsible for ensuring that the given [`ComponentBundle`] does not violate the archetype rules. See [`BundleValidator`](crate::component::BundleValidator).
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
    /// The caller is responsible for ensuring that the given [`ReadOnlyComponentQuery`] does not violate the borrow checker rules. See [`BundleValidator`](crate::component::BundleValidator).
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
    /// The caller is responsible for ensuring that the given [`ComponentQuery`] does not violate the borrow checker rules. See [`BundleValidator`](crate::component::BundleValidator).
    pub async unsafe fn query_unchecked_mut<'l, Q : ComponentQuery + 'l, F : ComponentFilter>(&'l self) -> Entities<'l, Q, F> {
        // SAFETY: The caller is responsible for ensuring that the archetype rules are not violated.
        FunctionCallFuture::new(|| unsafe{ Entities::<Q, F>::acquire_archetypes_unchecked(self) }).await
    }

}
