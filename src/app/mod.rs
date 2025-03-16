//! The highest-level API for setting up applications.


pub mod plugin;
use plugin::Plugin;


use crate::resource::{ Resource, RawResourceStorage };
use crate::schedule::ScheduleStorage;
use crate::schedule::label::ScheduleLabel;
use crate::schedule::system::IntoScheduledSystemConfig;
use core::any::{ TypeId, type_name };
use core::error::Error;
use core::pin::Pin;
use core::ops::AsyncFnOnce;
use core::mem;
use alloc::boxed::Box;
use alloc::collections::BTreeSet;


/// The primary API for writing applications.
///
/// ### Examples
/// Here is a simple "Hello World" app:
/// ```rust
/// use axecs::prelude::*;
/// # use async_std::main;
///
/// #[main]
/// async fn main() {
///     let mut app = App::new();
///     app.add_plugin(CycleSchedulerPlugin);
///     app.add_systems(Cycle, hello_world_system);
///     app.run().await;
/// }
///
/// async fn hello_world_system() {
///     println!("Hello, World!");
/// }
/// ```
pub struct App {

    /// [`TypeId`]s of [`Plugin`]s that have already been installed.
    installed_plugins : BTreeSet<TypeId>,

    /// The function that is called when [`App::run`] is called.
    runner            : Option<Box<dyn FnOnce(App) -> Pin<Box<dyn Future<Output = AppExit>>>>>,

    /// Schedules of [`System`](crate::system::System)s in this [`App`].
    schedules         : Option<ScheduleStorage>,

    /// Resources that the [`App`] will start with.
    resources         : Option<RawResourceStorage>

}

impl App {

    /// Create an empty [`App`].
    pub fn new() -> Self { Self {
        installed_plugins : BTreeSet::new(),
        runner            : None,
        schedules         : Some(ScheduleStorage::new()),
        resources         : Some(RawResourceStorage::new())
    } }

    /// Installs a [`Plugin`].
    ///
    /// ### Examples
    /// ```rust
    /// use axecs::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.add_plugin(CycleSchedulerPlugin);
    /// ```
    ///
    /// ### Panics
    /// Panics if a [`Plugin`] of the same type has already been added to this [`App`].
    #[track_caller]
    pub fn add_plugin<P : Plugin + 'static>(&mut self, plugin : P) -> &mut Self {
        if (! self.installed_plugins.insert(TypeId::of::<P>())) {
            panic!("App already has plugin {} installed", type_name::<P>())
        }
        plugin.build(self);
        self
    }

    /// Sets the function that will be called when the [`App`] is run.
    ///
    /// A standard runner can be set by adding the [`CycleSchedulerPlugin`](crate::app::plugin::CycleSchedulerPlugin) to the [`App`].
    ///
    /// ### Examples
    /// ```
    /// use axecs::prelude::*;
    /// # use async_std::main;
    ///
    /// #[main]
    /// async fn main() {
    ///     let mut app = App::new();
    ///     app.set_runner(run_app);
    ///     app.run().await;
    /// }
    ///
    /// async fn run_app(_app : App) -> AppExit {
    ///     println!("Running App!");
    ///     AppExit::Ok
    /// }
    /// ```
    ///
    /// ### Panics
    /// Panics if a runner function has already been set on this [`App`].
    #[track_caller]
    pub fn set_runner<F : AsyncFnOnce(App) -> AppExit + 'static>(&mut self, runner : F) -> &mut Self {
        if let Some(_) = self.runner {
            panic!("App already has a runner");
        }
        self.runner = Some(Box::new(|app| Box::pin(runner(app))));
        self
    }

    /// Adds a [`System`](crate::system::System) to the application under some schedule.
    ///
    /// ### Examples
    /// ```rust
    /// use axecs::prelude::*;
    /// # use async_std::main;
    ///
    /// #[main]
    /// async fn main() {
    ///     let mut app = App::new();
    ///     app.add_plugin(CycleSchedulerPlugin);
    ///     app.add_systems(Cycle, hello_world_system);
    ///     app.run().await;
    /// }
    ///
    /// async fn hello_world_system() {
    ///     println!("Hello, World!");
    /// }
    /// ```
    pub fn add_systems<L : ScheduleLabel + 'static, S : IntoScheduledSystemConfig<'static, Params>, Params : 'static>(&mut self, run_on : L, system : S) -> &mut Self {
        self.schedules.as_mut().expect("App schedules have already been taken").add_systems(run_on, system);
        self
    }

    /// Removes the [`ScheduleStorage`] from this [`App`], returning it.
    ///
    /// This is intended for runner functions and likely should not be used otherwise. See [App::set_runner].
    ///
    /// # Panics
    /// Panics if the schedules have already been taken from this [`App`].
    #[track_caller]
    pub fn take_schedules(&mut self) -> ScheduleStorage {
        mem::replace(&mut self.schedules, None).expect("App schedules have already been taken")
    }

    /// Inserts a [`Resource`] into the application world.
    ///
    /// ### Examples
    /// ```rust
    /// use axecs::prelude::*;
    /// # use async_std::main;
    ///
    /// #[main]
    /// async fn main() {
    ///     let mut app = App::new();
    ///     app.add_plugin(CycleSchedulerPlugin);
    ///     app.insert_resource(MyValue {
    ///         value : 123
    ///     });
    ///     app.add_systems(Cycle, tick_my_value);
    ///     app.run().await;
    /// }
    ///
    /// #[derive(Resource)]
    /// struct MyValue {
    ///     value : usize
    /// }
    ///
    /// async fn tick_my_value(
    ///     mut my_value : Res<&mut MyValue>
    /// ) {
    ///     my_value.value += 1;
    ///     println!("{}", my_value.value);
    /// }
    /// ```
    ///
    /// ### Panics
    /// Panics if a [`Resource`] of the same type has already been added to this [`App`].
    #[track_caller]
    pub fn insert_resource<R : Resource + 'static>(&mut self, resource : R) -> &mut Self {
        if (! self.resources.as_mut().expect("App resources have already been taken").insert(resource)) {
            panic!("App already has resource {} inserted", type_name::<R>());
        }
        self
    }

    /// Removes the [`RawResourceStorage`] from this [`App`], returning it.
    ///
    /// This is intended for runner functions and likely should not be used otherwise. See [App::set_runner].
    ///
    /// # Panics
    /// Panics if the resources have already been taken from this [`App`].
    #[track_caller]
    pub fn take_resources(&mut self) -> RawResourceStorage {
        mem::replace(&mut self.resources, None).expect("App resources have already been taken")
    }

    /// Runs the [`App`].
    ///
    /// This can only be run once. Future attempts to run will panic.
    ///
    /// ### Examples
    /// ```rust
    /// use axecs::prelude::*;
    /// # use async_std::main;
    ///
    /// #[main]
    /// async fn main() {
    ///     let mut app = App::new();
    ///     app.add_plugin(CycleSchedulerPlugin);
    ///     app.run().await;
    /// }
    /// ```
    ///
    /// ### Panics
    /// Panics if the no runner function has been set on this [`App`].
    #[track_caller]
    pub async fn run(mut self) -> AppExit {
        let Some(runner) = self.runner.take() else {
            panic!("App does not have a runner");
        };
        runner(self).await
    }

}


/// The status returned by an [`App`] when it exits.
#[derive(Debug)]
pub enum AppExit {

    /// The [`App`] exited without any problems.
    Ok,

    /// The [`App`] experienced an error.
    Err(Box<dyn Error + Send + Sync>)

}
