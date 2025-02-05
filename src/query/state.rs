//! TODO: Doc comment


use crate::world::World;
use crate::query::{ Query, StatelessQuery, QueryAcquireResult };
use core::pin::Pin;
use core::task::{ Context, Poll };
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::{ Deref, DerefMut };


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
            state : Q::init_state()
        }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub fn try_acquire<'query>(&'query mut self) -> Poll<Q::Item<'world, 'query>> {
        // SAFETY: TODO
        match (unsafe{ Q::acquire(&self.world, &mut self.state) }) {
            Poll::Ready(out) => Poll::Ready(out.unwrap()),
            Poll::Pending    => Poll::Pending
        }
    }

    /// TODO: Doc comments
    #[track_caller]
    pub async fn acquire<'query>(&'query mut self) -> Q::Item<'world, 'query> {
        // SAFETY: TODO
        unsafe{ QueryAcquireFuture::<Q>::new(
            self.world,
            &mut self.state
        ) }.await.unwrap()
    }

}


/// TODO: Doc comments
pub struct StatelessQueryItem<'world, 'state, Q : StatelessQuery> {

    /// TODO: Doc comments
    item  : MaybeUninit<Q::Item<'world, 'state>>,

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
}


/// A [`Future`] that repeatedly calls [`Q::acquire`](Query::acquire) until it is acquired or otherwise errors.
pub struct QueryAcquireFuture<'world, 'state, Q : Query> {

    /// TODO: Doc comments
    world  : &'world World,

    /// TODO: Doc comments
    state  : UnsafeCell<&'state mut Q::State>

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
        state : UnsafeCell::new(state)
    } }

}

impl<'world, 'state, Q : Query> Future for QueryAcquireFuture<'world, 'state, Q>
{
    type Output = QueryAcquireResult<Q::Item<'world, 'state>>;

    fn poll(self : Pin<&mut Self>, _ctx : &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: TODO
        unsafe{ Q::acquire(self.world, &mut*self.state.get()) }
    }
}
