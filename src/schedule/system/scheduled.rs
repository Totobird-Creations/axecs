//! TODO: Doc comment


use crate::system::{ System, IntoSystem };
use crate::schedule::system::{ ConditionSystemConfig, ConditionNoneMarkerSystem };


/// TODO: Doc comment
/// 
/// #### Safety
/// The implementor is responsible for ensuring that this transforms into a valid [`System`].
/// Queries may not violate the borrow checker rules. See [`QueryValidator`](crate::query::QueryValidator).
pub unsafe trait IntoScheduledSystemConfig<'l, Params> {

    /// TODO: Doc comment
    fn into_scheduled_system_config(self) -> ScheduledSystemConfig<impl System<(), Passed = ()> + 'l, impl System<bool, Passed = ()> + 'l>;

    // TODO: then

}


/// TODO: Doc comment
pub unsafe trait IntoConditionallyScheduledSystemConfig<'l, Params> : IntoScheduledSystemConfig<'l, Params> {

    // TODO: run_if

}


/// TODO: Doc comment
pub struct ScheduledSystemConfig<S : System<(), Passed = ()>, C : System<bool, Passed = ()>> {
    pub(super) run_if : Option<ConditionSystemConfig<C>>,
    pub(super) run    : S
}


unsafe impl<'l, S : System<(), Passed = ()> + 'l, C : System<bool, Passed = ()> + 'l> IntoScheduledSystemConfig<'l, ()> for ScheduledSystemConfig<S, C> {
    fn into_scheduled_system_config(self) -> ScheduledSystemConfig<impl System<(), Passed = ()> + 'l, impl System<bool, Passed = ()> + 'l> {
        self
    }
}

unsafe impl<'l, S : System<(), Passed = ()> + 'l> IntoConditionallyScheduledSystemConfig<'l, ()> for ScheduledSystemConfig<S, ConditionNoneMarkerSystem> { }


unsafe impl<'l, Params, S : IntoSystem<Params, (), System = S1>, S1 : System<(), Passed = ()> + 'l> IntoScheduledSystemConfig<'l, Params> for S {
    #[track_caller]
    fn into_scheduled_system_config(self) -> ScheduledSystemConfig<impl System<(), Passed = ()> + 'l, impl System<bool, Passed = ()> + 'l> {
        ScheduledSystemConfig {
            run_if : Option::<ConditionSystemConfig<ConditionNoneMarkerSystem>>::None,
            run    : <S as IntoSystem<_, _>>::into_system(self)
        }
    }
}
