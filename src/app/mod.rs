//! TODO: Doc comment


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


/// TODO: Doc comment
pub struct App {

    /// TODO: Doc comment
    installed_plugins : BTreeSet<TypeId>,

    /// TODO: Doc comment
    runner            : Option<Box<dyn FnOnce(App) -> Pin<Box<dyn Future<Output = AppExit>>>>>,

    /// TODO: Doc comment
    schedules         : Option<ScheduleStorage>,

    /// TODO: Doc comment
    resources         : Option<RawResourceStorage>

}

impl App {

    /// TODO: Doc comment
    pub fn new() -> Self { Self {
        installed_plugins : BTreeSet::new(),
        runner            : None,
        schedules         : Some(ScheduleStorage::new()),
        resources         : Some(RawResourceStorage::new())
    } }

    /// TODO: Doc comment
    #[track_caller]
    pub fn add_plugin<P : Plugin + 'static>(&mut self, plugin : P) -> &mut Self {
        if (! self.installed_plugins.insert(TypeId::of::<P>())) {
            panic!("App already has plugin {} installed", type_name::<P>())
        }
        plugin.build(self);
        self
    }

    /// TODO: Doc comment
    #[track_caller]
    pub fn set_runner<F : AsyncFnOnce(App) -> AppExit + 'static>(&mut self, runner : F) -> &mut Self {
        if let Some(_) = self.runner {
            panic!("App already has a runner");
        }
        self.runner = Some(Box::new(|app| Box::pin(runner(app))));
        self
    }

    /// TODO: Doc comment
    pub fn add_systems<L : ScheduleLabel + 'static, S : IntoScheduledSystemConfig<'static, Params>, Params>(&mut self, run_on : L, system : S) -> &mut Self {
        self.schedules.as_mut().expect("App schedules have already been taken").add_systems(run_on, system);
        self
    }

    /// TODO: Doc comment
    #[track_caller]
    pub fn take_schedules(&mut self) -> ScheduleStorage {
        mem::replace(&mut self.schedules, None).expect("App schedules have already been taken")
    }

    /// TODO: Doc comment
    #[track_caller]
    pub fn insert_resource<R : Resource + 'static>(&mut self, resource : R) -> &mut Self {
        if (! self.resources.as_mut().expect("App resources have already been taken").insert(resource)) {
            panic!("App already has resource {} inserted", type_name::<R>());
        }
        self
    }

    /// TODO: Doc comment
    #[track_caller]
    pub fn take_resources(&mut self) -> RawResourceStorage {
        mem::replace(&mut self.resources, None).expect("App resources have already been taken")
    }

    /// TODO: Doc comment
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
    Err(Box<dyn Error>)

}
