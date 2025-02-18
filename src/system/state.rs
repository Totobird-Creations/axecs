//! TODO: Doc comment


use crate::world::World;
use crate::system::System;
use core::marker::PhantomData;
use alloc::sync::Arc;


/// TODO: Doc comments
pub struct PersistentSystemState<S : System<Return>, Return> {

    /// TODO: Doc comments
    world : Arc<World>,

    /// TODO: Doc comments
    system : S,

    /// TODO: Doc comments
    marker : PhantomData<fn() -> Return>

}

impl<S : System<Return>, Return> PersistentSystemState<S, Return> {

    /// TODO: Doc comments
    pub(crate) unsafe fn new(world : Arc<World>, system : S) -> Self {
        Self { world, system, marker : PhantomData }
    }

}

impl<S : System<Return>, Return> PersistentSystemState<S, Return> {

    /// TODO: Doc comments
    #[track_caller]
    pub async fn run_with(&mut self, passed : S::Passed) -> Return {
        unsafe{ self.system.acquire_and_run(passed, Arc::clone(&self.world)) }.await
    }

}

impl<S : System<Return, Passed = ()>, Return> PersistentSystemState<S, Return> {

    /// TODO: Doc comments
    #[track_caller]
    pub async fn run(&mut self) -> Return {
        unsafe{ self.system.acquire_and_run((), Arc::clone(&self.world)) }.await
    }

}
