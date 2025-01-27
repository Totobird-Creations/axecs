//! TODO: Doc comments


use crate::world::World;
use crate::query::{ Query, ReadOnlyQuery, QueryAcquireResult, QueryValidator };
use crate::util::variadic::variadic_no_unit;
use core::task::Poll;


unsafe impl Query for () {
    type Item<'item> = ();

    fn init_state<'world>(_world : &'world World) -> Self::State<'world> { () }

    unsafe fn acquire<'world>(_world : &'world World, _state : &mut Self::State<'world>) -> Poll<QueryAcquireResult<Self::Item<'world>>> {
        Poll::Ready(QueryAcquireResult::Ready(()))
    }

    fn validate() -> QueryValidator {
        QueryValidator::empty()
    }

}
unsafe impl ReadOnlyQuery for () { }


unsafe impl<Q : Query> Query for Option<Q> {
    type Item<'item> = Option<<Q as Query>::Item<'item>>;
    type State<'state> = <Q as Query>::State<'state>;

    fn init_state<'world>(world : &'world World) -> Self::State<'world> {
        <Q as Query>::init_state(world)
    }

    unsafe fn acquire<'world>(world : &'world World, state : &mut Self::State<'world>) -> Poll<QueryAcquireResult<Self::Item<'world>>> {
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

    $( #[ $meta ] )*
    unsafe impl< $( $generic : Query ),* > Query for ( $( $generic , )* ) {
        type Item<'item> = ( $( <$generic as Query>::Item<'item> , )* );
        type State<'state> = TupleDefault<( $( <$generic as Query>::State<'state> , )* )>;

        fn init_state<'world>(world : &'world World) -> Self::State<'world> {
            TupleDefault(( $( <$generic as Query>::init_state(world) , )* ))
        }

        unsafe fn acquire<'world>(world : &'world World, state : &mut Self::State<'world>) -> Poll<QueryAcquireResult<Self::Item<'world>>> {
            $(
                #[allow(non_snake_case)]
                // SAFETY: TODO
                let $generic = match (unsafe{ <$generic as Query>::acquire(world, &mut state.0.${index()}) }) {
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


pub struct TupleDefault<T>(T);


variadic_no_unit!{ impl_default_for_tuple_default }
macro impl_default_for_tuple_default( $( #[$meta:meta] )* $( $generic:ident ),* $(,)? ) {

    $( #[ $meta ] )*
    impl< $( $generic : Default ),* > Default for TupleDefault<( $( $generic , )* )> {
        fn default() -> Self {
            TupleDefault(( $( <$generic as Default>::default() , )* ))
        }
    }

}
