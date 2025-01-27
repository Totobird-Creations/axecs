//! TODO: Doc comments


mod commands;
pub use commands::*;


use crate::archetype::ArchetypeStorage;
use crate::entity::Entity;
use crate::component::ComponentBundle;
use crate::query::{ Query, ReadOnlyQuery, PersistentQueryState };


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
    pub fn query<Q : ReadOnlyQuery>(&self) -> PersistentQueryState<Q> {
        Q::validate().panic_on_violation();
        unsafe{ self.query_unchecked::<Q>() }
    }

    /// TODO: Doc comments
    pub unsafe fn query_unchecked<Q : ReadOnlyQuery>(&self) -> PersistentQueryState<Q> {
        unsafe{ PersistentQueryState::<Q>::new(self) }
    }

    /// TODO: Doc comments
    pub fn query_mut<Q : Query>(&self) -> PersistentQueryState<Q> {
        Q::validate().panic_on_violation();
        unsafe{ self.query_unchecked_mut::<Q>() }
    }

    /// TODO: Doc comments
    pub unsafe fn query_unchecked_mut<Q : Query>(&self) -> PersistentQueryState<Q> {
        unsafe{ PersistentQueryState::<Q>::new(self) }
    }

}
