//! TODO: Doc comment


use crate::world::World;
use crate::system::SystemId;
use crate::query::{ Query, QueryAcquireResult };
use core::pin::Pin;
use core::task::{ Context, Poll };
use alloc::sync::Arc;


/// TODO: Doc comments
pub struct PersistentQueryState<Q : Query> {

    /// TODO: Doc comments
    world : Arc<World>,

    /// TODO: Doc comments
    state : Q::State

}

impl<Q : Query> PersistentQueryState<Q> {

    /// Creates a new [`PersistentQueryState`] which can later acquire values from the given [`World`].
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`Query`] does not violate the borrow checker rules. See [`QueryValidator`](crate::query::QueryValidator).
    pub(crate) unsafe fn new(world : Arc<World>, system_id : Option<SystemId>) -> Self {
        Self {
            world : Arc::clone(&world),
            state : Q::init_state(world, system_id)
        }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn try_acquire(&mut self) -> Poll<Q::Item> {
        // SAFETY: TODO
        match (unsafe{ Q::acquire(Arc::clone(&self.world), &mut self.state) }) {
            Poll::Ready(out) => Poll::Ready(out.unwrap("Query")),
            Poll::Pending    => Poll::Pending
        }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub async fn acquire(&mut self) -> Q::Item {
        // SAFETY: TODO
        unsafe{ QueryAcquireFuture::<Q>::new(
            Arc::clone(&self.world),
            &mut self.state
        ) }.await.unwrap("Query")
    }

}


/*/// TODO: Doc comments
pub struct StatelessQueryItem<Q : StatelessQuery> {

    /// TODO: Doc comments
    item  : MaybeUninit<Q::Item>,

    /// TODO: Doc comments
    state : UnsafeCell<Q::State>

}

impl<'world, 'state, Q : StatelessQuery> StatelessQueryItem<'world, 'state, Q> {

    /// TODO: Doc comment
    #[track_caller]
    pub(crate) async unsafe fn new(world : &'world World) -> Self {
        let mut item = Self {
            item  : MaybeUninit::uninit(),
            state : UnsafeCell::new(Q::init_state())
        };
        // SAFETY: TODO
        item.item.write(unsafe { QueryAcquireFuture::<Q>::new(
            world,
            &mut*item.state.get()
        ) }.await.unwrap());
        item
    }

}

impl<'world, 'state, Q : StatelessQuery> Deref for StatelessQueryItem<'world, 'state, Q> {
    type Target = Q::Item<'world, 'state>;
    fn deref(&self) -> &Self::Target {
        // SAFETY: TODO
        unsafe{ self.item.assume_init_ref() }
    }
}

impl<'world, 'state, Q : StatelessQuery> DerefMut for StatelessQueryItem<'world, 'state, Q> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: TODO
        unsafe{ self.item.assume_init_mut() }
    }
}

impl<'world, 'state, Q : StatelessQuery> Drop for StatelessQueryItem<'world, 'state, Q> {
    fn drop(&mut self) {
        // SAFETY: TODO
        unsafe{ self.item.assume_init_drop(); }
    }
}*/


/// A [`Future`] that repeatedly calls [`Q::acquire`](Query::acquire) until it is acquired or otherwise errors.
pub struct QueryAcquireFuture<'state, Q : Query> {

    /// TODO: Doc comments
    world  : Arc<World>,

    /// TODO: Doc comments
    state  : &'state mut Q::State

}

impl<'state, Q : Query> Unpin for QueryAcquireFuture<'state, Q> { }

impl<'state, Q : Query> QueryAcquireFuture<'state, Q>
{

    /// Creates a new [`QueryAcquireFuture`] which tries to acquire values from the given [`World`].
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`Query`] does not violate the borrow checker rules. See [`QueryValidator`](crate::query::QueryValidator).
    pub unsafe fn new(world : Arc<World>, state : &'state mut Q::State) -> Self {
        Self { world, state }
    }

}

impl<'state, Q : Query> Future for QueryAcquireFuture<'state, Q>
{
    type Output = QueryAcquireResult<Q::Item>;

    fn poll(mut self : Pin<&mut Self>, _ctx : &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: TODO
        unsafe{ Q::acquire(Arc::clone(&self.world), self.state) }
    }
}
