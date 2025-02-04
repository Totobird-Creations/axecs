//! TODO: Doc comment


use crate::app::{ App, AppExit };
use crate::app::plugin::Plugin;
use crate::world::World;
use crate::resource::ResourceStorage;
use crate::schedule::ScheduleStorage;
use crate::schedule::label::{ ScheduleLabel, Startup, Update, Shutdown };
use crate::schedule::system::TypeErasedSystem;
use crate::util::rwlock::RwLockWriteGuard;
use crate::util::sparsevec::SparseVec;
use core::pin::Pin;
use core::task::{ Context, Poll };
use core::mem::MaybeUninit;
use core::cell::UnsafeCell;
use alloc::boxed::Box;
use alloc::vec::Vec;


/// TODO: Doc comment
pub struct CycleSchedulerPlugin {

}

impl Default for CycleSchedulerPlugin {
    fn default() -> Self {
        Self {  }
    }
}


impl Plugin for CycleSchedulerPlugin {
    fn build(self, app : &mut App) {
        app.set_runner(Self::run);
    }
}


impl CycleSchedulerPlugin {


    /// TODO: Doc comments
    async fn run(mut app : App) -> AppExit {
        let world     = World::new_with( ResourceStorage::new_with(app.take_resources()) );
        let schedules = app.take_schedules();

        let scheduler = CycleSchedulerFuture::new(&world, &schedules);
        scheduler.await
    }

}



struct CycleSchedulerFuture<'l> {

    /// TODO: Doc comments
    world        : &'l World,

    /// TODO: Doc comments
    schedules    : &'l ScheduleStorage,

    /// TODO: Doc comments
    futures      : SparseVec<Pin<Box<dyn Future<Output = ()> + 'l>>>,

    /// TODO: Doc comments
    shutdown_ran : bool,

}

impl<'l> CycleSchedulerFuture<'l> {

    /// TODO: Doc comments
    fn new(world : &'l World, schedules : &'l ScheduleStorage) -> Self {
        let mut scheduler = Self {
            world,
            schedules,
            futures      : SparseVec::new(),
            shutdown_ran : false
        };
        scheduler.run_label_oneshot(Startup);
        scheduler.run_label_cycle(Update);
        scheduler
    }

    /// TODO: Doc comments
    fn run_label_oneshot<S : ScheduleLabel + 'static>(&mut self, label : S) {
        self.futures.append(&mut self.schedules.get_schedule(label)
            .into_iter().map::<_, _>(|system| Box::pin(async{
                let mut system = system.write().await;
                // SAFETY: TODO
                unsafe{ system.acquire_and_run((), self.world) }.await;
            }) as _)
            .collect::<Vec<_>>()
        );
    }

    /// TODO: Doc comments
    fn run_label_cycle<S : ScheduleLabel + 'static>(&mut self, label : S) {
        self.futures.append(&mut self.schedules.get_schedule(label)
            .into_iter().map::<_, _>(|system| Box::pin(async {
                SystemCycleFuture::new(self.world, system.write().await).await
            }) as _)
            .collect::<Vec<_>>()
        );
    }

}

impl<'l> Unpin for CycleSchedulerFuture<'l> { }

impl<'l> Future for CycleSchedulerFuture<'l> {
    type Output = AppExit;

    fn poll(mut self : Pin<&mut Self>, ctx : &mut Context<'_>) -> Poll<Self::Output> {
        let is_exiting = self.world.is_exiting();

        if (is_exiting && ! self.shutdown_ran) {
            self.shutdown_ran = true;
            self.run_label_oneshot(Shutdown);
        }

        self.futures.retain(|fut| {
            fut.as_mut().poll(ctx).is_pending()
        });

        if (self.futures.is_empty() && is_exiting) {
            Poll::Ready(self.world.take_exit_status())
        } else { Poll::Pending }
    }
}


/// TODO: Doc comments
struct SystemCycleFuture<'l> {

    /// TODO: Doc comments
    world  : &'l World,

    /// TODO: Doc comments
    system : UnsafeCell<RwLockWriteGuard<Box<dyn TypeErasedSystem<(), ()>>>>,

    /// TODO: Doc comments
    future : MaybeUninit<Pin<Box<dyn Future<Output = ()> + 'l>>>

}

impl<'l> SystemCycleFuture<'l> {

    /// TODO: Doc comments
    fn new(world : &'l World, system : RwLockWriteGuard<Box<dyn TypeErasedSystem<(), ()>>>) -> Self {
        let mut cycle = Self {
            world,
            system : UnsafeCell::new(system),
            future : MaybeUninit::uninit()
        };
        cycle.future.write(Box::pin(Self::cycle(
            cycle.world,
            // SAFETY: TODO
            unsafe{ &mut*cycle.system.get() }.as_mut()
        )));
        cycle
    }

    /// TODO: Doc comments
    async fn cycle(world : &'l World, system : &'l mut dyn TypeErasedSystem<(), ()>) {
        // SAFETY: TODO
        unsafe{ system.acquire_and_run((), world) }.await;
    }

}

impl<'l> Future for SystemCycleFuture<'l> {
    type Output = ();

    fn poll(mut self : Pin<&mut Self>, ctx : &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: TODO
        if let Poll::Ready(_) = unsafe{ self.future.assume_init_mut() }.as_mut().poll(ctx) {
            let world = self.world;
            if (world.is_exiting()) { return Poll::Ready(()); }
            // SAFETY: TODO
            let system = unsafe{ &mut*self.system.get() }.as_mut();
            self.future.write(Box::pin(Self::cycle(world, system)));
            ctx.waker().wake_by_ref();
        };
        Poll::Pending
    }
}

impl<'l> Drop for SystemCycleFuture<'l> {
    fn drop(&mut self) {
        // SAFETY: TODO
        unsafe{ self.future.assume_init_drop(); }
    }
}
