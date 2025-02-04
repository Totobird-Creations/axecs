//! TODO: Doc comments


mod commands;
pub use commands::*;


use crate::resource::{ Resource, ResourceStorage, ResourceCellReadGuard, ResourceCellWriteGuard };
use crate::entity::Entity;
use crate::component::bundle::ComponentBundle;
use crate::component::archetype::ArchetypeStorage;
use crate::query::{ Query, ReadOnlyQuery, PersistentQueryState };
use crate::system::{ IntoSystem, IntoReadOnlySystem, ReadOnlySystem, PersistentSystemState };
use crate::app::AppExit;
use crate::schedule::system::TypeErasedSystem;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{ AtomicU8, Ordering };


/// TODO: Doc comments
pub struct World {

    /// TODO: Doc comments
    ///
    /// 0. Not exiting.
    /// 1. Exiting, status being written.
    /// 2. Exiting, status exists.
    /// 3. Exiting, status taken.
    is_exiting  : AtomicU8,

    /// TODO: Doc comments
    exit_status : UnsafeCell<MaybeUninit<AppExit>>,

    /// TODO: Doc comments
    resources   : ResourceStorage,

    /// TODO: Doc comments
    archetypes  : ArchetypeStorage

}

impl World {

    /// TODO: Doc comments
    #[inline]
    pub fn resources(&self) -> &ResourceStorage {
        &self.resources
    }

    /// TODO: Doc comments
    #[inline]
    pub fn archetypes(&self) -> &ArchetypeStorage {
        &self.archetypes
    }

}

impl World {

    /// TODO: Doc comments
    #[inline]
    pub fn new() -> Self { Self {
        is_exiting  : AtomicU8::new(0),
        exit_status : UnsafeCell::new(MaybeUninit::uninit()),
        resources   : ResourceStorage::new(),
        archetypes  : ArchetypeStorage::new()
    } }

    /// TODO: Doc comments
    #[inline]
    pub fn new_with(resources : ResourceStorage) -> Self { Self {
        is_exiting  : AtomicU8::new(0),
        exit_status : UnsafeCell::new(MaybeUninit::uninit()),
        resources,
        archetypes  : ArchetypeStorage::new()
    } }


    /// TODO: Doc comments
    pub fn is_exiting(&self) -> bool {
        self.is_exiting.load(Ordering::Relaxed) >= 2
    }

    /// TODO: Doc comments
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

    /// TODO: Doc comments
    pub fn exit(&self, status : AppExit) {
        match (self.is_exiting.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)) {
            Ok(_) => {
                // SAFETY: TODO
                unsafe{ (*self.exit_status.get()).write(status); }
                self.is_exiting.store(2, Ordering::Relaxed);
            },
            Err(_) => { panic!("Can not exit already exited app") }
        }
    }

    /// TODO: Doc comments
    pub fn try_exit(&self, status : AppExit) {
        match (self.is_exiting.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)) {
            Ok(_) => {
                // SAFETY: TODO
                unsafe{ (*self.exit_status.get()).write(status); }
                self.is_exiting.store(2, Ordering::Relaxed);
            },
            Err(_) => { }
        }
    }


    /// TODO: Doc comments
    pub async fn insert_resource<R : Resource + 'static>(&self, resource : R) {
        self.resources.insert::<R>(resource).await
    }

    /// TODO: Doc comments
    pub async fn replace_resource<R : Resource + 'static>(&self, resource : R) -> Option<R> {
        self.resources.replace::<R>(resource).await
    }

    /// TODO: Doc comments
    pub async fn remove_resource<R : Resource + 'static>(&self) {
        self.resources.remove::<R>().await
    }

    /// TODO: Doc comments
    pub async fn take_resource<R : Resource + 'static>(&self) -> Option<R> {
        self.resources.take::<R>().await
    }

    /// TODO: Doc comments
    pub async fn get_resource_ref<R : Resource + 'static>(&self) -> Option<ResourceCellReadGuard<'_, R>> {
        self.resources.get_ref::<R>().await
    }

    /// TODO: Doc comments
    pub async fn get_resource_mut<R : Resource + 'static>(&self) -> Option<ResourceCellWriteGuard<'_, R>> {
        self.resources.get_mut::<R>().await
    }


    /// TODO: Doc comments
    #[track_caller]
    pub async fn spawn<B : ComponentBundle + 'static>(&self, bundle : B) -> Entity {
        self.archetypes.spawn::<B>(bundle).await
    }

    /// TODO: Doc comments
    pub async unsafe fn spawn_unchecked<B : ComponentBundle + 'static>(&self, bundle : B) -> Entity {
        unsafe{ self.archetypes.spawn_unchecked::<B>(bundle).await }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub async fn spawn_batch<B : ComponentBundle + 'static>(&self, bundles : impl IntoIterator<Item = B>) -> impl IntoIterator<Item = Entity> {
        self.archetypes.spawn_batch::<B>(bundles).await
    }

    /// TODO: Doc comments
    pub async unsafe fn spawn_batch_unchecked<B : ComponentBundle + 'static>(&self, bundles : impl IntoIterator<Item = B>) -> impl IntoIterator<Item = Entity> {
        unsafe{ self.archetypes.spawn_batch_unchecked::<B>(bundles).await }
    }

    /// TODO: Doc comments
    pub async fn despawn(&self, entity : Entity) {
        self.archetypes.despawn(entity).await
    }

    /// TODO: Doc comments
    pub async unsafe fn despawn_unchecked(&self, entity : Entity) {
        self.archetypes.despawn_unchecked(entity).await
    }


    /// TODO: Doc comments
    #[track_caller]
    pub fn query<Q : ReadOnlyQuery>(&self) -> PersistentQueryState<'_, Q> {
        Q::validate().panic_on_violation();
        unsafe{ self.query_unchecked::<Q>() }
    }

    /// TODO: Doc comments
    pub unsafe fn query_unchecked<Q : ReadOnlyQuery>(&self) -> PersistentQueryState<'_, Q> {
        unsafe{ PersistentQueryState::<Q>::new(self) }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn query_mut<Q : Query>(&self) -> PersistentQueryState<'_, Q> {
        Q::validate().panic_on_violation();
        unsafe{ self.query_unchecked_mut::<Q>() }
    }

    /// TODO: Doc comments
    pub unsafe fn query_unchecked_mut<Q : Query>(&self) -> PersistentQueryState<'_, Q> {
        unsafe{ PersistentQueryState::<Q>::new(self) }
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
