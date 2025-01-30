//! TODO: Doc commands


use crate::world::World;
use crate::component::query::{ ComponentQuery, ReadOnlyComponentQuery, ComponentFilter, True };
use crate::component::archetype::{ Archetype, ArchetypeStorage };
use crate::query::{ Query, ReadOnlyQuery, QueryAcquireResult, QueryValidator };
use crate::util::rwlock::RwLockWriteGuard;
use core::task::Poll;
use core::marker::PhantomData;
use alloc::vec::Vec;


/// TODO: Doc comments
pub struct Entities<'l, Q : ComponentQuery, F : ComponentFilter = True> {

    /// TODO: Doc comments
    archetypes : Vec<RwLockWriteGuard<Archetype>>,

    /// TODO: Doc comments
    marker     : PhantomData<(&'l Q, fn(F) -> bool)>

}

impl<Q : ComponentQuery, F : ComponentFilter> Entities<'_, Q, F> {

    /// TODO: Doc comments
    ///
    /// # Safety
    /// The caller is responsible for ensuring that `Q` follows the archetype rules. See [`BundleValidator`](crate::component::bundle::BundleValidator).
    pub(crate) unsafe fn acquire_archetypes_unchecked<'l>(archetypes : &'l ArchetypeStorage) -> Poll<Entities<'l, Q, F>> {
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

unsafe impl<Q : ComponentQuery + 'static, F : ComponentFilter> Query for Entities<'_, Q, F> {
    type Item<'world, 'state> = Entities<'world, Q, F>;

    fn init_state() -> Self::State { () }

    unsafe fn acquire<'world, 'state>(world : &'world World, _state : &'state mut Self::State) -> Poll<QueryAcquireResult<Self::Item<'world, 'state>>> {
        // SAFETY: TODO
        unsafe{ Self::acquire_archetypes_unchecked(world.archetypes()) }.map(|out| QueryAcquireResult::Ready(out))
    }

    fn validate() -> QueryValidator {
        <Q as ComponentQuery>::validate()
    }

}

unsafe impl<'l, Q : ReadOnlyComponentQuery + 'static, F : ComponentFilter> ReadOnlyQuery for Entities<'l, Q, F> { }


impl<'l, Q : ComponentQuery, F : ComponentFilter> Entities<'l, Q, F> {

    /// TODO: Doc comments
    pub fn iter(&'l self) -> impl Iterator<Item = Q::Item<'l>> {
        <&Self as IntoIterator>::into_iter(self)
    }

    /// TODO: Doc comments
    pub fn iter_mut(&'l mut self) -> impl Iterator<Item = Q::ItemMut<'l>> {
        <&mut Self as IntoIterator>::into_iter(self)
    }

}


impl<'l, 'k, Q : ComponentQuery, F : ComponentFilter> IntoIterator for &'l Entities<'k, Q, F> {
    type Item     = Q::Item<'l>;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // SAFETY: TODO
        self.archetypes.iter().map(|archetype| unsafe{ Q::iter_rows_ref(archetype) }.unwrap()).flatten()
    }
}


impl<'l, 'k, Q : ComponentQuery, F : ComponentFilter> IntoIterator for &'l mut Entities<'k, Q, F> {
    type Item     = Q::ItemMut<'l>;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // SAFETY: TODO
        self.archetypes.iter_mut().map(|archetype| unsafe{ Q::iter_rows_mut(archetype) }.unwrap()).flatten()
    }
}
