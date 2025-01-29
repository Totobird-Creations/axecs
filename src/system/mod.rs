//! TODO: Doc comment


mod function;
pub use function::*;

mod piped;
pub use piped::*;

mod mapped;
pub use mapped::*;

mod state;
pub use state::*;

mod param;
pub use param::*;


use crate::world::World;
use core::ops::AsyncFnMut;
use core::marker::PhantomData;
use alloc::boxed::Box;


/// TODO: Doc comment
pub trait System<Return> {

    /// TODO: Doc comment
    type Passed = ();

    /// TODO: Doc comment
    async unsafe fn acquire_and_run(&mut self, passed : Self::Passed, world : &World) -> Return;

}


/// TODO: Doc comment
pub unsafe trait ReadOnlySystem<Return> : System<Return> { }


/// TODO: Doc comment
pub trait IntoSystem<Params, Return> : Sized {

    /// TODO: Doc comment
    type System : System<Return>;

    /// TODO: Doc comment
    fn into_system(self, world : &World) -> Self::System;

    /// TODO: Doc comment
    unsafe fn into_system_unchecked(self, world : &World) -> Self::System;

    /// TODO: Doc comment
    fn pipe<'l, B, BParams, BReturn>(self, into : B)
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
    fn map<'l, B, BParams, BReturn>(self, with : B)
        -> IntoMappedSystem<<Self::System as System<Return>>::Passed, Self, Params, Return, B, BReturn>
    where   B            : AsyncFnMut(Return) -> BReturn,
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
pub unsafe trait IntoReadOnlySystem<Params, Return> : IntoSystem<Params, Return>
where <Self as IntoSystem<Params, Return>>::System : ReadOnlySystem<Return>
{ }


/// TODO: Doc comment
pub trait SystemPassable { }


/// TODO: Doc comment
pub unsafe trait TypeEraseableSystem<Passed : SystemPassable, Return> {

    /// TODO: Doc comment
    unsafe fn acquire_and_run<'l>(&'l mut self, passed : Passed, world : &'l World) -> Box<dyn Future<Output = Return> + 'l>;

}

unsafe impl<S : System<Return, Passed = Passed>, Passed : SystemPassable, Return : 'static> TypeEraseableSystem<Passed, Return> for S {
    unsafe fn acquire_and_run<'l>(&'l mut self, passed : Passed, world : &'l World) -> Box<dyn Future<Output = Return> + 'l> {
        // SAFETY: TODO
        Box::new(unsafe{ <Self as System<_>>::acquire_and_run(self, passed, world) })
    }
}



#[cfg(test)]
mod tests {
    use crate::entity::Entities;
    use super::*;

    fn require_into_schedulable_system<S : IntoSystem<Params, ()>, Params>(_system : S)
        where S::System : System<(), Passed = ()>
    { }

    async fn simple_system( _q : Entities<'_, ()> ) -> () { }

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
