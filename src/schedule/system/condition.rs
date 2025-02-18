//! TODO: Doc comment


use crate::world::World;
use crate::system::{ System, ReadOnlySystem, IntoSystem, IntoReadOnlySystem };
use alloc::sync::Arc;


/// TODO: Doc comment
pub unsafe trait IntoConditionSystemConfig<Params> : Sized {

    /// TODO: Doc comment
    fn into_condition_system_config(self) -> ConditionSystemConfig<impl System<bool, Passed = ()>>;

}


/// TODO: Doc comment
pub struct ConditionSystemConfig<C : System<bool, Passed = ()>> {
    pub(super) run : C
}

unsafe impl<C : System<bool, Passed = ()>> IntoConditionSystemConfig<()> for ConditionSystemConfig<C> {
    fn into_condition_system_config(self) -> ConditionSystemConfig<impl System<bool, Passed = ()>> {
        self
    }
}


unsafe impl<Params, C : IntoReadOnlySystem<Params, bool, System = C1>, C1 : System<bool, Passed = ()>> IntoConditionSystemConfig<Params> for C
where <Self as IntoSystem<Params, bool>>::System : ReadOnlySystem<bool>
{
    fn into_condition_system_config(self) -> ConditionSystemConfig<impl System<bool, Passed = ()>> {
        ConditionSystemConfig {
            run : <C as IntoSystem<_, _>>::into_system(self)
        }
    }
}


/// TODO: Doc comment
pub(crate) struct ConditionNoneMarkerSystem();
impl System<bool> for ConditionNoneMarkerSystem {
    async unsafe fn acquire_and_run(&mut self, _passed : Self::Passed, _world : Arc<World>) -> bool {
        unreachable!()
    }
}
