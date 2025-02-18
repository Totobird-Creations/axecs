//! TODO: Doc commands


use crate::world::World;
use crate::component::query::{ ComponentQuery, ReadOnlyComponentQuery, ComponentFilter, True };
use crate::component::archetype::{ ArchetypeStorage, Archetype };
use crate::query::{ Query, ReadOnlyQuery, QueryAcquireResult, QueryValidator };
use crate::util::rwlock::RwLockWriteGuard;
use core::task::Poll;
use core::marker::PhantomData;
use alloc::vec::Vec;
use alloc::sync::Arc;


/// TODO: Doc comments
pub struct Entities<Q : ComponentQuery, F : ComponentFilter = True> {

    /// TODO: Doc comments
    archetypes : Vec<RwLockWriteGuard<Archetype>>,

    /// TODO: Doc comments
    marker     : PhantomData<(Q, fn(F) -> bool)>

}

impl<Q : ComponentQuery, F : ComponentFilter> Entities<Q, F> {

    /// TODO: Doc comments
    ///
    /// # Safety
    /// The caller is responsible for ensuring that `Q` follows the archetype rules. See [`BundleValidator`](crate::component::bundle::BundleValidator).
    pub(crate) unsafe fn acquire_archetypes_unchecked(archetypes : &ArchetypeStorage) -> Poll<Entities<Q, F>> {
        match (archetypes.try_read_raw()) {
            Poll::Ready(inner) => {
                match (inner.archetype_components()
                    .filter_map(|(components, archetype_id)| {
                        (F::archetype_matches(components) && Q::is_subset_of_archetype(components))
                            // SAFETY: TODO
                            .then(|| match (unsafe{ archetypes.get_mut_by_id_unchecked(archetype_id) }) {
                                Poll::Ready(out) => Some(out),
                                Poll::Pending    => None
                            })
                    })
                    .collect::<Option<Vec<_>>>()
                ) {
                    Some(out) => Poll::Ready(Entities {
                        archetypes : out,
                        marker     : PhantomData
                    }),
                    None => Poll::Pending,
                }
            },
            Poll::Pending => Poll::Pending
        }
    }

}

unsafe impl<Q : ComponentQuery + 'static, F : ComponentFilter> Query for Entities<Q, F> {
    type Item = Entities<Q, F>;

    fn init_state() -> Self::State { () }

    unsafe fn acquire(world : Arc<World>, _state : &mut Self::State) -> Poll<QueryAcquireResult<Self::Item>> {
        // SAFETY: TODO
        unsafe{ Self::acquire_archetypes_unchecked(world.archetypes()) }.map(|out| QueryAcquireResult::Ready(out))
    }

    fn validate() -> QueryValidator {
        <Q as ComponentQuery>::validate()
    }

}

unsafe impl<Q : ReadOnlyComponentQuery + 'static, F : ComponentFilter> ReadOnlyQuery for Entities<Q, F> { }


impl<Q : ComponentQuery, F : ComponentFilter> Entities<Q, F> {

    /// TODO: Doc comments
    pub fn iter(&self) -> impl Iterator<Item = Q::Item<'_>> {
        <&Self as IntoIterator>::into_iter(self)
    }

    /// TODO: Doc comments
    pub fn iter_mut(&mut self) -> impl Iterator<Item = Q::ItemMut<'_>> {
        <&mut Self as IntoIterator>::into_iter(self)
    }

}


impl<'l, Q : ComponentQuery, F : ComponentFilter> IntoIterator for &'l Entities<Q, F> {
    type Item     = Q::Item<'l>;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // SAFETY: TODO
        self.archetypes.iter().map(|archetype| archetype.rows().map(|row|
            unsafe{ Q::get_row_ref(archetype, row).unwrap_unchecked() }
        )).flatten()
    }
}


impl<'l, Q : ComponentQuery, F : ComponentFilter> IntoIterator for &'l mut Entities<Q, F> {
    type Item     = Q::ItemMut<'l>;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // SAFETY: TODO
        self.archetypes.iter_mut().map(|archetype| archetype.rows().map(|row|
            unsafe{ Q::get_row_mut(archetype, row).unwrap_unchecked() }
        )).flatten()
    }
}
