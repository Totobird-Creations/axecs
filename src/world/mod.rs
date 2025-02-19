//! TODO: Doc comments


mod commands;
pub use commands::*;


use crate::resource::{ Resource, ResourceStorage, ResourceCellReadGuard, ResourceCellWriteGuard };
use crate::entity::Entity;
use crate::component::bundle::ComponentBundle;
use crate::component::archetype::ArchetypeStorage;
use crate::query::{ Query, ReadOnlyQuery, PersistentQueryState };
use crate::system::{ SystemId, IntoSystem, IntoReadOnlySystem, ReadOnlySystem, PersistentSystemState };
use crate::app::AppExit;
use crate::schedule::system::TypeErasedSystem;
use crate::util::rwlock::RwLock;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{ AtomicU8, Ordering };
use core::pin::Pin;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::sync::Arc;


/// A wrapper for an application's exiting state, resources, entities, etc.
pub struct World {

    /// The current exiting state of the app.
    ///
    /// ### Possible values
    /// - `0`: Not exiting.
    /// - `1`: Exiting, status being written.
    /// - `2`: Exiting, status exists.
    /// - `3`: Exiting, status taken.
    is_exiting  : AtomicU8,

    /// The [`AppExit`] status of the app.
    exit_status : UnsafeCell<MaybeUninit<AppExit>>,

    /// The [`Resource`]s in this world.
    resources   : ResourceStorage,

    /// The [`Component`](crate::component::Component) [`Archetype`](crate::component::archetype::Archetype)s in this world.
    archetypes  : ArchetypeStorage,

    /// TODO: Doc comments
    pub(crate) cmd_queue : RwLock<Vec<Box<dyn FnOnce(Arc<World>) -> Pin<Box<dyn Future<Output = ()>>>>>>

}

impl World {

    /// Returns a reference to the [`Resource`]s in this world.
    #[inline]
    pub fn resources(&self) -> &ResourceStorage {
        &self.resources
    }

    /// Returns a reference to the [`Archetype`](crate::component::archetype::Archetype)s in this world.
    #[inline]
    pub fn archetypes(&self) -> &ArchetypeStorage {
        &self.archetypes
    }

}

impl World {

    /// Creates a new [`World`] with nothing in it.
    #[inline]
    pub fn new() -> Self { Self {
        is_exiting  : AtomicU8::new(0),
        exit_status : UnsafeCell::new(MaybeUninit::uninit()),
        resources   : ResourceStorage::new(),
        archetypes  : ArchetypeStorage::new(),
        cmd_queue   : RwLock::new(Vec::new())
    } }

    /// creates a new [`World`] with some [`Resource`]s in it to start.
    #[inline]
    pub fn new_with(resources : ResourceStorage) -> Self { Self {
        is_exiting  : AtomicU8::new(0),
        exit_status : UnsafeCell::new(MaybeUninit::uninit()),
        resources,
        archetypes  : ArchetypeStorage::new(),
        cmd_queue   : RwLock::new(Vec::new())
    } }


    /// Returns `true` if the application is currently exiting and should run its shutdown schedules.
    pub fn is_exiting(&self) -> bool {
        self.is_exiting.load(Ordering::Relaxed) >= 2
    }

    /// Takes the current [`AppExit`] from this application.
    ///
    /// # Panics
    /// Panics if the app is not exiting, or the [`AppExit`] has already been taken.
    pub fn take_exit_status(&self) -> AppExit {
        match (self.is_exiting.compare_exchange(2, 3, Ordering::Acquire, Ordering::Relaxed)) {
            Ok(_) => {
                // SAFETY: TODO
                unsafe{ (*self.exit_status.get()).assume_init_read() }
            },
            Err(1) => { panic!("Can not take exit status while app is writing exit status") },
            Err(3) => { panic!("Can not take exit status when app exit status has already been taken") }
            Err(_) => { panic!("Can not take exit status when app is not exiting") }
        }
    }

