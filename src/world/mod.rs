//! TODO: Doc comments


mod commands;
pub use commands::*;


use crate::resource::{ Resource, ResourceStorage, ResourceCellReadGuard, ResourceCellWriteGuard };
use crate::entity::Entity;
use crate::component::bundle::ComponentBundle;
use crate::component::archetype::ArchetypeStorage;
use crate::query::{ Query, ReadOnlyQuery, StatelessQuery, PersistentQueryState, StatelessQueryItem, QueryAcquireFuture };
use crate::system::{ IntoSystem, IntoReadOnlySystem, IntoStatelessSystem, ReadOnlySystem, StatelessSystem, PersistentSystemState };
use crate::app::AppExit;
use crate::schedule::system::TypeErasedSystem;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{ AtomicU8, Ordering };


/// TODO: Doc comment
static mut UNIT : () = ();


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
    archetypes  : ArchetypeStorage

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
        archetypes  : ArchetypeStorage::new()
    } }

    /// creates a new [`World`] with some [`Resource`]s in it to start.
    #[inline]
    pub fn new_with(resources : ResourceStorage) -> Self { Self {
        is_exiting  : AtomicU8::new(0),
        exit_status : UnsafeCell::new(MaybeUninit::uninit()),
        resources,
        archetypes  : ArchetypeStorage::new()
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
    pub async fn insert_resource<R : Resource + 'static>(&self, resource : R) {
        self.resources.insert::<R>(resource).await
    }

    /// Inserts a [`Resource`] into this world, returning the old resource of the same type if it existed.
    /// 
    /// Use [`World::insert_resource`] if you don't need the old value.
    pub async fn replace_resource<R : Resource + 'static>(&self, resource : R) -> Option<R> {
        self.resources.replace::<R>(resource).await
    }

    /// Removes a [`Resource`] from this world.
    /// 
    /// This is more efficient than [`World::take_resource`], as it doesn't have to wait for the individual resource to lock.
    pub async fn remove_resource<R : Resource + 'static>(&self) {
        self.resources.remove::<R>().await
    }

    /// Removes a [`Resource`] from this world, returning it if it existed.
    /// 
    /// Use [`World::remove_resource`] if you don't need the old value.
    pub async fn take_resource<R : Resource + 'static>(&self) -> Option<R> {
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


    /// Spawns an entity with some [`Component`](crate::component::Component)s.
    ///
    /// # Panics
    /// Panics if the given [`ComponentBundle`] is not valid.
    /// See [`BundleValidator`](crate::component::bundle::BundleValidator).
    #[track_caller]
    pub async fn spawn<B : ComponentBundle + 'static>(&self, bundle : B) -> Entity {
        self.archetypes.spawn::<B>(bundle).await
    }

    /// Spawns an entity with some [`Component`](crate::component::Component)s, without checking that the given [`ComponentBundle`] is valid.
    /// 
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ComponentBundle`] does not violate the archetype rules. See [`BundleValidator`](crate::component::bundle::BundleValidator).
    pub async unsafe fn spawn_unchecked<B : ComponentBundle + 'static>(&self, bundle : B) -> Entity {
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
    pub async fn spawn_batch<B : ComponentBundle + 'static>(&self, bundles : impl IntoIterator<Item = B>) -> impl IntoIterator<Item = Entity> {
        self.archetypes.spawn_batch::<B>(bundles).await
    }

    /// Spawns multiple entities with some [`Component`](crate::component::Component)s, without checking that the given [`ComponentBundle`] is valid.
    ///
    /// This is more efficient than [`World::spawn`], but has the downside of only being able to spawn entities with the same [`ComponentBundle`] type.
    /// 
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ComponentBundle`] does not violate the archetype rules. See [`BundleValidator`](crate::component::bundle::BundleValidator).
    pub async unsafe fn spawn_batch_unchecked<B : ComponentBundle + 'static>(&self, bundles : impl IntoIterator<Item = B>) -> impl IntoIterator<Item = Entity> {
        unsafe{ self.archetypes.spawn_batch_unchecked::<B>(bundles).await }
    }

    /// Removes an entity.
    pub async fn despawn(&self, entity : Entity) {
        self.archetypes.despawn(entity).await
    }

    /// Removes an entity without checking that it exists.
    /// 
    /// # Safety
    /// You are responsible for ensuring that the given entity exists.
    pub async unsafe fn despawn_unchecked(&self, entity : Entity) {
        self.archetypes.despawn_unchecked(entity).await
    }


    /// TODO: Doc comments
    #[track_caller]
    pub async fn stateless_acquire_query<Q : ReadOnlyQuery + StatelessQuery>(&self) -> StatelessQueryItem<Q> {
        Q::validate().panic_on_violation();
        // SAFETY: TODO
        unsafe{ self.stateless_acquire_query_unchecked::<Q>() }.await
    }

    /// TODO: Doc comments
    pub async unsafe fn stateless_acquire_query_unchecked<Q : ReadOnlyQuery + StatelessQuery>(&self) -> StatelessQueryItem<Q> {
        // SAFETY: TODO
        unsafe{ StatelessQueryItem::<Q>::new(self) }.await
    }

    /// TODO: Doc comments
    #[track_caller]
    pub async fn stateless_acquire_query_mut<Q : StatelessQuery>(&self) -> StatelessQueryItem<Q> {
        Q::validate().panic_on_violation();
        // SAFETY: TODO
        unsafe{ self.stateless_acquire_query_unchecked_mut::<Q>() }.await
    }

    /// TODO: Doc comments
    pub async unsafe fn stateless_acquire_query_unchecked_mut<Q : StatelessQuery>(&self) -> StatelessQueryItem<Q> {
        // SAFETY: TODO
        unsafe{ StatelessQueryItem::<Q>::new(self) }.await
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn query<Q : ReadOnlyQuery>(&self) -> PersistentQueryState<'_, Q> {
        Q::validate().panic_on_violation();
        // SAFETY: TODO
        unsafe{ self.query_unchecked::<Q>() }
    }

    /// TODO: Doc comments
    pub unsafe fn query_unchecked<Q : ReadOnlyQuery>(&self) -> PersistentQueryState<'_, Q> {
        // SAFETY: TODO
        unsafe{ PersistentQueryState::<Q>::new(self) }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn query_mut<Q : Query>(&self) -> PersistentQueryState<'_, Q> {
        Q::validate().panic_on_violation();
        // SAFETY: TODO
        unsafe{ self.query_unchecked_mut::<Q>() }
    }

    /// TODO: Doc comments
    pub unsafe fn query_unchecked_mut<Q : Query>(&self) -> PersistentQueryState<'_, Q> {
        // SAFETY: TODO
        unsafe{ PersistentQueryState::<Q>::new(self) }
    }


    /// TODO: Doc comments
    #[track_caller]
    pub fn stateless_run_system<S : IntoReadOnlySystem<Params, Return> + IntoStatelessSystem<Params, Return>, Params, Return>(&self, system : S) -> ()
    where <S as IntoSystem<Params, Return>>::System : ReadOnlySystem<Return> + StatelessSystem<Return>
    {
        todo!()
    }

    /// TODO: Doc comments
    pub fn stateless_run_system_unchecked<S : IntoReadOnlySystem<Params, Return> + IntoStatelessSystem<Params, Return>, Params, Return>(&self, system : S) -> ()
    where <S as IntoSystem<Params, Return>>::System : ReadOnlySystem<Return> + StatelessSystem<Return>
    {
        todo!()
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn stateless_run_system_mut<S : IntoStatelessSystem<Params, Return>, Params, Return>(&self, system : S) -> ()
    where <S as IntoSystem<Params, Return>>::System : StatelessSystem<Return>
    {
        todo!()
    }

    /// TODO: Doc comments
    pub fn stateless_run_system_unchecked_mut<S : IntoStatelessSystem<Params, Return>, Params, Return>(&self, system : S) -> ()
    where <S as IntoSystem<Params, Return>>::System : StatelessSystem<Return>
    {
        todo!()
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn system<S : IntoReadOnlySystem<Params, Return>, Params, Return>(&self, system : S) -> PersistentSystemState<'_, S::System, Return>
    where <S as IntoSystem<Params, Return>>::System : ReadOnlySystem<Return>
    {
        unsafe{ PersistentSystemState::new(self, system.into_system()) }
    }

    /// TODO: Doc comments
    pub fn system_unchecked<S : IntoReadOnlySystem<Params, Return>, Params, Return>(&self, system : S) -> PersistentSystemState<'_, S::System, Return>
    where <S as IntoSystem<Params, Return>>::System : ReadOnlySystem<Return>
    {
        unsafe{ PersistentSystemState::new(self, system.into_system_unchecked()) }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn system_mut<S : IntoSystem<Params, Return>, Params, Return>(&self, system : S) -> PersistentSystemState<'_, S::System, Return> {
        unsafe{ PersistentSystemState::new(self, system.into_system()) }
    }

    /// TODO: Doc comments
    pub fn system_unchecked_mut<S : IntoSystem<Params, Return>, Params, Return>(&self, system : S) -> PersistentSystemState<'_, S::System, Return> {
        unsafe{ PersistentSystemState::new(self, system.into_system_unchecked()) }
    }

    /// TODO: Doc comments
    pub async unsafe fn run_erased_system<Passed, Return>(&self, system : &mut dyn TypeErasedSystem<Passed, Return>, passed : Passed) -> Return {
        unsafe{ system.acquire_and_run(passed, self) }.await
    }

}
