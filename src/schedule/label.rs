//! TODO: Doc comment


use core::any::Any;


/// TODO: Doc comment
pub trait ScheduleLabel : Eq + Clone + Copy { }


/// TODO: Doc comment
pub(crate) unsafe trait ScheduleLabelEq {

    /// TODO: Doc comment
    fn schedule_label_eq(&self, other : &dyn ScheduleLabelEq) -> bool;

    /// TODO: Doc comment
    fn any_ref(&self) -> &dyn Any;

}

unsafe impl<L : ScheduleLabel + 'static> ScheduleLabelEq for L {

    fn schedule_label_eq(&self, other : &dyn ScheduleLabelEq) -> bool {
        if let Some(other) = other.any_ref().downcast_ref::<Self>() {
            self == other
        } else { false }
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }

}


/// TODO: Doc comment
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PreStartup;

impl ScheduleLabel for PreStartup {}


/// TODO: Doc comment
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Startup;

impl ScheduleLabel for Startup {}


/// TODO: Doc comment
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Cycle;

impl ScheduleLabel for Cycle {}


/// TODO: Doc comment
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Shutdown;

impl ScheduleLabel for Shutdown {}


/// TODO: Doc comment
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PostShutdown;

impl ScheduleLabel for PostShutdown {}
