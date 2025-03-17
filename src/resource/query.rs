//! TODO: Doc comment


use crate::world::World;
use crate::resource::{ self, Resource, ResourceCell };
use crate::system::SystemId;
use crate::query::{ Query, ReadOnlyQuery, QueryAcquireResult, QueryValidator };
use crate::util::rwlock::{ RwLockReadGuard, RwLockWriteGuard };
use core::ops::{ Deref, DerefMut };
use core::any::{ TypeId, type_name };
use core::task::Poll;
use alloc::sync::Arc;


/// TODO: Doc comment
mod __internal {

    /// TODO: Doc comment
    pub trait ResInner {
        /// TODO: Doc comment
        type Guard;
    }

}
use __internal::*;


pub struct Res<R : ResInner> {
    guard : R::Guard
}
unsafe impl<R : ResInner + Send> Send for Res<R> { }
unsafe impl<R : ResInner + Sync> Sync for Res<R> { }


impl<R : Resource> ResInner for &R {
    type Guard = RwLockReadGuard<ResourceCell>;
}

unsafe impl<'l, R : Resource + 'static> Query for Res<&'l R> {
    type Item = Res<&'l R>;
    type State = ();

    fn init_state(_world : Arc<World>, _system_id : Option<SystemId>) -> Self::State { }

    unsafe fn acquire(world : Arc<World>, _state : &mut Self::State) -> Poll<QueryAcquireResult<Self::Item>> {
        match (world.resources().try_read_raw()) {
            Poll::Ready(inner) => {
                let type_id = TypeId::of::<R>();
                match (inner.resources().find(|resource| resource.0 == type_id)) {
                    Some((_, lock)) => match (lock.try_read()) {
                        Poll::Ready(out) => Poll::Ready(QueryAcquireResult::Ready(Res { guard : out })),
                        Poll::Pending    => Poll::Pending
                    },
                    None => Poll::Ready(QueryAcquireResult::DoesNotExist {
                        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                        name : type_name::<resource::marker::Resource<R>>()
                    })
                }
            },
            Poll::Pending => Poll::Pending
        }
    }

    fn validate() -> QueryValidator {
        QueryValidator::of_immutable::<resource::marker::Resource<R>>()
    }

}

unsafe impl<R : Resource + 'static> ReadOnlyQuery for Res<&R> { }

impl<R : Resource> Deref for Res<&R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        // SAFETY: TODO
        unsafe{ self.guard.get_ref::<R>() }
    }
}

impl<R : Resource> Res<&R> {
    pub fn as_static(self) -> Res<&'static R> {
        Res { guard : self.guard }
    }
}


impl<R : Resource> ResInner for &mut R {
    type Guard = RwLockWriteGuard<ResourceCell>;
}

unsafe impl<'l, R : Resource + 'static> Query for Res<&'l mut R> {
    type Item = Res<&'l mut R>;
    type State = ();

    fn init_state(_world : Arc<World>, _system_id : Option<SystemId>) -> Self::State { }

    unsafe fn acquire(world : Arc<World>, _state : &mut Self::State) -> Poll<QueryAcquireResult<Self::Item>> {
        match (world.resources().try_read_raw()) {
            Poll::Ready(inner) => {
                let type_id = TypeId::of::<R>();
                match (inner.resources().find(|resource| resource.0 == type_id)) {
                    Some((_, lock)) => match (lock.try_write()) {
                        Poll::Ready(out) => Poll::Ready(QueryAcquireResult::Ready(Res { guard : out })),
                        Poll::Pending    => Poll::Pending
                    },
                    None => Poll::Ready(QueryAcquireResult::DoesNotExist {
                        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                        name : type_name::<resource::marker::Resource<R>>()
                    })
                }
            },
            Poll::Pending => Poll::Pending
        }
    }

    fn validate() -> QueryValidator {
        QueryValidator::of_immutable::<resource::marker::Resource<R>>()
    }

}

impl<R : Resource> Deref for Res<&mut R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        // SAFETY: TODO
        unsafe{ self.guard.get_ref::<R>() }
    }
}

impl<R : Resource> DerefMut for Res<&mut R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: TODO
        unsafe{ self.guard.get_mut::<R>() }
    }
}

impl<R : Resource> Res<&mut R> {
    pub fn as_static(self) -> Res<&'static mut R> {
        Res { guard : self.guard }
    }
}
