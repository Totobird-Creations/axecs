//! TODO: Doc comment


use crate::world::World;
use crate::system::{ SystemId, System, ReadOnlySystem, IntoSystem, IntoReadOnlySystem };
use core::any::TypeId;
use core::marker::PhantomData;
use alloc::sync::Arc;
use alloc::collections::BTreeSet;


pub struct IntoSeriesSystem<APassed, A, AParams, B, BParams, Return>
where   A         : IntoSystem<AParams, ()>,
        B         : IntoSystem<BParams, Return>,
        A::System : System<(), Passed = APassed>,
        B::System : System<Return, Passed = ()>
{

    /// TODO: Doc comment
    pub(super) a : A,

    /// TODO: Doc comment
    pub(super) b : B,

    /// TODO: Doc comment
    pub(super) marker_a : PhantomData<fn(APassed, AParams) -> ()>,

    /// TODO: Doc comment
    pub(super) marker_b : PhantomData<fn(BParams) -> Return>

}

impl<APassed, A, AParams, B, BParams, Return>
    IntoSystem<(), Return>
    for IntoSeriesSystem<APassed, A, AParams, B, BParams, Return>
where   A         : IntoSystem<AParams, ()>,
        B         : IntoSystem<BParams, Return>,
        A::System : System<(), Passed = APassed>,
        B::System : System<Return, Passed = ()>
{
    type System = SeriesSystem<APassed, A::System, B::System, Return>;

    fn extend_scheduled_system_config_ids(ids : &mut BTreeSet<TypeId>) {
        A::extend_scheduled_system_config_ids(ids);
        B::extend_scheduled_system_config_ids(ids);
    }

    #[track_caller]
    fn into_system(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System {
        SeriesSystem {
            a : self.a.into_system(Arc::clone(&world), system_id),
            b : self.b.into_system(world, system_id),
            marker_a : PhantomData,
            marker_b : PhantomData
        }
    }

    #[track_caller]
    unsafe fn into_system_unchecked(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System {
        SeriesSystem {
            // SAFETY: TODO
            a : unsafe{ self.a.into_system_unchecked(Arc::clone(&world), system_id) },
            // SAFETY: TODO
            b : unsafe{ self.b.into_system_unchecked(world, system_id) },
            marker_a : PhantomData,
            marker_b : PhantomData
        }
    }
}

unsafe impl<APassed, A, AParams, B, BParams, Return>
    IntoReadOnlySystem<(), Return>
    for IntoSeriesSystem<APassed, A, AParams, B, BParams, Return>
where   A         : IntoReadOnlySystem<AParams, ()>,
        B         : IntoReadOnlySystem<BParams, Return>,
        A::System : ReadOnlySystem<(), Passed = APassed>,
        B::System : ReadOnlySystem<Return, Passed = ()>
{ }


/// TODO: Doc comment
pub struct SeriesSystem<APassed, A, B, Return>
where   A : System<(), Passed = APassed>,
        B : System<Return, Passed = ()>
{

    /// TODO: Doc comment
    a        : A,

    /// TODO: Doc comment
    b        : B,

    /// TODO: Doc comment
    marker_a : PhantomData<fn(APassed) -> ()>,

    /// TODO: Doc comment
    marker_b : PhantomData<fn() -> Return>

}

impl<APassed, A, B, Return>
    System<Return>
    for SeriesSystem<APassed, A, B, Return>
where   A : System<(), Passed = APassed>,
        B : System<Return, Passed = ()>
{
    type Passed = APassed;

    #[track_caller]
    async unsafe fn acquire_and_run(&mut self, a_passed : Self::Passed, world : Arc<World>) -> Return {
        // SAFETY: TODO
        unsafe{ self.a.acquire_and_run(a_passed, Arc::clone(&world)) }.await;
        // SAFETY: TODO
        unsafe{ self.b.acquire_and_run((), world) }.await
    }
}

unsafe impl<APassed, A, B, Return>
    ReadOnlySystem<Return>
    for SeriesSystem<APassed, A, B, Return>
where   A : ReadOnlySystem<(), Passed = APassed>,
        B : ReadOnlySystem<Return, Passed = ()>
{ }
