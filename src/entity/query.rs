//! TODO: Doc commands


use crate::world::World;
use crate::component::query::{ ComponentQuery, ReadOnlyComponentQuery, ComponentFilter, True };
use crate::component::archetype::{ ArchetypeStorage, Archetype };
use crate::query::{ Query, ReadOnlyQuery, QueryAcquireResult, QueryValidator };
use crate::util::rwlock::RwLockWriteGuard;
use core::task::Poll;
use core::ops::{ Deref, DerefMut };
use core::mem::MaybeUninit;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use alloc::vec::Vec;
use alloc::sync::Arc;


/// TODO: Doc comments
pub struct Entities<Q : ComponentQuery, F : ComponentFilter = True> {

    /// TODO: Doc comments
    archetypes : Vec<RwLockWriteGuard<Archetype>>,

    /// TODO: Doc comments
    marker_a  : PhantomData<fn(&Archetype, usize) -> Q::ItemMut<'static>>,

    /// TODO: Doc comments
    marker_b   : PhantomData<fn(F) -> bool>

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
                        marker_a   : PhantomData,
                        marker_b   : PhantomData
                    }),
                    None => Poll::Pending,
                }
            },
            Poll::Pending => Poll::Pending
        }
    }


    /// TODO: Doc comments
    pub fn as_static(self) -> Entities<Q::AsStatic, F> {
        Entities {
            archetypes : self.archetypes,
            marker_a   : PhantomData,
            marker_b   : PhantomData
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
        self.archetypes.iter().map(|archetype| archetype.rows().map(|row|
            // SAFETY: TODO
            unsafe{ Q::get_row_ref(archetype, row).unwrap_unchecked() }
        )).flatten()
    }
}


impl<'l, Q : ComponentQuery, F : ComponentFilter> IntoIterator for &'l mut Entities<Q, F> {
    type Item     = Q::ItemMut<'l>;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.archetypes.iter_mut().map(|archetype| archetype.rows().map(|row|
            // SAFETY: TODO
            unsafe{ Q::get_row_mut(archetype, row).unwrap_unchecked() }
        )).flatten()
    }
}


impl<Q : ComponentQuery, F : ComponentFilter> IntoIterator for Entities<Q, F> {
    type Item     = EntitiesEntry<Q::AsStatic>;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.archetypes.into_iter().map(|archetype| {
            let archetype = Arc::new(archetype);
            archetype.rows().collect::<Vec<_>>().into_iter().map(move |row| {
                let mut entry = EntitiesEntry {
                    archetype : UnsafeCell::new(Arc::clone(&archetype)),
                    entry     : MaybeUninit::uninit()
                };
                // SAFETY: TODO
                entry.entry.write(unsafe{ Q::AsStatic::get_row_mut(&*entry.archetype.get(), row).unwrap_unchecked() });
                entry
            })
        }).flatten()
    }
}

/// TODO: Doc comments
pub struct EntitiesEntry<Q : ComponentQuery> {

    /// TODO: Doc comments
    archetype : UnsafeCell<Arc<RwLockWriteGuard<Archetype>>>,

    /// TODO: Doc comments
    entry : MaybeUninit<Q::ItemMut<'static>>

}

unsafe impl<Q : ComponentQuery> Sync for EntitiesEntry<Q>
where Q::ItemMut<'static> : Sync
{ }

unsafe impl<Q : ComponentQuery> Send for EntitiesEntry<Q>
where Q::ItemMut<'static> : Send
{ }

impl<Q : ComponentQuery> Deref for EntitiesEntry<Q> {
    type Target = Q::ItemMut<'static>;
    fn deref(&self) -> &Self::Target {
        // SAFETY: TODO
        unsafe{ self.entry.assume_init_ref() }
    }
}

impl<Q : ComponentQuery> DerefMut for EntitiesEntry<Q> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: TODO
        unsafe{ self.entry.assume_init_mut() }
    }
}

impl<Q : ComponentQuery> Drop for EntitiesEntry<Q> {
    fn drop(&mut self) {
        // SAFETY: TODO
        unsafe{ self.entry.assume_init_drop(); }
    }
}
