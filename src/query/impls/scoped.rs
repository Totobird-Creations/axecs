//! TODO: Doc comments


use crate::world::World;
use crate::system::SystemId;
use crate::query::{ Query, ReadOnlyQuery, QueryAcquireResult, QueryAcquireFuture, QueryValidator };
use core::ops::AsyncFnMut;
use core::task::Poll;
use alloc::sync::Arc;


/// TODO: Doc comment
pub struct Scoped<Q : Query> {

    /// TODO: Doc comment
    world : Arc<World>,

    /// TODO: Doc comment
    state : Q::State

}


impl<Q : Query> Scoped<Q> {

    #[track_caller]
    pub async fn with<F : AsyncFnMut(Q::Item) -> U, U>(&mut self, f : F) -> U {
        self.maybe_lock(f).await.unwrap("Scoped")
    }

    #[track_caller]
    pub async fn lock(&mut self) -> Q::Item {
        self.maybe_lock(async |q| q).await.unwrap("Scoped")
    }

    pub async fn maybe_lock<F : AsyncFnMut(Q::Item) -> U, U>(&mut self, mut f : F) -> QueryAcquireResult<U> {
        // SAFETY: TODO
        match (unsafe{ QueryAcquireFuture::<Q>::new(Arc::clone(&self.world), &mut self.state) }.await) {

            QueryAcquireResult::Ready(out) => QueryAcquireResult::Ready(f(out).await),

            #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
            QueryAcquireResult::DoesNotExist { name } => QueryAcquireResult::DoesNotExist { name },
            #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
            QueryAcquireResult::DoesNotExist {      } => QueryAcquireResult::DoesNotExist {      },

        }
    }

}


unsafe impl<Q : Query> Query for Scoped<Q> {

    type Item = Scoped<Q>;

    type State = Option<SystemId>;

    fn init_state(_world : Arc<World>, system_id : Option<SystemId>) -> Self::State {
        system_id
    }

    unsafe fn acquire(world : Arc<World>, state : &mut Self::State) -> Poll<QueryAcquireResult<Self::Item>> {
        Poll::Ready(QueryAcquireResult::Ready(Scoped {
            world : Arc::clone(&world),
            state : Q::init_state(world, *state)
        }))
    }

    fn validate() -> QueryValidator {
        Q::validate()
    }

}

unsafe impl<Q : ReadOnlyQuery> ReadOnlyQuery for Scoped<Q> { }
