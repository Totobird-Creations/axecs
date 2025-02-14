//! TODO: Doc comment


use crate::world::World;
use crate::query::{ Query, ReadOnlyQuery, StatelessQuery, QueryAcquireFuture };
use crate::system::{ System, ReadOnlySystem, StatelessSystem, IntoSystem, IntoReadOnlySystem, IntoStatelessSystem, SystemPassable };
use crate::util::future::multijoin;
use crate::util::variadic::variadic;
use core::ops::AsyncFnMut;
use core::marker::PhantomData;


variadic!{ impl_into_system_for_f }
macro impl_into_system_for_f( $( #[$meta:meta] )* $( $generic:ident ),* $(,)? ) {

    #[allow(non_snake_case, unused_variables)]
    $( #[ $meta ] )*
    impl< F, $( $generic : Query , )* Return >
        IntoSystem<( (), $( $generic , )* ), Return>
        for F
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( $( $generic , )* ) -> Return) +
        (AsyncFnMut( $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    {
        type System = FunctionSystem<Self, (), ( $( <$generic as Query>::State , )* ), ( $( $generic , )* ), Return>;

        #[track_caller]
        fn into_system(self) -> Self::System {
            <( $( $generic , )* )>::validate().panic_on_violation();
            $( let $generic = <$generic as Query>::init_state(); )*
            FunctionSystem {
                function     : self,
                query_states : ( $( $generic , )* ),
                marker       : PhantomData
            }
        }

        #[track_caller]
        unsafe fn into_system_unchecked(self) -> Self::System {
            $( let $generic = <$generic as Query>::init_state(); )*
            FunctionSystem {
                function     : self,
                query_states : ( $( $generic , )* ),
                marker       : PhantomData
            }
        }

    }

    $( #[ $meta ] )*
    unsafe impl< F, $( $generic : ReadOnlyQuery , )* Return >
        IntoReadOnlySystem<( (), $( $generic , )* ), Return>
        for F
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( $( $generic , )* ) -> Return) +
        (AsyncFnMut( $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    { }

    $( #[ $meta ] )*
    unsafe impl< F, $( $generic : StatelessQuery , )* Return >
        IntoStatelessSystem<( (), $( $generic , )* ), Return>
        for F
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( $( $generic , )* ) -> Return) +
        (AsyncFnMut( $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    { }


    #[allow(non_snake_case, unused_variables)]
    $( #[ $meta ] )*
    impl< F, Passed : SystemPassable, $( $generic : Query , )* Return >
        IntoSystem<( Passed, $( $generic , )* ), Return>
        for F
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( Passed, $( $generic , )* ) -> Return) +
        (AsyncFnMut( Passed, $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    {
        type System = FunctionSystem<Self, Passed, ( $( <$generic as Query>::State , )* ), ( $( $generic , )* ), Return>;

        #[track_caller]
        fn into_system(self) -> Self::System {
            <( $( $generic , )* )>::validate().panic_on_violation();
            $( let $generic = <$generic as Query>::init_state(); )*
            FunctionSystem {
                function     : self,
                query_states : ( $( $generic , )* ),
                marker       : PhantomData
            }
        }

        #[track_caller]
        unsafe fn into_system_unchecked(self) -> Self::System {
            $( let $generic = <$generic as Query>::init_state(); )*
            FunctionSystem {
                function     : self,
                query_states : ( $( $generic , )* ),
                marker       : PhantomData
            }
        }

    }

    $( #[ $meta ] )*
    unsafe impl< F, Passed : SystemPassable, $( $generic : ReadOnlyQuery , )* Return >
        IntoReadOnlySystem<( Passed, $( $generic , )* ), Return>
        for F
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( Passed, $( $generic , )* ) -> Return) +
        (AsyncFnMut( Passed, $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    { }

    $( #[ $meta ] )*
    unsafe impl< F, Passed : SystemPassable, $( $generic : StatelessQuery , )* Return >
        IntoStatelessSystem<( Passed, $( $generic , )* ), Return>
        for F
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( Passed, $( $generic , )* ) -> Return) +
        (AsyncFnMut( Passed, $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    { }

}


/// TODO: Doc comment
pub struct FunctionSystem<F, Passed, Q, Params, Return> {

    /// TODO: Doc comment
    function     : F,

    /// TODO: Doc comment
    query_states : Q,

    /// TODO: Doc comment
    marker       : PhantomData<fn(Passed, Params) -> Return>

}


variadic!{ impl_system_for_function_system }
macro impl_system_for_function_system( $( #[$meta:meta] )* $( $generic:ident ),* $(,)? ) {

    #[allow(non_snake_case, unused_variables)]
    $( #[ $meta ] )*
    impl< F, $( $generic : Query , )* Return >
        System<Return>
        for FunctionSystem<F, (), ( $( <$generic as Query>::State , )* ), ( $( $generic , )* ), Return>
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( $( $generic , )* ) -> Return) +
        (AsyncFnMut( $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    {
        #[track_caller]
        async unsafe fn acquire_and_run(&mut self, passed : Self::Passed, world : &World) -> Return {
            #[inline]
            async fn run_inner< $( $generic , )* Return >(
                mut func : impl AsyncFnMut( $( $generic , )* ) -> Return,
                $( $generic : $generic , )* )
                -> Return
            {
                func( $( $generic , )* ).await
            }
            // SAFETY: TODO
            $( let $generic = unsafe{ QueryAcquireFuture::<$generic>::new(world, &mut self.query_states.${index()}) }; )*
            let ( $( $generic , )* ) = multijoin!( $( $generic , )* );
            run_inner::< $( $generic::Item<'_, '_> , )* Return >( &mut self.function $( , $generic.unwrap() )* ).await
        }
    }

    $( #[ $meta ] )*
    unsafe impl< F, $( $generic : ReadOnlyQuery , )* Return >
        ReadOnlySystem<Return>
        for FunctionSystem<F, (), ( $( <$generic as Query>::State , )* ), ( $( $generic , )* ), Return>
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( $( $generic , )* ) -> Return) +
        (AsyncFnMut( $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    { }

    $( #[ $meta ] )*
    unsafe impl< F, $( $generic : StatelessQuery , )* Return >
        StatelessSystem<Return>
        for FunctionSystem<F, (), ( $( <$generic as Query>::State , )* ), ( $( $generic , )* ), Return>
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( $( $generic , )* ) -> Return) +
        (AsyncFnMut( $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    { }


    #[allow(non_snake_case, unused_variables)]
    $( #[ $meta ] )*
    impl< F, Passed : SystemPassable, $( $generic : Query , )* Return >
        System<Return>
        for FunctionSystem<F, Passed, ( $( <$generic as Query>::State , )* ), ( $( $generic , )* ), Return>
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( Passed, $( $generic , )* ) -> Return) +
        (AsyncFnMut( Passed, $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    {
        type Passed = Passed;

        #[track_caller]
        async unsafe fn acquire_and_run(&mut self, passed : Self::Passed, world : &World) -> Return {
            #[inline]
            async fn run_inner< Passed : SystemPassable, $( $generic , )* Return >(
                mut func   : impl AsyncFnMut( Passed, $( $generic , )* ) -> Return,
                    passed : Passed,
                $( $generic : $generic , )* )
                -> Return
            {
                func( passed, $( $generic , )* ).await
            }
            // SAFETY: TODO
            $( let $generic = unsafe{ QueryAcquireFuture::<$generic>::new(world, &mut self.query_states.${index()}) }; )*
            let ( $( $generic , )* ) = multijoin!( $( $generic , )* );
            run_inner::< Passed, $( $generic::Item<'_, '_> , )* Return >( &mut self.function, passed $( , $generic.unwrap() )* ).await
        }
    }

    $( #[ $meta ] )*
    unsafe impl< F, Passed : SystemPassable, $( $generic : ReadOnlyQuery , )* Return >
        ReadOnlySystem<Return>
        for FunctionSystem<F, Passed, ( $( <$generic as Query>::State , )* ), ( $( $generic , )* ), Return>
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( Passed, $( $generic , )* ) -> Return) +
        (AsyncFnMut( Passed, $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    { }

    $( #[ $meta ] )*
    unsafe impl< F, Passed : SystemPassable, $( $generic : StatelessQuery , )* Return >
        StatelessSystem<Return>
        for FunctionSystem<F, Passed, ( $( <$generic as Query>::State , )* ), ( $( $generic , )* ), Return>
    where for<'l, 'world, 'state> &'l mut F:
        (AsyncFnMut( Passed, $( $generic , )* ) -> Return) +
        (AsyncFnMut( Passed, $( <$generic as Query>::Item<'world, 'state> , )* ) -> Return)
    { }

}
