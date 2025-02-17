//! TODO: Doc comments


use crate::app::AppExit;
use crate::world::World;
use crate::resource::Resource;
use crate::query::{ Query, QueryAcquireResult, QueryValidator };
use crate::system::{ IntoSystem, IntoReadOnlySystem, IntoStatelessSystem, System, ReadOnlySystem, StatelessSystem };
use core::task::Poll;
use alloc::boxed::Box;


/// TODO: Doc comments
#[derive(Clone, Copy)]
pub struct Commands<'l> {

    /// TODO: Doc comments
    world : &'l World

}


impl<'l> Commands<'l> {

    /// TODO: Doc comments
    ///
    /// # Warning
    /// If this is used to run an operation on the [`World`] which requests some values,
    ///  but the calling system has already locked it, the operation will deadlock.
    /// Use with caution.
    pub fn world(&self) -> &World {
        self.world
    }

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

    /// TODO: Doc comments
    pub async fn insert_resource<R : Resource + 'static>(&self, resource : R) {
        self.world.cmd_queue.write().await.push(Box::new(move |world|
            Box::pin(world.insert_resource(resource))
        ))
    }

    /// TODO: Doc comments
    /*pub async fn spawn<B : ComponentBundle + 'static>(&self, bundle : B) -> Entity { // TODO: Immediately reserve space for the entities.
        self.world.cmd_queue.write().await.push(Box::new(move |world|
            Box::pin(self.world.spawn(bundle))
        ))
    }

    /// TODO: Doc comments
    pub async fn spawn_batch<B : ComponentBundle + 'static>(&self, bundles : impl IntoIterator<Item = B>) -> impl Iterator<Item = Entity> {
        self.world.cmd_queue.write().await.push(Box::new(move |world|
            Box::pin(self.world.spawn_batch(bundles))
        ))
    }*/

    /// TODO: Doc comments
    pub async fn stateless_run_system<S : IntoReadOnlySystem<Params, ()> + IntoStatelessSystem<Params, ()> + 'static, Params>(&self, system : S)
    where <S as IntoSystem<Params, ()>>::System : System<(), Passed = ()> + ReadOnlySystem<()> + StatelessSystem<()>
    {
        self.world.cmd_queue.write().await.push(Box::new(|world| Box::pin(async {
            let mut system = system.into_system();
            // SAFETY: TODO
            unsafe{ system.acquire_and_run((), world) }.await;
        })));
    }

    /// TODO: Doc comments
    pub async fn stateless_run_system_mut<S : IntoStatelessSystem<Params, ()> + 'static, Params>(&self, system : S)
    where <S as IntoSystem<Params, ()>>::System : System<(), Passed = ()> + StatelessSystem<()>
    {
        self.world.cmd_queue.write().await.push(Box::new(|world| Box::pin(async {
            let mut system = system.into_system();
            // SAFETY: TODO
            unsafe{ system.acquire_and_run((), world) }.await;
        })));
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
