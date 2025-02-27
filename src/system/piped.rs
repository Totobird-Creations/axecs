//! TODO: Doc comment


use crate::world::World;
use crate::system::{ SystemId, System, ReadOnlySystem, IntoSystem, IntoReadOnlySystem, SystemPassable };
use core::any::TypeId;
use core::marker::PhantomData;
use core::ops::{ Deref, DerefMut };
use alloc::sync::Arc;
use alloc::collections::BTreeSet;


pub struct IntoPipedSystem<APassed, A, AParams, BPassed, B, BParams, Return>
where   A         : IntoSystem<AParams, BPassed>,
        B         : IntoSystem<BParams, Return>,
        A::System : System<BPassed, Passed = APassed>,
        B::System : System<Return, Passed = In<BPassed>>
{

    /// TODO: Doc comment
    pub(super) a : A,

    /// TODO: Doc comment
    pub(super) b : B,

    /// TODO: Doc comment
    pub(super) marker_a : PhantomData<fn(APassed, AParams) -> BPassed>,

    /// TODO: Doc comment
    pub(super) marker_b : PhantomData<fn(BPassed, BParams) -> Return>

}

impl<APassed, A, AParams, BPassed, B, BParams, Return>
    IntoSystem<(), Return>
    for IntoPipedSystem<APassed, A, AParams, BPassed, B, BParams, Return>
where   A         : IntoSystem<AParams, BPassed>,
        B         : IntoSystem<BParams, Return>,
        A::System : System<BPassed, Passed = APassed>,
        B::System : System<Return, Passed = In<BPassed>>
{
    type System = PipedSystem<APassed, A::System, BPassed, B::System, Return>;

    fn extend_scheduled_system_config_ids(ids : &mut BTreeSet<TypeId>) {
        A::extend_scheduled_system_config_ids(ids);
        B::extend_scheduled_system_config_ids(ids);
    }

    #[track_caller]
    fn into_system(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System {
        PipedSystem {
            a : self.a.into_system(Arc::clone(&world), system_id),
            b : self.b.into_system(world, system_id),
            marker_a : PhantomData,
            marker_b : PhantomData
        }
    }

    #[track_caller]
    unsafe fn into_system_unchecked(self, world : Arc<World>, system_id : Option<SystemId>) -> Self::System {
        PipedSystem {
            // SAFETY: TODO
            a : unsafe{ self.a.into_system_unchecked(Arc::clone(&world), system_id) },
            // SAFETY: TODO
            b : unsafe{ self.b.into_system_unchecked(world, system_id) },
            marker_a : PhantomData,
            marker_b : PhantomData
        }
    }
}

unsafe impl<APassed, A, AParams, BPassed, B, BParams, Return>
    IntoReadOnlySystem<(), Return>
    for IntoPipedSystem<APassed, A, AParams, BPassed, B, BParams, Return>
where   A         : IntoReadOnlySystem<AParams, BPassed>,
        B         : IntoReadOnlySystem<BParams, Return>,
        A::System : ReadOnlySystem<BPassed, Passed = APassed>,
        B::System : ReadOnlySystem<Return, Passed = In<BPassed>>
{ }


/// TODO: Doc comment
pub struct PipedSystem<APassed, A, BPassed, B, Return>
where   A : System<BPassed, Passed = APassed>,
        B : System<Return, Passed = In<BPassed>>
{

    /// TODO: Doc comment
    a        : A,

    /// TODO: Doc comment
    b        : B,

    /// TODO: Doc comment
    marker_a : PhantomData<fn(APassed) -> BPassed>,

    /// TODO: Doc comment
    marker_b : PhantomData<fn(BPassed) -> Return>

}

impl<APassed, A, BPassed, B, Return>
    System<Return>
    for PipedSystem<APassed, A, BPassed, B, Return>
where   A : System<BPassed, Passed = APassed>,
        B : System<Return, Passed = In<BPassed>>
{
    type Passed = APassed;

    #[track_caller]
    async unsafe fn acquire_and_run(&mut self, a_passed : Self::Passed, world : Arc<World>) -> Return {
        // SAFETY: TODO
        let b_passed = unsafe{ self.a.acquire_and_run(a_passed, Arc::clone(&world)) }.await;
        // SAFETY: TODO
        unsafe{ self.b.acquire_and_run(In(b_passed), world) }.await
    }
}

unsafe impl<APassed, A, BPassed, B, Return>
    ReadOnlySystem<Return>
    for PipedSystem<APassed, A, BPassed, B, Return>
where   A : ReadOnlySystem<BPassed, Passed = APassed>,
        B : ReadOnlySystem<Return, Passed = In<BPassed>>
{ }


/// TODO: Doc comment
pub struct In<T> (

    /// TODO: Doc comment
    pub T

);

impl<T> SystemPassable for In<T> { }

impl<T> Deref for In<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for In<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
