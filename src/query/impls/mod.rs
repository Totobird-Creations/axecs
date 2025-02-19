//! TODO: Doc comments


mod scoped;
pub use scoped::*;


use crate::world::World;
use crate::system::SystemId;
use crate::query::{ Query, ReadOnlyQuery, QueryAcquireResult, QueryValidator };
use crate::util::variadic::variadic_no_unit;
use core::task::Poll;
use alloc::sync::Arc;


unsafe impl Query for () {
    type Item = ();

    fn init_state(_world : Arc<World>, _system_id : Option<SystemId>) -> Self::State { () }

    unsafe fn acquire(_world : Arc<World>, _state : &mut Self::State) -> Poll<QueryAcquireResult<Self::Item>> {
        Poll::Ready(QueryAcquireResult::Ready(()))
    }

    fn validate() -> QueryValidator {
        QueryValidator::empty()
    }

}
unsafe impl ReadOnlyQuery for () { }


unsafe impl<Q : Query> Query for Option<Q> {
    type Item = Option<<Q as Query>::Item>;
    type State = <Q as Query>::State;

    fn init_state(world : Arc<World>, system_id : Option<SystemId>) -> Self::State {
        <Q as Query>::init_state(world, system_id)
    }

    unsafe fn acquire(world : Arc<World>, state : &mut Self::State) -> Poll<QueryAcquireResult<Self::Item>> {
        // SAFETY: TODO
        match (unsafe{ <Q as Query>::acquire(world, state) }) {
            Poll::Ready(QueryAcquireResult::Ready(out))          => Poll::Ready(QueryAcquireResult::Ready(Some(out))),
            Poll::Ready(QueryAcquireResult::DoesNotExist { .. }) => Poll::Ready(QueryAcquireResult::Ready(None)),
            Poll::Pending                                        => Poll::Pending
        }
    }

    fn validate() -> QueryValidator {
        Q::validate()
    }

}
unsafe impl<Q : ReadOnlyQuery> ReadOnlyQuery for Option<Q> { }


variadic_no_unit!{ #[doc(fake_variadic)] impl_query_for_tuple }
/// TODO: Doc comments
macro impl_query_for_tuple( $( #[$meta:meta] )* $( $generic:ident ),* $(,)? ) {

    #[allow(non_snake_case)]
    $( #[ $meta ] )*
    unsafe impl< $( $generic : Query ),* > Query for ( $( $generic , )* ) {
        type Item = ( $( <$generic as Query>::Item , )* );
        type State = ( $( <$generic as Query>::State , )* );

        fn init_state(world : Arc<World>, system_id : Option<SystemId>) -> Self::State {
            $( let $generic = <$generic as Query>::init_state(Arc::clone(&world), system_id); )*
            ( $( $generic , )* )
        }

        unsafe fn acquire(world : Arc<World>, state : &mut Self::State) -> Poll<QueryAcquireResult<Self::Item>> {
            $(
                // SAFETY: TODO
                let $generic = match (unsafe{ <$generic as Query>::acquire(Arc::clone(&world), &mut state.${index()}) }) {
                    Poll::Ready(QueryAcquireResult::Ready(out))            => out,
                    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                    Poll::Ready(QueryAcquireResult::DoesNotExist { name }) => { return Poll::Ready(QueryAcquireResult::DoesNotExist { name }); },
                    #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
                    Poll::Ready(QueryAcquireResult::DoesNotExist { })      => { return Poll::Ready(QueryAcquireResult::DoesNotExist {  }); },
                    Poll::Pending                                          => { return Poll::Pending; }
                };
            )*
            Poll::Ready(QueryAcquireResult::Ready(( $( $generic , )* )))
        }

        fn validate() -> QueryValidator {
            let mut qv = QueryValidator::empty();
            $( qv = QueryValidator::join(qv, <$generic as Query>::validate()); )*
            qv
        }

    }

    $( #[ $meta ] )*
    unsafe impl< $( $generic : ReadOnlyQuery ),* > ReadOnlyQuery for ( $( $generic , )* ) { }

}
