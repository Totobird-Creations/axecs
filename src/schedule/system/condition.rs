//! TODO: Doc comment


use crate::world::World;
use crate::system::{ SystemId, System, ReadOnlySystem, IntoSystem, IntoReadOnlySystem };
use crate::util::lazycell::LazyCell;
use alloc::sync::Arc;
use alloc::boxed::Box;


/// TODO: Doc comment
pub unsafe trait IntoConditionSystemConfig<Params> : Sized {

    /// TODO: Doc comment
    fn into_condition_system_config(self) -> ConditionSystemConfig<impl System<bool, Passed = ()>>;

}


/// TODO: Doc comment
pub struct ConditionSystemConfig<C : System<bool, Passed = ()>> {
    pub(super) run : LazyCell<C, Arc<World>, Box<dyn FnOnce(Arc<World>) -> C>>
}

unsafe impl<C : System<bool, Passed = ()>> IntoConditionSystemConfig<()> for ConditionSystemConfig<C> {
    fn into_condition_system_config(self) -> ConditionSystemConfig<impl System<bool, Passed = ()>> {
        self
    }
}


unsafe impl<Params, C : IntoReadOnlySystem<Params, bool, System = C1> + 'static, C1 : System<bool, Passed = ()>> IntoConditionSystemConfig<Params> for C
where <Self as IntoSystem<Params, bool>>::System : ReadOnlySystem<bool>
{
    fn into_condition_system_config(self) -> ConditionSystemConfig<impl System<bool, Passed = ()>> {
        ConditionSystemConfig {
            run : LazyCell::new(Box::new(|world| <C as IntoSystem<_, _>>::into_system(self, world, Some(SystemId::unique()))))
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
