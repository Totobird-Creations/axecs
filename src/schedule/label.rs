//! TODO: Doc comment


/// TODO: Doc comment
pub trait ScheduleLabel : Eq { }


/// TODO: Doc comment
#[derive(PartialEq, Eq)]
pub struct Startup;

impl ScheduleLabel for Startup {}


/// TODO: Doc comment
#[derive(PartialEq, Eq)]
pub struct Shutdown;

impl ScheduleLabel for Shutdown {}
