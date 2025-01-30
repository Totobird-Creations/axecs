//! TODO: Doc comment


use crate::world::World;
use crate::schedule::system::{ IntoScheduledSystemConfig, TypeErasedSystem };
use crate::util::future::VecMultijoinFuture;
use alloc::boxed::Box;
use alloc::vec::Vec;


/// TODO: Doc comment
pub struct ScheduleStorage {

    /// TODO: Doc comment
    systems : Vec<Box<dyn TypeErasedSystem<(), ()>>>

}


impl ScheduleStorage {

    /// TODO: Doc comment
    pub fn new() -> Self {
        Self { systems : Vec::new() }
    }

    /// TODO: Doc comment
    pub fn add<S : IntoScheduledSystemConfig<Params> + 'static, Params : 'static>(&mut self, system : S) {
        self.systems.push( Box::new(system.into_scheduled_system_config()) );
    }

    // TODO: Doc comment
    pub async unsafe fn run(&mut self, world : &World) {
        let mut futures = Vec::with_capacity(self.systems.len());
        for system in &mut self.systems {
            // SAFETY: TODO
            futures.push(Box::pin(unsafe{ world.run_erased_system(system.as_mut(), ()) }));
        }
        VecMultijoinFuture::new(futures).await;
    }

}
