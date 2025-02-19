//! TODO: Doc comment


use crate::world::World;
use crate::system::{ SystemId, System, ReadOnlySystem, IntoSystem, IntoReadOnlySystem, In };
use core::marker::PhantomData;
use alloc::sync::Arc;


/// TODO: Doc comment
pub struct IntoPassedSystem<Passed, S, Params, Return>
where   S         : IntoSystem<Params, Return>,
        S::System : System<Return, Passed = In<Passed>>,
        Passed    : Clone
{

    /// TODO: Doc comment
    pub(super) system : S,

    /// TODO: Doc comment
    pub(super) passed : Passed,

    /// TODO: Doc comment
    pub(super) marker : PhantomData<fn(Passed, Params) -> Return>

}


impl<Passed, S, Params, Return>
    IntoSystem<(), Return>
    for IntoPassedSystem<Passed, S, Params, Return>
where   S         : IntoSystem<Params, Return>,
        S::System : System<Return, Passed = In<Passed>>,
        Passed    : Clone
{

    type System = PassedSystem<Passed, S::System, Return>;

    #[track_caller]
    fn into_system(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System {
        PassedSystem {
            system : self.system.into_system(world, system_id),
            passed : self.passed,
            marker : PhantomData
        }
    }

    #[track_caller]
    unsafe fn into_system_unchecked(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System {
        PassedSystem {
            system : unsafe{ self.system.into_system_unchecked(world, system_id) },
            passed : self.passed,
            marker : PhantomData
        }
    }

}


unsafe impl<Passed, S, Params, Return>
    IntoReadOnlySystem<(), Return>
    for IntoPassedSystem<Passed, S, Params, Return>
where   S         : IntoReadOnlySystem<Params, Return>,
        S::System : ReadOnlySystem<Return, Passed = In<Passed>>,
        Passed    : Clone
{ }


/// TODO: Doc comment
pub struct PassedSystem<Passed, S, Return>
where   S : System<Return, Passed = In<Passed>>
{

    /// TODO: Doc comment
    system : S,

    /// TODO: Doc comment
    passed : Passed,

    /// TODO: Doc comment
    marker : PhantomData<fn(Passed) -> Return>

}


impl<Passed, S, Return>
    System<Return>
    for PassedSystem<Passed, S, Return>
where   S      : System<Return, Passed = In<Passed>>,
        Passed : Clone
{

    type Passed = ();

    async unsafe fn acquire_and_run(&mut self, _ : Self::Passed, world : Arc<World>) -> Return {
        unsafe{ self.system.acquire_and_run(In(self.passed.clone()), world) }.await
    }

}


unsafe impl<Passed, S, Return>
    ReadOnlySystem<Return>
    for PassedSystem<Passed, S, Return>
where   S      : ReadOnlySystem<Return, Passed = In<Passed>>,
        Passed : Clone
{ }
