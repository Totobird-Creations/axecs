//! TODO: Doc comments


use crate::world::World;
use crate::query::{ Query, ReadOnlyQuery, QueryAcquireResult, QueryValidator };
use crate::util::future::multijoin;
use crate::util::variadic::variadic_no_unit;
use core::task::Poll;


unsafe impl Query for () {
    type Item<'world, 'state> = ();

    async fn init_state<'world>(_world : &'world World) -> Self::State { () }

    unsafe fn acquire<'world, 'state>(_world : &'world World, _state : &'state mut Self::State) -> Poll<QueryAcquireResult<Self::Item<'world, 'state>>> {
        Poll::Ready(QueryAcquireResult::Ready(()))
    }

    fn validate() -> QueryValidator {
        QueryValidator::empty()
    }

}
unsafe impl ReadOnlyQuery for () { }


unsafe impl<Q : Query> Query for Option<Q> {
    type Item<'world, 'state> = Option<<Q as Query>::Item<'world, 'state>>;
    type State = <Q as Query>::State;

    async fn init_state<'world>(world : &'world World) -> Self::State {
        <Q as Query>::init_state(world).await
    }

    unsafe fn acquire<'world, 'state>(world : &'world World, state : &'state mut Self::State) -> Poll<QueryAcquireResult<Self::Item<'world, 'state>>> {
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
        type Item<'world, 'state> = ( $( <$generic as Query>::Item<'world, 'state> , )* );
        type State = ( $( <$generic as Query>::State , )* );

        async fn init_state<'world>(world : &'world World) -> Self::State {
            $( let $generic = <$generic as Query>::init_state(world); )*
            multijoin!( $( $generic , )* )
        }

        unsafe fn acquire<'world, 'state>(world : &'world World, state : &'state mut Self::State) -> Poll<QueryAcquireResult<Self::Item<'world, 'state>>> {
            $(
                // SAFETY: TODO
                let $generic = match (unsafe{ <$generic as Query>::acquire(world, &mut state.${index()}) }) {
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
