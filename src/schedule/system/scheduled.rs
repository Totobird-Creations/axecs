//! TODO: Doc comment


use crate::world::World;
use crate::system::{ System, IntoSystem, TypeEraseableSystem };
use crate::schedule::system::ConditionNoneMarkerSystem;
use alloc::boxed::Box;


/// TODO: Doc comment
pub unsafe trait IntoScheduledSystemConfig<Params> {

    /// TODO: Doc comment
    fn into_schedulable_system_config(self, world : &World) -> ScheduledSystemConfig<impl System<()>, impl System<bool>>;

    // TODO: then

}


/// TODO: Doc comment
pub unsafe trait IntoConditionallyScheduledSystemConfig<Params> : IntoScheduledSystemConfig<Params> {

    // TODO: run_if

}


/// TODO: Doc comment
pub struct ScheduledSystemConfig<S : System<()>, C : System<bool>> {
    run_if : Option<C>,
    run    : S
}


/// TODO: Doc comment
pub struct TypeErasedScheduledSystemConfig {
    run : Box<dyn TypeEraseableSystem<(), ()>>
}


unsafe impl<S : System<()>, C : System<bool>> IntoScheduledSystemConfig<()> for ScheduledSystemConfig<S, C> {
    fn into_schedulable_system_config(self, _world : &World) -> ScheduledSystemConfig<impl System<()>, impl System<bool>> {
        self
    }
}

unsafe impl<S : System<()>> IntoConditionallyScheduledSystemConfig<()> for ScheduledSystemConfig<S, ConditionNoneMarkerSystem> { }


unsafe impl<Params, S : IntoSystem<Params, ()>> IntoScheduledSystemConfig<Params> for S {
    fn into_schedulable_system_config(self, world : &World) -> ScheduledSystemConfig<impl System<()>, impl System<bool>> {
        ScheduledSystemConfig {
            run_if : Option::<ConditionNoneMarkerSystem>::None,
            run    : <S as IntoSystem<_, _>>::into_system(self, world)
        }
    }
}
