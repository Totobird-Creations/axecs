//! A good default system scheduler.


use crate::app::{ App, AppExit };
use crate::app::plugin::Plugin;
use crate::world::World;
use crate::resource::ResourceStorage;
use crate::schedule::ScheduleStorage;
use crate::schedule::label::{ ScheduleLabel, Always, PreStartup, Startup, Cycle, Shutdown, PostShutdown };
use crate::schedule::system::TypeErasedSystem;
use crate::util::rwlock::{RwLock, RwLockWriteGuard};
use crate::util::sparsevec::SparseVec;
use core::pin::Pin;
use core::task::{ Context, Poll };
use core::mem::MaybeUninit;
use core::cell::UnsafeCell;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::sync::Arc;


/// A good default system scheduler.
///
/// The cycle scheduler will use [`PreStartup`], [`Startup`], [`Cycle`], [`Shutdown`], and [`PostShutdown`].
/// See the documentations for each individual label for more details.
///
/// # Examples
/// ```rust
/// use axecs::prelude::*;
/// # use async_std::main;
///
/// #[main]
/// async fn main() {
///
///     let mut app = App::new();
///
///     app.add_plugin(CycleSchedulerPlugin);
///
///     app.add_systems(PreStartup, setup);
///     app.add_systems(Cycle, update);
///
///     app.run().await;
///
/// }
///
/// async fn setup() {
///     println!("Hello!");
/// }
///
/// async fn update() {
///     println!("Tick");
/// }
/// ```
pub struct CycleSchedulerPlugin;

impl Default for CycleSchedulerPlugin {
    fn default() -> Self {
        Self {  }
    }
}


impl Plugin for CycleSchedulerPlugin {
    fn build(self, app : &mut App) {
        app.set_runner(move |app| self.run(app));
    }
}


impl CycleSchedulerPlugin {

    /// Runs the application using this cycle scheduler.
    async fn run(self, mut app : App) -> AppExit {
        let world     = Arc::new(World::new_with( ResourceStorage::new_with(app.take_resources()) ));
        let schedules = Arc::new(app.take_schedules());

        let scheduler = CycleSchedulerFuture::new(world, schedules);
        scheduler.await
    }

}



/// A [`Future`] which handles running schedules as needed.
struct CycleSchedulerFuture {

    /// The current state of this scheduler.
    state          : CycleSchedulerState,

    /// The [`World`] to operate on.
    world          : Arc<World>,

    /// The schedules in the app.
    schedules      : Arc<ScheduleStorage>,

    /// The currently running [`Always`] futures.
    always_futures : Vec<Pin<Box<dyn Future<Output = ()>>>>,

    /// The currently running futures.
    futures        : SparseVec<Pin<Box<dyn Future<Output = ()>>>>

}

/// The current state of this [`CycleSchedulerFuture`].
enum CycleSchedulerState {

    /// This scheduler was just created and needs to be set up.
    /// Switch to [`PreStartup`](Self::PreStartup).
    Init,

    /// The scheduler is currently running [`PreStartup`] systems.
    /// Once that is done, switch to [`Main`](Self::Main).
    PreStartup,

    /// The scheduler is currently running [`Startup`] and [`Cycle`] systems.
    /// When the app begins to exit, switch to [`Shutdown`](Self::Shutdown).
    Main,

    /// The scheduler is currently running [`Shutdown`] systems, and finishing up [`Startup`] and [`Cycle`] systems.
    /// Once all are done, switch to [`PostShutdown`](Self::PostShutdown).
    Shutdown,

    /// The scheduler is currently running [`PostShutdown`] systems.
    /// Once all are done, exit the app.
    PostShutdown
}

impl CycleSchedulerFuture {

    /// Creates a new [`CycleSchedulerFuture`] from a [`World`] and some [`System`](crate::system::System)s.
    fn new(world : Arc<World>, schedules : Arc<ScheduleStorage>) -> Self {
        Self {
            state          : CycleSchedulerState::Init,
            world,
            schedules,
            always_futures : Vec::new(),
            futures        : SparseVec::new()
        }
    }

    /// TODO: Doc comment
    fn run_label_always<L : ScheduleLabel + 'static>(&mut self, label : L) {
        self.always_futures.append(&mut self.schedules.get_schedule(label)
            .into_iter().map(|system| {
                let world  = Arc::clone(&self.world);
                let system = RwLock::arc_clone(&system);
                Box::pin(async move {
                    SystemCycleFuture::new(world, system.write().await).await
                }) as _
            })
            .collect::<Vec<_>>()
        );
    }

    /// Adds every system under the given label to the running futures.
    fn run_label_oneshot<L : ScheduleLabel + 'static>(&mut self, label : L) {
        self.futures.append(&mut self.schedules.get_schedule(label)
            .into_iter().map(|system| {
                let world  = Arc::clone(&self.world);
                let system = RwLock::arc_clone(&system);
                Box::pin(async move {
                    let mut system = system.write().await;
                    // SAFETY: `ScheduleStorage::add_systems` is the only way to add systems.
                    //         It takes a value implementing `IntoScheduledSystemConfig`. The
                    //         implementors of `IntoScheduledSystemConfig` are responsible for
                    //         ensuring that this system is valid.
                    unsafe{ system.acquire_and_run((), Arc::clone(&world)) }.await;
                }) as _
            })
            .collect::<Vec<_>>()
        );
    }

    /// Adds every system under the given label to the running futures.
    ///
    /// These systems will be wrapped in [`SystemCycleFuture`], and will loop until the app begins to exit.
    fn run_label_cycle<L : ScheduleLabel + 'static>(&mut self, label : L) {
        self.futures.append(&mut self.schedules.get_schedule(label)
            .into_iter().map(|system| {
                let world  = Arc::clone(&self.world);
                let system = RwLock::arc_clone(&system);
                Box::pin(async move {
                    SystemCycleFuture::new(Arc::clone(&world), system.write().await).await
                }) as _
            })
            .collect::<Vec<_>>()
        );
    }

}

