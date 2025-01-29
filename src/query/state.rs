//! TODO: Doc comment


use crate::world::World;
use crate::query::{ Query, QueryAcquireResult };
use core::pin::Pin;
use core::task::{ Context, Poll };


/// TODO: Doc comments
pub struct PersistentQueryState<'world, Q : Query> {

    /// TODO: Doc comments
    world : &'world World,

    /// TODO: Doc comments
    state : Q::State

}

impl<'world, Q : Query> PersistentQueryState<'world, Q> {

    /// Creates a new [`PersistentQueryState`] which can later acquire values from the given [`World`].
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`Query`] does not violate the borrow checker rules. See [`QueryValidator`](crate::query::QueryValidator).
    pub(crate) unsafe fn new(world : &'world World) -> Self {
        Self {
            world,
            state : Q::init_state(world)
        }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn try_acquire<'query>(&'query mut self) -> Poll<Q::Item<'world, 'query>> {
        match (unsafe{ Q::acquire(&self.world, &mut self.state) }) {
            Poll::Ready(out) => Poll::Ready(out.unwrap()),
            Poll::Pending    => Poll::Pending
        }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub async fn acquire<'query>(&'query mut self) -> Q::Item<'world, 'query> {
        unsafe{ QueryAcquireFuture::<Q>::new(
            self.world,
            &mut self.state
        ) }.await.unwrap()
    }

}


/// A [`Future`] that repeatedly calls [`Q::acquire`](Query::acquire) until it is acquired or otherwise errors.
pub struct QueryAcquireFuture<'world, 'state, Q : Query>
where 'world : 'state
{

    /// TODO: Doc comments
    world  : &'world World,

    /// TODO: Doc comments
    state  : Option<&'state mut Q::State>

}

impl<'world, 'state, Q : Query> Unpin for QueryAcquireFuture<'world, 'state, Q> { }

impl<'world, 'state, Q : Query> QueryAcquireFuture<'world, 'state, Q>
{

    /// Creates a new [`QueryAcquireFuture`] which tries to acquire values from the given [`World`].
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`Query`] does not violate the borrow checker rules. See [`QueryValidator`](crate::query::QueryValidator).
    pub unsafe fn new(world : &'world World, state : &'state mut Q::State) -> Self { Self {
        world,
        state : Some(state)
    } }

}

impl<'world, 'state, Q : Query> Future for QueryAcquireFuture<'world, 'state, Q>
{
    type Output = QueryAcquireResult<Q::Item<'world, 'state>>;

    fn poll(mut self : Pin<&mut Self>, _ctx : &mut Context<'_>) -> Poll<Self::Output> {
        unsafe{ Q::acquire(self.world, self.state.take().unwrap()) }
    }
}
