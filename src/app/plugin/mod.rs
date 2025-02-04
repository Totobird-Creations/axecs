//! TODO: Doc comment


mod cycle_scheduler;
pub use cycle_scheduler::CycleSchedulerPlugin;


use crate::app::App;


/// TODO: Doc comment
pub trait Plugin : Sized {

    /// TODO: Doc comment
    fn build(self, app : &mut App);

}


impl<F : FnOnce(&mut App) -> ()> Plugin for F {
    fn build(self, app : &mut App) {
        self(app)
    }
}
