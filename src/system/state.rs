//! TODO: Doc comment


use crate::world::World;
use crate::system::System;
use core::marker::PhantomData;


/// TODO: Doc comments
pub struct PersistentSystemState<'l, S : System<Return>, Return> {

    /// TODO: Doc comments
    world : &'l World,

    /// TODO: Doc comments
    system : S,

    /// TODO: Doc comments
    marker : PhantomData<fn() -> Return>

}

impl<'l, S : System<Return>, Return> PersistentSystemState<'l, S, Return> {

    /// TODO: Doc comments
    pub(crate) unsafe fn new(world : &'l World, system : S) -> Self {
        Self { world, system, marker : PhantomData }
    }

}

impl<'l, S : System<Return>, Return> PersistentSystemState<'l, S, Return> {

    /// TODO: Doc comments
    #[track_caller]
    pub async fn run_with(&mut self, passed : S::Passed) -> Return {
        unsafe{ self.system.acquire_and_run(passed, self.world) }.await
    }

}

impl<'l, S : System<Return, Passed = ()>, Return> PersistentSystemState<'l, S, Return> {

    /// TODO: Doc comments
    #[track_caller]
    pub async fn run(&mut self) -> Return {
        unsafe{ self.system.acquire_and_run((), self.world) }.await
    }

}
