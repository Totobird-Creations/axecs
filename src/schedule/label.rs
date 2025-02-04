//! Label which group systems together to be run together.


use core::any::Any;


/// A label which groups systems together to be run together.
pub trait ScheduleLabel : Eq + Clone + Copy { }


/// A [`ScheduleLabel`] with helper methods for dealing with type-erasure.
pub(crate) unsafe trait TypeErasedScheduleLabel {

    /// Compares this [`TypeErasedScheduleLabel`] to another.
    ///
    /// This should [`Any::downcast_ref`](https://doc.rust-lang.org/nightly/core/any/trait.Any.html#method.downcast_ref) and compare using [`==`](PartialEq::eq).
    fn schedule_label_eq(&self, other : &dyn TypeErasedScheduleLabel) -> bool; // TODO: Swap out the direct link for rustdoc thing.

    /// Get this value as [`&dyn Any`](Any).
    fn any_ref(&self) -> &dyn Any;

}

unsafe impl<L : ScheduleLabel + 'static> TypeErasedScheduleLabel for L {

    fn schedule_label_eq(&self, other : &dyn TypeErasedScheduleLabel) -> bool {
        if let Some(other) = other.any_ref().downcast_ref::<Self>() {
            self == other
        } else { false }
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }

}


/// The schedule that first runs when the app starts.
///
/// All [`PreStartup`] systems must have fully completed before the program can go on to [`Startup`].
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PreStartup;

impl ScheduleLabel for PreStartup {}


/// The schedule that runs after [`PreStartup`] is fully completed.
///
/// [`Startup`] will run alongside [`Cycle`], making it perfect for creating loops that run the entire duration of the program.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Startup;

impl ScheduleLabel for Startup {}


/// The schedule that runs and loops after [`PreStartup`] is fully completed.
///
/// [`Cycle`] will run alongside [`Startup`].
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Cycle;

impl ScheduleLabel for Cycle {}


/// The schedule that runs when the program begins to exit.
///
/// See [`World::exit`](crate::world::World::exit) and [`Commands::exit`](crate::world::Commands::exit).
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Shutdown;

impl ScheduleLabel for Shutdown {}


/// The schedule that runs after the program begins to exit, and all other systems are fully completed.
///
/// See [`World::exit`](crate::world::World::exit) and [`Commands::exit`](crate::world::Commands::exit).
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PostShutdown;

impl ScheduleLabel for PostShutdown {}
