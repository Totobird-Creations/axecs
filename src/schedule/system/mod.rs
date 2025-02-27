//! TODO: Doc comment


mod scheduled;
pub use scheduled::*;

mod condition;
pub use condition::*;


use crate::world::World;
use crate::system::{ System, SystemPassable };
use core::pin::Pin;
use alloc::boxed::Box;
use alloc::sync::Arc;
use async_std::task;


/// TODO: Doc comment
///
/// #### Safety
/// Do not implement this manually. Instead, implement [`IntoSystem`](crate::system::IntoSystem) or [`System`].
pub unsafe trait TypeErasedSystem<Passed, Return> {

    /// TODO: Doc comment
    unsafe fn acquire_and_run<'l>(&'l mut self, passed : Passed, world : Arc<World>) -> Pin<Box<dyn Future<Output = Return> + 'l>>;

}


unsafe impl<S : System<Return, Passed = Passed>, Passed : SystemPassable, Return : 'static> TypeErasedSystem<Passed, Return> for S {
    unsafe fn acquire_and_run<'l>(&'l mut self, passed : Passed, world : Arc<World>) -> Pin<Box<dyn Future<Output = Return> + 'l>> {
        // SAFETY: TODO
        Box::pin(unsafe{ <Self as System<_>>::acquire_and_run(self, passed, world) })
    }
}

unsafe impl<S : System<(), Passed = ()>, C : System<bool, Passed = ()>> TypeErasedSystem<(), ()> for ScheduledSystemConfig<S, C> {
    unsafe fn acquire_and_run<'l>(&'l mut self, _passed : (), world : Arc<World>) -> Pin<Box<dyn Future<Output = ()> + 'l>> {
        Box::pin(async {
            if (! self.depends_ids.is_empty()) {
                'wait_for_dependencies : loop {
                    if (world.is_exiting()) { return; }
                    {
                        let mut success = true;
                        let ran_systems = world.ran_systems.read().await;
                        'check_dependencies : for depends_on in &self.depends_ids {
                            if (! ran_systems.contains(depends_on)) {
                                success = false;
                                break 'check_dependencies;
                            }
                        }
                        if (success) { break 'wait_for_dependencies; }
                    }
                    task::yield_now().await;
                }
            }
            if let Some(run_if) = &mut self.run_if {
                // SAFETY: TODO
                if (! unsafe{ run_if.run.get_mut(Arc::clone(&world)).acquire_and_run((), Arc::clone(&world)) }.await) {
                    return;
                }
            }
            // SAFETY: TODO
            unsafe{ self.run.get_mut(Arc::clone(&world)).acquire_and_run((), world) }.await
        })
    }
}

unsafe impl<C : System<bool, Passed = ()>> TypeErasedSystem<(), bool> for ConditionSystemConfig<C> {
    unsafe fn acquire_and_run<'l>(&'l mut self, _passed : (), world : Arc<World>) -> Pin<Box<dyn Future<Output = bool> + 'l>> {
        // SAFETY: TODO
        Box::pin(unsafe{ self.run.get_mut(Arc::clone(&world)).acquire_and_run((), world) })
    }
}
