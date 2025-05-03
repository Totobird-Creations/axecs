//! Traits for creating collections of [`App`] setup logic


mod cycle_scheduler;
pub use cycle_scheduler::CycleSchedulerPlugin;

mod ctrlc;
pub use ctrlc::{ CtrlCPlugin, CtrlCStatus };


use crate::app::App;


/// A collection of [`App`] setup logic.
///
/// When a [`Plugin`] is installed to an [`App`] through [`App::add_plugin`], the plugin's [`Plugin::build`] method is run.
///
/// Plugins may only be added to an [`App`] once.
pub trait Plugin : Sized {

    /// Configures the [`App`] that this [`Plugin`] is installed to.
    fn build(self, app : &mut App);

}


impl<F : FnOnce(&mut App) -> ()> Plugin for F {
    fn build(self, app : &mut App) {
        self(app)
    }
}
