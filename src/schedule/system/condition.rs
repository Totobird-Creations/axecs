//! TODO: Doc comment


use crate::world::World;
use crate::system::{ System, ReadOnlySystem, IntoSystem, IntoReadOnlySystem };


/// TODO: Doc comment
pub unsafe trait IntoConditionSystemConfig<Params> {

    /// TODO: Doc comment
    fn into_condition_system_config(self, world : &World) -> ConditionSystemConfig<impl System<bool>>;

    // TODO: not

    // TODO: and

    // TODO: nand

    // TODO: or

    // TODO: nor

    // TODO: xor

    // TODO: xnor

}


/// TODO: Doc comment
pub struct ConditionSystemConfig<C : System<bool>> {
    pub(crate) run : C
}

unsafe impl<C : System<bool>> IntoConditionSystemConfig<()> for ConditionSystemConfig<C> {
    fn into_condition_system_config(self, _world : &World) -> ConditionSystemConfig<impl System<bool>> {
        self
    }
}


unsafe impl<Params, C : IntoReadOnlySystem<Params, bool>> IntoConditionSystemConfig<Params> for C
where <Self as IntoSystem<Params, bool>>::System : ReadOnlySystem<bool>
{
    fn into_condition_system_config(self, world : &World) -> ConditionSystemConfig<impl System<bool>> {
        ConditionSystemConfig {
            run : <C as IntoSystem<_, _>>::into_system(self, world)
        }
    }
}


/// TODO: Doc comment
pub(crate) struct ConditionNoneMarkerSystem();
impl System<bool> for ConditionNoneMarkerSystem {
    async unsafe fn acquire_and_run(&mut self, _passed : Self::Passed, _world : &World) -> bool {
        unreachable!()
    }
}
