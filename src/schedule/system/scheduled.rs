//! TODO: Doc comment


use crate::world::World;
use crate::system::{ SystemId, System, IntoSystem };
use crate::schedule::system::{ ConditionSystemConfig, IntoConditionSystemConfig, ConditionNoneMarkerSystem };
use crate::util::lazycell::LazyCell;
use alloc::boxed::Box;
use alloc::sync::Arc;


/// TODO: Doc comment
///
/// #### Safety
/// The implementor is responsible for ensuring that this transforms into a valid [`System`].
/// Queries may not violate the borrow checker rules. See [`QueryValidator`](crate::query::QueryValidator).
pub unsafe trait IntoScheduledSystemConfig<'l, Params : 'l> : Sized
where Self : 'l
{

    /// TODO: Doc comment
    fn into_scheduled_system_config(self) -> ScheduledSystemConfig<impl System<(), Passed = ()> + 'l, impl System<bool, Passed = ()> + 'l>;

    // TODO: Doc comment
    fn run_if<C : IntoConditionSystemConfig<Params1>, Params1>(self, condition : C) -> ScheduledSystemConfig<impl System<(), Passed = ()>, impl System<bool, Passed = ()>> {
        let config = self.into_scheduled_system_config();
        match (config.run_if) {
            Some(_) => todo!(), // TODO: Merge `run_if`s
            None => ScheduledSystemConfig {
                run_if : Some(condition.into_condition_system_config()),
                run    : config.run
            }
        }
    }

}


/// TODO: Doc comment
pub struct ScheduledSystemConfig<S : System<(), Passed = ()>, C : System<bool, Passed = ()>> {
    pub(super) run_if : Option<ConditionSystemConfig<C>>,
    pub(super) run    : LazyCell<S, Arc<World>, Box<dyn FnOnce(Arc<World>) -> S>>
}


unsafe impl<'l, S : System<(), Passed = ()> + 'l, C : System<bool, Passed = ()> + 'l> IntoScheduledSystemConfig<'l, ()> for ScheduledSystemConfig<S, C> {
    fn into_scheduled_system_config(self) -> ScheduledSystemConfig<impl System<(), Passed = ()> + 'l, impl System<bool, Passed = ()> + 'l> {
        self
    }
}


unsafe impl<'l, Params : 'l, S : IntoSystem<Params, (), System = S1> + 'static, S1 : System<(), Passed = ()> + 'l> IntoScheduledSystemConfig<'l, Params> for S {
    #[track_caller]
    fn into_scheduled_system_config(self) -> ScheduledSystemConfig<impl System<(), Passed = ()> + 'l, impl System<bool, Passed = ()> + 'l> {
        ScheduledSystemConfig {
            run_if : Option::<ConditionSystemConfig<ConditionNoneMarkerSystem>>::None,
            run    : LazyCell::new(Box::new(|world| <S as IntoSystem<_, _>>::into_system(self, world, Some(SystemId::unique()))))
        }
    }
}
