//! TODO: Doc comment


use crate::prelude::World;
use crate::system::{ System, ReadOnlySystem, IntoSystem, IntoReadOnlySystem };
use core::marker::PhantomData;
use alloc::sync::Arc;


/// TODO: Doc comment
pub struct IntoMappedSystem<APassed, A, AParams, BPassed, B, Return>
where   A         : IntoSystem<AParams, BPassed>,
        B         : FnMut(BPassed) -> Return,
        A::System : System<BPassed, Passed = APassed>
{

    /// TODO: Doc comment
    pub(super) a : A,

    /// TODO: Doc comment
    pub(super) b : B,

    /// TODO: Doc comment
    pub(super) marker_a : PhantomData<fn(APassed, AParams) -> BPassed>,

    /// TODO: Doc comment
    pub(super) marker_b : PhantomData<fn(BPassed) -> Return>

}

impl<APassed, A, AParams, BPassed, B, Return>
    IntoSystem<(), Return>
    for IntoMappedSystem<APassed, A, AParams, BPassed, B, Return>
where   A         : IntoSystem<AParams, BPassed>,
        B         : FnMut(BPassed) -> Return,
        A::System : System<BPassed, Passed = APassed>
{
    type System = MappedSystem<APassed, A::System, BPassed, B, Return>;

    #[track_caller]
    fn into_system(self) -> Self::System {
        MappedSystem {
            a : self.a.into_system(),
            b : self.b,
            marker_a : PhantomData,
            marker_b : PhantomData
        }
    }

    #[track_caller]
    unsafe fn into_system_unchecked(self) -> Self::System {
        MappedSystem {
            // SAFETY: TODO
            a : unsafe{ self.a.into_system_unchecked() },
            b : self.b,
            marker_a : PhantomData,
            marker_b : PhantomData
        }
    }

}

unsafe impl<APassed, A, AParams, BPassed, B, Return>
    IntoReadOnlySystem<(), Return>
    for IntoMappedSystem<APassed, A, AParams, BPassed, B, Return>
where   A         : IntoReadOnlySystem<AParams, BPassed>,
        B         : FnMut(BPassed) -> Return,
        A::System : ReadOnlySystem<BPassed, Passed = APassed>
{ }


/// TODO: Doc comment
pub struct MappedSystem<APassed, A, BPassed, B, Return>
where   A : System<BPassed, Passed = APassed>,
        B : FnMut(BPassed) -> Return
{
    /// TODO: Doc comment
    a : A,

    /// TODO: Doc comment
    b : B,

    /// TODO: Doc comment
    marker_a : PhantomData<fn(APassed) -> BPassed>,

    /// TODO: Doc comment
    marker_b : PhantomData<fn(BPassed) -> Return>

}

impl<APassed, A, BPassed, B, Return>
    System<Return>
    for MappedSystem<APassed, A, BPassed, B, Return>
where   A : System<BPassed, Passed = APassed>,
        B : FnMut(BPassed) -> Return
{
    type Passed = APassed;

    #[track_caller]
    async unsafe fn acquire_and_run(&mut self, a_passed : Self::Passed, world : Arc<World>) -> Return {
        // SAFETY: TODO
        let b_passed = unsafe{ self.a.acquire_and_run(a_passed, world) }.await;
        (self.b)(b_passed)
    }
}

unsafe impl<APassed, A, BPassed, B, Return>
    ReadOnlySystem<Return>
    for MappedSystem<APassed, A, BPassed, B, Return>
where   A : ReadOnlySystem<BPassed, Passed = APassed>,
        B : FnMut(BPassed) -> Return
{ }