    /// Signals that the application should begin exiting.
    ///
    /// # Panics
    /// Panics if the app is already exiting.
    /// See [`World::try_exit`] for a non-panicking variant.
    pub fn exit(&self, status : AppExit) {
        match (self.is_exiting.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)) {
            Ok(_) => {
                // SAFETY: `self.is_exiting` was 0, meaning `self.exit_status` has not been written, nor is being written.
                //         `self.is_exiting` is now 1, preventing it from being overwritten again and leaking.
                unsafe{ (*self.exit_status.get()).write(status); }
                self.is_exiting.store(2, Ordering::Relaxed);
            },
            Err(_) => { panic!("Can not exit already exiting app") }
        }
    }

    /// Signals that the application should begin exiting.
    ///
    /// If the application is already exiting, this is a no-op.
    pub fn try_exit(&self, status : AppExit) {
        match (self.is_exiting.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)) {
            Ok(_) => {
                // SAFETY: `self.is_exiting` was 0, meaning `self.exit_status` has not been written, nor is being written.
                //         `self.is_exiting` is now 1, preventing it from being overwritten again and leaking.
                unsafe{ (*self.exit_status.get()).write(status); }
                self.is_exiting.store(2, Ordering::Relaxed);
            },
            Err(_) => { }
        }
    }


    /// Inserts a [`Resource`] into this world, overwriting any previous resource of the same type.
    ///
    /// This is more efficient than [`World::replace_resource`], as it doesn't have to wait for the individual resource to lock.
    pub async fn insert_resource<R : Resource + 'static>(self : &Arc<Self>, resource : R) {
        self.resources.insert::<R>(resource).await
    }

    /// Inserts a [`Resource`] into this world, returning the old resource of the same type if it existed.
    ///
    /// Use [`World::insert_resource`] if you don't need the old value.
    pub async fn replace_resource<R : Resource + 'static>(self : &Arc<Self>, resource : R) -> Option<R> {
        self.resources.replace::<R>(resource).await
    }

    /// Removes a [`Resource`] from this world.
    ///
    /// This is more efficient than [`World::take_resource`], as it doesn't have to wait for the individual resource to lock.
    pub async fn remove_resource<R : Resource + 'static>(self : &Arc<Self>) {
        self.resources.remove::<R>().await
    }

    /// Removes a [`Resource`] from this world, returning it if it existed.
    ///
    /// Use [`World::remove_resource`] if you don't need the old value.
    pub async fn take_resource<R : Resource + 'static>(self : &Arc<Self>) -> Option<R> {
        self.resources.take::<R>().await
    }

    /// Returns a reference to a [`Resource`] if it exists.
    pub async fn get_resource_ref<R : Resource + 'static>(&self) -> Option<ResourceCellReadGuard<'_, R>> {
        self.resources.get_ref::<R>().await
    }

    /// Returns a mutable reference to a [`Resource`] if it exists.
    pub async fn get_resource_mut<R : Resource + 'static>(&self) -> Option<ResourceCellWriteGuard<'_, R>> {
        self.resources.get_mut::<R>().await
    }

    /// Returns a mutable reference to a [`Resource`], creating it if needed.
    pub async fn get_resource_mut_or_insert<R : Resource + 'static>(&self, f : impl FnOnce() -> R) -> ResourceCellWriteGuard<'_, R> {
        self.resources.get_mut_or_insert::<R>(f).await
    }


    /// Spawns an entity with some [`Component`](crate::component::Component)s.
    ///
    /// # Panics
    /// Panics if the given [`ComponentBundle`] is not valid.
    /// See [`BundleValidator`](crate::component::bundle::BundleValidator).
    #[track_caller]
    pub async fn spawn<B : ComponentBundle + 'static>(self : &Arc<Self>, bundle : B) -> Entity {
        self.archetypes.spawn::<B>(bundle).await
    }

    /// Spawns an entity with some [`Component`](crate::component::Component)s, without checking that the given [`ComponentBundle`] is valid.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ComponentBundle`] does not violate the archetype rules. See [`BundleValidator`](crate::component::bundle::BundleValidator).
    pub async unsafe fn spawn_unchecked<B : ComponentBundle + 'static>(self : &Arc<Self>, bundle : B) -> Entity {
        unsafe{ self.archetypes.spawn_unchecked::<B>(bundle).await }
    }

    /// Spawns multiple entities with some [`Component`](crate::component::Component)s.
    ///
    /// This is more efficient than [`World::spawn`], but has the downside of only being able to spawn entities with the same [`ComponentBundle`] type.
    ///
    /// # Panics
    /// Panics if the given [`ComponentBundle`]s are not valid.
    /// See [`BundleValidator`](crate::component::bundle::BundleValidator).
    #[track_caller]
    pub async fn spawn_batch<'l, B : ComponentBundle + 'static>(self : &'l Arc<Self>, bundles : impl IntoIterator<Item = B> + 'l) -> impl Iterator<Item = Entity> {
        self.archetypes.spawn_batch::<B>(bundles).await
    }

    /// Spawns multiple entities with some [`Component`](crate::component::Component)s, without checking that the given [`ComponentBundle`] is valid.
    ///
    /// This is more efficient than [`World::spawn`], but has the downside of only being able to spawn entities with the same [`ComponentBundle`] type.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ComponentBundle`] does not violate the archetype rules. See [`BundleValidator`](crate::component::bundle::BundleValidator).
    pub async unsafe fn spawn_batch_unchecked<'l, B : ComponentBundle + 'static>(self : &'l Arc<Self>, bundles : impl IntoIterator<Item = B> + 'l) -> impl Iterator<Item = Entity> {
        unsafe{ self.archetypes.spawn_batch_unchecked::<B>(bundles).await }
    }

    /// Removes an entity.
    pub async fn despawn(self : &Arc<Self>, entity : Entity) {
        self.archetypes.despawn(entity).await
    }

    /// Removes an entity without checking that it exists.
    ///
    /// # Safety
    /// You are responsible for ensuring that the given entity exists.
    pub async unsafe fn despawn_unchecked(self : &Arc<Self>, entity : Entity) {
        self.archetypes.despawn_unchecked(entity).await
    }


    /// TODO: Doc comments
    #[track_caller]
    pub fn query<Q : ReadOnlyQuery>(self : &Arc<Self>) -> PersistentQueryState<Q> {
        Q::validate().panic_on_violation();
        // SAFETY: TODO
        unsafe{ self.query_unchecked::<Q>() }
    }

    /// TODO: Doc comments
    pub unsafe fn query_unchecked<Q : ReadOnlyQuery>(self : &Arc<Self>) -> PersistentQueryState<Q> {
        // SAFETY: TODO
        unsafe{ PersistentQueryState::<Q>::new(Arc::clone(self), Some(SystemId::unique())) }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn query_mut<Q : Query>(self : &Arc<Self>) -> PersistentQueryState<Q> {
        Q::validate().panic_on_violation();
        // SAFETY: TODO
        unsafe{ self.query_unchecked_mut::<Q>() }
    }

    /// TODO: Doc comments
    pub unsafe fn query_unchecked_mut<Q : Query>(self : &Arc<Self>) -> PersistentQueryState<Q> {
        // SAFETY: TODO
        unsafe{ PersistentQueryState::<Q>::new(Arc::clone(self), Some(SystemId::unique())) }
    }


    /// TODO: Doc comments
    #[track_caller]
    pub fn system<S : IntoReadOnlySystem<Params, Return>, Params, Return>(self : &Arc<Self>, system : S) -> PersistentSystemState<S::System, Return>
    where <S as IntoSystem<Params, Return>>::System : ReadOnlySystem<Return>
    {
        unsafe{ PersistentSystemState::new(Arc::clone(self), system.into_system(Arc::clone(self), Some(SystemId::unique()))) }
    }

    /// TODO: Doc comments
    pub fn system_unchecked<S : IntoReadOnlySystem<Params, Return>, Params, Return>(self : &Arc<Self>, system : S) -> PersistentSystemState<S::System, Return>
    where <S as IntoSystem<Params, Return>>::System : ReadOnlySystem<Return>
    {
        unsafe{ PersistentSystemState::new(Arc::clone(self), system.into_system_unchecked(Arc::clone(self), Some(SystemId::unique()))) }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn system_mut<S : IntoSystem<Params, Return>, Params, Return>(self : &Arc<Self>, system : S) -> PersistentSystemState<S::System, Return> {
        unsafe{ PersistentSystemState::new(Arc::clone(self), system.into_system(Arc::clone(self), Some(SystemId::unique()))) }
    }

    /// TODO: Doc comments
    pub fn system_unchecked_mut<S : IntoSystem<Params, Return>, Params, Return>(self : &Arc<Self>, system : S) -> PersistentSystemState<S::System, Return> {
        unsafe{ PersistentSystemState::new(Arc::clone(self), system.into_system_unchecked(Arc::clone(self), Some(SystemId::unique()))) }
    }

    /// TODO: Doc comments
    pub async unsafe fn run_erased_system<Passed, Return>(self : &Arc<Self>, system : &mut dyn TypeErasedSystem<Passed, Return>, passed : Passed) -> Return {
        unsafe{ system.acquire_and_run(passed, Arc::clone(self)) }.await
    }

}
