//! TODO: Doc comment


mod function;
pub use function::*;

mod piped;
pub use piped::*;

mod mapped;
pub use mapped::*;

mod series;
pub use series::*;

mod parallel;
pub use parallel::*;

mod passed;
pub use passed::*;

mod state;
pub use state::*;


use crate::world::World;
use core::marker::PhantomData;
use core::sync::atomic::{ AtomicU64, Ordering as AtomicOrdering };
use alloc::sync::Arc;


/// TODO: Doc comment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemId(u64);
impl SystemId {

    /// TODO: Doc comment
    pub(crate) fn unique() -> Self {
        Self(NEXT_SYSTEM_ID.fetch_add(1, AtomicOrdering::Relaxed))
    }

}

/// TODO: Doc comment
pub(crate) static NEXT_SYSTEM_ID : AtomicU64 = AtomicU64::new(0);


/// TODO: Doc comment
pub trait System<Return> {

    /// TODO: Doc comment
    type Passed = ();

    /// TODO: Doc comment
    async unsafe fn acquire_and_run(&mut self, passed : Self::Passed, world : Arc<World>) -> Return;

}


/// TODO: Doc comment
pub unsafe trait ReadOnlySystem<Return> : System<Return> { }


/// TODO: Doc comment
pub trait IntoSystem<Params, Return> : Sized {

    /// TODO: Doc comment
    type System : System<Return>;

    /// TODO: Doc comment
    fn into_system(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System;

    /// TODO: Doc comment
    unsafe fn into_system_unchecked(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System;

    /// TODO: Doc comment
    fn pipe<B, BParams, BReturn>(self, into : B)
        -> IntoPipedSystem<<Self::System as System<Return>>::Passed, Self, Params, Return, B, BParams, BReturn>
    where   B            : IntoSystem<BParams, BReturn>,
            Self::System : System<Return>,
            B::System    : System<BReturn, Passed = In<Return>>
    {
        IntoPipedSystem {
            a : self,
            b : into,
            marker_a : PhantomData,
            marker_b : PhantomData
        }
    }

    /// TODO: Doc comment
    fn map<B, BParams, BReturn>(self, with : B)
        -> IntoMappedSystem<<Self::System as System<Return>>::Passed, Self, Params, Return, B, BReturn>
    where   B            : FnMut(Return) -> BReturn,
            Self::System : System<Return>
    {
        IntoMappedSystem {
            a : self,
            b : with,
            marker_a : PhantomData,
            marker_b : PhantomData
        }
    }

}


/// TODO: Doc comment
pub trait IntoSystemPassable<Passed : Clone, Params, Return> : Sized
where   Self         : IntoSystem<Params, Return>,
        Self::System : System<Return, Passed = In<Passed>>
{

    /// TODO: Doc comment
    fn pass(self, passed : Passed) -> IntoPassedSystem<Passed, Self, Params, Return>
    {
        IntoPassedSystem {
            system : self,
            passed,
            marker : PhantomData
        }
    }

}

impl<Passed, S : IntoSystem<Params, Return>, Params, Return> IntoSystemPassable<Passed, Params, Return> for S
where   S::System : System<Return, Passed = In<Passed>>,
        Passed    : Clone
{ }


/// TODO: Doc comment
pub trait IntoUnitSystem<Params> : IntoSystem<Params, ()> {

    /// TODO: Doc comment
    fn then<B, BParams, BReturn>(self, then : B)
        -> IntoSeriesSystem<<Self::System as System<()>>::Passed, Self, Params, B, BParams, BReturn>
    where   B            : IntoSystem<BParams, BReturn>,
            Self::System : System<()>,
            B::System    : System<BReturn, Passed = ()>
    {
        IntoSeriesSystem {
            a : self,
            b : then,
            marker_a : PhantomData,
            marker_b : PhantomData
        }
    }

}

impl<S : IntoSystem<Params, ()>, Params> IntoUnitSystem<Params> for S { }


/// TODO: Doc comment
pub trait IntoBoolSystem<Params> : IntoSystem<Params, bool> {

    // TODO: not

    // TODO: and

    // TODO: nand

    // TODO: or

    // TODO: nor

    // TODO: xor

    // TODO: xnor

}

impl<S : IntoSystem<Params, bool>, Params> IntoBoolSystem<Params> for S { }


/// TODO: Doc comment
pub unsafe trait IntoReadOnlySystem<Params, Return> : IntoSystem<Params, Return>
where <Self as IntoSystem<Params, Return>>::System : ReadOnlySystem<Return>
{ }


/// TODO: Doc comment
pub trait SystemPassable { }



#[cfg(test)]
mod tests {
    use crate::entity::Entities;
    use super::*;

    fn require_into_schedulable_system<S : IntoSystem<Params, ()>, Params>(_system : S)
        where S::System : System<(), Passed = ()>
    { }

    async fn simple_system( _q : Entities<()> ) -> () { }

    async fn system_no_params( ) -> () { }

    async fn system_returns_usize( ) -> usize { 123 }

    async fn system_takes_usize( _input : In<usize> ) -> () { }


    #[test]
    fn test_system_impl() {

        require_into_schedulable_system(simple_system);

        require_into_schedulable_system(system_no_params);

        // require_into_schedulable_system(system_returns_usize); // Will not compile.

        require_into_schedulable_system(system_returns_usize.pipe(system_takes_usize));

    }

}
