//! TODO: Doc comments


use crate::app::AppExit;
use crate::world::World;
use crate::resource::Resource;
use crate::component::bundle::ComponentBundle;
use crate::query::{ Query, QueryAcquireResult, QueryValidator };
use crate::system::{ IntoSystem, IntoReadOnlySystem, System, ReadOnlySystem };
use core::task::Poll;
use alloc::boxed::Box;
use alloc::sync::Arc;


/// TODO: Doc comments
#[derive(Clone)]
pub struct Commands {

    /// TODO: Doc comments
    world : Arc<World>

}


impl Commands {

    /*/// TODO: Doc comments
    ///
    /// # Warning
    /// If this is used to run an operation on the [`World`] which requests some values,
    ///  but the calling system has already locked it, the operation will deadlock.
    /// Use with caution.
    pub fn world(&self) -> &World {
        self.world.as_ref()
    }*/

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
            Box::pin(async move { world.insert_resource(resource).await })
        ))
    }

    /// TODO: Doc comments
    pub async fn remove_resource<R : Resource + 'static>(&self) {
        self.world.cmd_queue.write().await.push(Box::new(move |world|
            Box::pin(async move { world.remove_resource::<R>().await })
        ))
    }

    /// TODO: Doc comments
    pub async fn spawn<B : ComponentBundle + 'static>(&self, bundle : B) { // TODO: Immediately reserve space for the entities.
        self.world.cmd_queue.write().await.push(Box::new(move |world|
            Box::pin(async move { world.spawn(bundle).await; })
        ))
    }

    /// TODO: Doc comments
    pub async fn spawn_batch<B : ComponentBundle + 'static>(&self, bundles : impl IntoIterator<Item = B> + 'static) {
        self.world.cmd_queue.write().await.push(Box::new(move |world|
            Box::pin(async move { let _ = world.spawn_batch(bundles).await; })
        ))
    }

    /// TODO: Doc comments
    pub async fn run_system<S : IntoReadOnlySystem<Params, ()> + 'static, Params>(&self, system : S)
    where <S as IntoSystem<Params, ()>>::System : System<(), Passed = ()> + ReadOnlySystem<()>
    {
        self.world.cmd_queue.write().await.push(Box::new(|world| Box::pin(async {
            let mut system = system.into_system();
            // SAFETY: TODO
            unsafe{ system.acquire_and_run((), world) }.await;
        })));
    }

    /// TODO: Doc comments
    pub async fn run_system_mut<S : IntoSystem<Params, ()> + 'static, Params>(&self, system : S)
    where <S as IntoSystem<Params, ()>>::System : System<(), Passed = ()>
    {
        self.world.cmd_queue.write().await.push(Box::new(|world| Box::pin(async {
            let mut system = system.into_system();
            // SAFETY: TODO
            unsafe{ system.acquire_and_run((), world) }.await;
        })));
    }

}


unsafe impl Query for Commands {

    type Item = Commands;

    type State = ();

    fn init_state() -> Self::State {
        ()
    }

    unsafe fn acquire(world : Arc<World>, _state : &mut Self::State) -> Poll<QueryAcquireResult<Self::Item>> {
        Poll::Ready(QueryAcquireResult::Ready(Commands{ world }))
    }

    fn validate() -> QueryValidator {
        QueryValidator::empty()
    }

}
