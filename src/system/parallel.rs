//! TODO: Doc comment


use crate::world::World;
use crate::system::{ SystemId, System, ReadOnlySystem, IntoSystem, IntoReadOnlySystem };
//use crate::util::variadic::variadic_no_unit;
use core::marker::PhantomData;
use core::future::join;
use alloc::sync::Arc;
use paste::paste;


pub struct IntoParallelSystem<A, AParams, B, BParams>
where   A         : IntoSystem<AParams, ()>,
        B         : IntoSystem<BParams, ()>,
        A::System : System<(), Passed = ()>,
        B::System : System<(), Passed = ()>
{

    /// TODO: Doc comment
    pub(super) a        : A,

    /// TODO: Doc comment
    pub(super) b        : B,

    /// TODO: Doc comment
    pub(super) marker_a : PhantomData<fn(AParams) -> ()>,

    /// TODO: Doc comment
    pub(super) marker_b : PhantomData<fn(BParams) -> ()>

}

impl<A, AParams, B, BParams>
    IntoSystem<(), ()>
    for IntoParallelSystem<A, AParams, B, BParams>
where   A         : IntoSystem<AParams, ()>,
        B         : IntoSystem<BParams, ()>,
        A::System : System<(), Passed = ()>,
        B::System : System<(), Passed = ()>
{
    type System = ParallelSystem<A::System, B::System>;

    #[track_caller]
    fn into_system(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System {
        ParallelSystem {
            a : self.a.into_system(Arc::clone(&world), system_id),
            b : self.b.into_system(world, system_id)
        }
    }

    #[track_caller]
    unsafe fn into_system_unchecked(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System {
        ParallelSystem {
            // SAFETY: TODO
            a : unsafe{ self.a.into_system_unchecked(Arc::clone(&world), system_id) },
            // SAFETY: TODO
            b : unsafe{ self.b.into_system_unchecked(world, system_id) }
        }
    }
}

unsafe impl<A, AParams, B, BParams>
    IntoReadOnlySystem<(), ()>
    for IntoParallelSystem<A, AParams, B, BParams>
where   A         : IntoReadOnlySystem<AParams, ()>,
        B         : IntoReadOnlySystem<BParams, ()>,
        A::System : ReadOnlySystem<(), Passed = ()>,
        B::System : ReadOnlySystem<(), Passed = ()>
{ }


/// TODO: Doc comment
pub struct ParallelSystem<A, B>
where   A : System<(), Passed = ()>,
        B : System<(), Passed = ()>
{

    /// TODO: Doc comment
    a : A,

    /// TODO: Doc comment
    b : B

}

impl<A, B>
    System<()>
    for ParallelSystem<A, B>
where   A : System<(), Passed = ()>,
        B : System<(), Passed = ()>
{
    type Passed = ();

    #[track_caller]
    async unsafe fn acquire_and_run(&mut self, a_passed : Self::Passed, world : Arc<World>) -> () {
        // SAFETY: TODO
        let a = unsafe{ self.a.acquire_and_run(a_passed, Arc::clone(&world)) };
        // SAFETY: TODO
        let b = unsafe{ self.b.acquire_and_run((), world) };

        join!(a, b).await;
    }
}

unsafe impl<A, B>
    ReadOnlySystem<()>
    for ParallelSystem<A, B>
where   A : ReadOnlySystem<(), Passed = ()>,
        B : ReadOnlySystem<(), Passed = ()>
{ }


// TODO: Somehow bring back tuples for parallel systems. Damn you Rust 1.85.0.
/*variadic_no_unit!{ #[doc(fake_variadic)] impl_into_system_for_tuple }
/// TODO: Doc comments
macro impl_into_system_for_tuple( $( #[$meta:meta] )* $( $generic:ident ),* $(,)? ) { paste!{

    #[allow(non_snake_case)]
    $( #[ $meta ] )*
    impl< $( $generic , [<$generic Params>] , [<$generic System>] , )* > IntoSystem<( $( [<$generic Params>] , )* ), ()> for ( $( $generic , )* )
    where $(
        $generic : IntoSystem<[<$generic Params>], (), System = [<$generic System>]> ,
        [<$generic System>] : System<(), Passed = ()> ,
    )*
    {
        type System = impl System<(), Passed = ()>;

        #[track_caller]
        fn into_system(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System {
            IntoSystem::into_system(
                impl_into_system_for_tuple_inner!( $( self.${index()} ${ignore($generic)} , )* ),
                world, system_id
            )
        }

        #[track_caller]
        unsafe fn into_system_unchecked(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System {
            // SAFETY: TODO
            unsafe{ IntoSystem::into_system_unchecked(
                impl_into_system_for_tuple_inner!( $( self.${index()} ${ignore($generic)} , )* ),
                world, system_id
            ) }
        }
    }

    // TODO: IntoReadOnlySystem

} }
/// TODO: Doc comments
macro impl_into_system_for_tuple_inner {

    ( $entry:expr $(,)? ) => {
        $entry
    },

    ( $first:expr , $second:expr $( , $remaining:expr )* $(,)? ) => {
        impl_into_system_for_tuple_inner!(
            IntoParallelSystem {
                a : impl_into_system_for_tuple_inner!( $first  ),
                b : impl_into_system_for_tuple_inner!( $second ),
                marker_a : PhantomData, marker_b : PhantomData
            }
            $( , $remaining )*
        )
    }

}*/
