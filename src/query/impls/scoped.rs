//! TODO: Doc comments


use crate::world::World;
use crate::query::{ Query, ReadOnlyQuery, QueryAcquireResult, QueryAcquireFuture, QueryValidator };
use core::ops::AsyncFnMut;
use core::task::Poll;


/// TODO: Doc comment
pub struct Scoped<'l, Q : Query> {

    /// TODO: Doc comment
    world : &'l World,

    /// TODO: Doc comment
    state : Q::State

}


impl<'l, Q : Query> Scoped<'l, Q> {

    pub async fn with<F : AsyncFnMut(Q::Item<'l, 'l>) -> U, U>(&'l mut self, f : F) -> U {
        self.try_with(f).await.unwrap()
    }

    pub async fn try_with<F : AsyncFnMut(Q::Item<'l, 'l>) -> U, U>(&'l mut self, mut f : F) -> QueryAcquireResult<U> {
        // SAFETY: TODO
        match (unsafe{ QueryAcquireFuture::<Q>::new(self.world, &mut self.state) }.await) {

            QueryAcquireResult::Ready(out) => QueryAcquireResult::Ready(f(out).await),

            #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
            QueryAcquireResult::DoesNotExist { name } => QueryAcquireResult::DoesNotExist { name },
            #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
            QueryAcquireResult::DoesNotExist {      } => QueryAcquireResult::DoesNotExist {      },

        }
    }

}


unsafe impl<'l, Q : Query> Query for Scoped<'l, Q> {

    type Item<'world, 'state> = Scoped<'world, Q>;

    type State = ();

    fn init_state() -> Self::State {
        ()
    }

    unsafe fn acquire<'world, 'state>(world : &'world World, _state : &'state mut Self::State) -> Poll<QueryAcquireResult<Self::Item<'world, 'state>>> {
        Poll::Ready(QueryAcquireResult::Ready(Scoped {
            world,
            state : Q::init_state()
        }))
    }

    fn validate() -> QueryValidator {
        Q::validate()
    }

}

unsafe impl<'l, Q : ReadOnlyQuery> ReadOnlyQuery for Scoped<'l, Q> { }
