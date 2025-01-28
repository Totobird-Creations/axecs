//! TODO: Doc comments


mod commands;
pub use commands::*;


use crate::archetype::ArchetypeStorage;
use crate::entity::Entity;
use crate::component::ComponentBundle;
use crate::query::{ Query, ReadOnlyQuery, PersistentQueryState };
use crate::system::{ IntoSystem, IntoReadOnlySystem, ReadOnlySystem, PersistentSystemState };


/// TODO: Doc comments
pub struct World {

    /// TODO: Doc comments
    archetypes : ArchetypeStorage

}

impl World {

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
        archetypes : ArchetypeStorage::new()
    } }


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
    pub async fn query<Q : ReadOnlyQuery>(&self) -> PersistentQueryState<'_, Q> {
        Q::validate().panic_on_violation();
        unsafe{ self.query_unchecked::<Q>() }.await
    }

    /// TODO: Doc comments
    pub async unsafe fn query_unchecked<Q : ReadOnlyQuery>(&self) -> PersistentQueryState<'_, Q> {
        unsafe{ PersistentQueryState::<Q>::new(self) }.await
    }

    /// TODO: Doc comments
    #[track_caller]
    pub async fn query_mut<Q : Query>(&self) -> PersistentQueryState<'_, Q> {
        Q::validate().panic_on_violation();
        unsafe{ self.query_unchecked_mut::<Q>() }.await
    }

    /// TODO: Doc comments
    pub async unsafe fn query_unchecked_mut<Q : Query>(&self) -> PersistentQueryState<'_, Q> {
        unsafe{ PersistentQueryState::<Q>::new(self) }.await
    }


    /// TODO: Doc comments
    #[track_caller]
    pub async fn system<S : IntoReadOnlySystem<Params, Return>, Params, Return>(&self, system : S) -> PersistentSystemState<'_, S::System, Return>
    where <S as IntoSystem<Params, Return>>::System : ReadOnlySystem<Return>
    {
        unsafe{ PersistentSystemState::new(self, system.into_system(self).await) }
    }

    /// TODO: Doc comments
    pub async fn system_unchecked<S : IntoReadOnlySystem<Params, Return>, Params, Return>(&self, system : S) -> PersistentSystemState<'_, S::System, Return>
    where <S as IntoSystem<Params, Return>>::System : ReadOnlySystem<Return>
    {
        unsafe{ PersistentSystemState::new(self, system.into_system_unchecked(self).await) }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub async fn system_mut<S : IntoSystem<Params, Return>, Params, Return>(&self, system : S) -> PersistentSystemState<'_, S::System, Return> {
        unsafe{ PersistentSystemState::new(self, system.into_system(self).await) }
    }

    /// TODO: Doc comments
    pub async fn system_unchecked_mut<S : IntoSystem<Params, Return>, Params, Return>(&self, system : S) -> PersistentSystemState<'_, S::System, Return> {
        unsafe{ PersistentSystemState::new(self, system.into_system_unchecked(self).await) }
    }

}
