//! TODO: Doc comments


use crate::app::AppExit;
use crate::world::World;
use crate::query::{ Query, QueryAcquireResult, QueryValidator };
use core::task::Poll;


/// TODO: Doc comments
pub struct Commands<'l> {

    /// TODO: Doc comments
    world : &'l World

}


impl<'l> Commands<'l> {

    /// TODO: Doc comments
    pub fn is_exiting(&self) -> bool {
        self.world.is_exiting()
    }

    /// TODO: Doc comments
    pub fn exit(&self, status : AppExit) {
        self.world.exit(status)
    }

    /// TODO: Doc comments
    pub fn try_exit(&self, status : AppExit) {
        self.world.try_exit(status)
    }

}


unsafe impl<'l> Query for Commands<'l> {

    type Item<'world, 'state> = Commands<'world>;

    type State = ();

    fn init_state() -> Self::State {
        ()
    }

    unsafe fn acquire<'world, 'state>(world : &'world World, _state : &'state mut Self::State) -> Poll<QueryAcquireResult<Self::Item<'world, 'state>>> {
        Poll::Ready(QueryAcquireResult::Ready(Commands{ world }))
    }

    fn validate() -> QueryValidator {
        QueryValidator::empty()
    }

}