impl Unpin for CycleSchedulerFuture { }

impl Future for CycleSchedulerFuture {
    type Output = AppExit;

    fn poll(mut self : Pin<&mut Self>, ctx : &mut Context<'_>) -> Poll<Self::Output> {

        self.always_futures.retain_mut(|fut| {
            fut.as_mut().poll(ctx).is_pending()
        });

        self.futures.retain(|fut| {
            fut.as_mut().poll(ctx).is_pending()
        });

        if let Poll::Ready(mut cmd_queue) = self.world.cmd_queue.try_write() {
            for cmd in cmd_queue.drain(..) {
                let fut = cmd(Arc::clone(&self.world));
                self.futures.push(fut);
            }
        }

        match (self.state) {

            CycleSchedulerState::Init => {
                self.run_label_always(Always);
                self.run_label_oneshot(PreStartup);
                self.state = CycleSchedulerState::PreStartup;
                ctx.waker().wake_by_ref();
            }

            CycleSchedulerState::PreStartup => {
                if (self.futures.is_empty()) {
                    self.run_label_oneshot(Startup);
                    self.run_label_cycle(Cycle);
                    self.state = CycleSchedulerState::Main;
                    ctx.waker().wake_by_ref();
                }
            },

            CycleSchedulerState::Main => {
                if (self.world.is_exiting()) {
                    self.run_label_oneshot(Shutdown);
                    self.state = CycleSchedulerState::Shutdown;
                    ctx.waker().wake_by_ref();
                }
            },

            CycleSchedulerState::Shutdown => {
                if (self.futures.is_empty()) {
                    self.run_label_oneshot(PostShutdown);
                    self.state = CycleSchedulerState::PostShutdown;
                    ctx.waker().wake_by_ref();
                }
            },

            CycleSchedulerState::PostShutdown => {
                if (self.futures.is_empty()) {
                    return Poll::Ready(self.world.take_exit_status())
                }
            }

        }

        Poll::Pending
    }
}


/// A [`Future`] that runs a [`System`](crate::system::System) repeatedly until the app begins to exit.
struct SystemCycleFuture {

    /// The world to operate on.
    world  : Arc<World>,

    /// The system to loop repeatedly.
    system : UnsafeCell<RwLockWriteGuard<Box<dyn TypeErasedSystem<(), ()>>>>,

    /// The currently running future.
    future : MaybeUninit<Pin<Box<dyn Future<Output = ()>>>>

}

impl SystemCycleFuture {

    /// Create a new [`SystemCycleFuture`] from a [`World`] and [`TypeErasedSystem`].
    fn new(world : Arc<World>, system : RwLockWriteGuard<Box<dyn TypeErasedSystem<(), ()>>>) -> Self {
        let mut cycle = Self {
            world,
            system : UnsafeCell::new(system),
            future : MaybeUninit::uninit()
        };
        cycle.future.write(Box::pin(Self::cycle(
            Arc::clone(&cycle.world),
            // SAFETY: Nothing else is accessing `cycle.system`, as it was just created.
            unsafe{ &mut*cycle.system.get() }.as_mut()
        )));
        cycle
    }

    /// Run the given system one time.
    async fn cycle(world : Arc<World>, system : &mut dyn TypeErasedSystem<(), ()>) {
        // SAFETY: `ScheduleStorage::add_systems` is the only way to add systems.
        //         It takes a value implementing `IntoScheduledSystemConfig`. The
        //         implementors of `IntoScheduledSystemConfig` are responsible for
        //         ensuring that this system is valid.
        unsafe{ system.acquire_and_run((), world) }.await;
    }

}

impl Future for SystemCycleFuture {
    type Output = ();

    fn poll(mut self : Pin<&mut Self>, ctx : &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: `self.future` is always initialised.
        if let Poll::Ready(_) = unsafe{ self.future.assume_init_mut() }.as_mut().poll(ctx) {
            ctx.waker().wake_by_ref();
            let world = Arc::clone(&self.world);
            if (world.is_exiting()) { return Poll::Ready(()); }
            // SAFETY: `self.future` is always initialised.
            //         It will be re-written immediately below.
            unsafe{ self.future.assume_init_drop() };
            // SAFETY: `self` is borrowed mutably, and the previous reference to
            //         `self.system` was in `self.future`, dropped in the line above.
            let system = unsafe{ &mut*self.system.get() }.as_mut();
            self.future.write(Box::pin(Self::cycle(world, system)));
        };
        Poll::Pending
    }
}

impl Drop for SystemCycleFuture {
    fn drop(&mut self) {
        // `self.future` is always initialised.
        unsafe{ self.future.assume_init_drop(); }
    }
}
