//! TODO: Doc comments


use crate::world::World;
use crate::query::{ Query, ReadOnlyQuery, QueryAcquireResult, QueryValidator };
use core::ops::{ Deref, DerefMut };
use core::task::Poll;


/// TODO: Doc comments
pub struct Local<'l, T : Default + 'static>(

    /// TODO: Doc comments
    &'l mut T

);

impl<'l, T : Default + 'static> Deref for Local<'l, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'l, T : Default + 'static> DerefMut for Local<'l, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}


unsafe impl<'l, T : Default + 'static> Query for Local<'l, T> {

    type Item<'world, 'state> = Local<'state, T>;

    type State = T;

    fn init_state(_world : &World) -> Self::State {
        <T as Default>::default()
    }

    unsafe fn acquire<'world, 'state>(_world : &'world World, state : &'state mut Self::State) -> Poll<QueryAcquireResult<Self::Item<'world, 'state>>> {
        Poll::Ready(QueryAcquireResult::Ready(Local(state)))
    }

    fn validate() -> QueryValidator {
        QueryValidator::empty()
    }
}

unsafe impl<'l, T : Default + 'static> ReadOnlyQuery for Local<'l, T> { }
