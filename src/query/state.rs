use crate::world::World;
use crate::query::{ Query, QueryAcquireResult };
use core::pin::Pin;
use core::task::{ Context, Poll };


/// TODO: Doc comments
pub struct PersistentQueryState<'l, Q : Query> {

    /// TODO: Doc comments
    world : &'l World,

    /// TODO: Doc comments
    state : Q::State<'l>

}

impl<'l, Q : Query> PersistentQueryState<'l, Q> {

    //// Creates a new [`PersistentQueryState`] which can later acquire values from the given [`World`].
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`Query`] does not violate the borrow checker rules. See [`QueryValidator`](crate::query::QueryValidator).
    pub(crate) unsafe fn new(world : &'l World) -> Self { Self {
        world,
        state : Q::init_state(world)
    } }

    /// TODO: Doc comments
    #[track_caller]
    pub fn try_acquire(&'l mut self) -> Poll<Q::Item<'l>> {
        match (unsafe{ Q::acquire(&self.world, &mut self.state) }) {
            Poll::Ready(out) => Poll::Ready(out.unwrap()),
            Poll::Pending    => Poll::Pending
        }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub async fn acquire(&'l mut self) -> Q::Item<'l> {
        unsafe{ QueryAcquireFuture::<Q>::new(
            self.world,
            &mut self.state
        ) }.await.unwrap()
    }

}


/// A [`Future`] that repeatedly calls [`Q::acquire`](Query::acquire) until it is acquired or otherwise errors.
pub struct QueryAcquireFuture<'l, Q : Query> {

    /// TODO: Doc comments
    world  : &'l World,

    /// TODO: Doc comments
    state  : &'l mut Q::State<'l>

}

impl<'l, Q : Query> Unpin for QueryAcquireFuture<'l, Q> { }

impl<'l, Q : Query> QueryAcquireFuture<'l, Q> {

    //// Creates a new [`QueryAcquireFuture`] which tries to acquire values from the given [`World`].
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`Query`] does not violate the borrow checker rules. See [`QueryValidator`](crate::query::QueryValidator).
    pub unsafe fn new(world : &'l World, state : &'l mut Q::State<'l>) -> Self { Self {
        world,
        state
    } }

}

impl<'l, Q : Query> Future for QueryAcquireFuture<'l, Q> {
    type Output = QueryAcquireResult<Q::Item<'l>>;

    fn poll(mut self : Pin<&mut Self>, _ctx : &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: TODO
        unsafe{ Q::acquire(&self.world, &mut self.state) }
    }
}
