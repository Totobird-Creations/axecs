//! TODO: Doc comment


use crate::system::{ System, IntoSystem };
use crate::schedule::system::{ ConditionSystemConfig, ConditionNoneMarkerSystem };


/// TODO: Doc comment
pub unsafe trait IntoScheduledSystemConfig<Params> {

    /// TODO: Doc comment
    fn into_scheduled_system_config(self) -> ScheduledSystemConfig<impl System<(), Passed = ()>, impl System<bool, Passed = ()>>;

    // TODO: then

}


/// TODO: Doc comment
pub unsafe trait IntoConditionallyScheduledSystemConfig<Params> : IntoScheduledSystemConfig<Params> {

    // TODO: run_if

}


/// TODO: Doc comment
pub struct ScheduledSystemConfig<S : System<(), Passed = ()>, C : System<bool, Passed = ()>> {
    pub(super) run_if : Option<ConditionSystemConfig<C>>,
    pub(super) run    : S
}


unsafe impl<S : System<(), Passed = ()>, C : System<bool, Passed = ()>> IntoScheduledSystemConfig<()> for ScheduledSystemConfig<S, C> {
    fn into_scheduled_system_config(self) -> ScheduledSystemConfig<impl System<(), Passed = ()>, impl System<bool, Passed = ()>> {
        self
    }
}

unsafe impl<S : System<(), Passed = ()>> IntoConditionallyScheduledSystemConfig<()> for ScheduledSystemConfig<S, ConditionNoneMarkerSystem> { }


unsafe impl<Params, S : IntoSystem<Params, (), System = S1>, S1 : System<(), Passed = ()>> IntoScheduledSystemConfig<Params> for S {
    fn into_scheduled_system_config(self) -> ScheduledSystemConfig<impl System<(), Passed = ()>, impl System<bool, Passed = ()>> {
        ScheduledSystemConfig {
            run_if : Option::<ConditionSystemConfig<ConditionNoneMarkerSystem>>::None,
            run    : <S as IntoSystem<_, _>>::into_system(self)
        }
    }
}
