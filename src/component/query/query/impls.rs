//! Implementations of [`ComponentQuery`] and [`ReadOnlyComponentQuery`] on foreign types.


use crate::archetype::Archetype;
use crate::entity::Entity;
use crate::component::{ self, Component };
use crate::component::query::{ ComponentQuery, ReadOnlyComponentQuery };
use crate::query::{ QueryAcquireResult, QueryValidator };
use crate::util::variadic::variadic_no_unit;
use crate::util::multizip::multizip;
use core::any::TypeId;
#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use core::any::type_name;


unsafe impl ComponentQuery for () {
    type Item<'item> = ();
    type ItemMut<'item> = Self::Item<'item>;

    fn is_subset_of_archetype(_column_types : &[TypeId]) -> bool {
        true
    }

    unsafe fn iter_rows_ref<'world>(archetype : &'world Archetype) -> QueryAcquireResult<impl Iterator<Item = Self::Item<'world>>> {
        QueryAcquireResult::Ready(archetype.rows().map(|_| ()))
    }

    unsafe fn iter_rows_mut<'world>(archetype : &'world Archetype) -> QueryAcquireResult<impl Iterator<Item = Self::ItemMut<'world>>> {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ Self::iter_rows_ref(archetype) }
    }

    fn validate() -> QueryValidator {
        QueryValidator::empty()
    }

}
unsafe impl ReadOnlyComponentQuery for () { }


unsafe impl ComponentQuery for Entity {
    type Item<'item> = Entity;
    type ItemMut<'item> = Self::Item<'item>;

    fn is_subset_of_archetype(_column_types : &[TypeId]) -> bool {
        true
    }

    unsafe fn iter_rows_ref<'world>(archetype : &'world Archetype) -> QueryAcquireResult<impl Iterator<Item = Self::Item<'world>>> {
        QueryAcquireResult::Ready(archetype.rows().map(move |row| Entity::new(
            archetype.archetype_id(),
            #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
            archetype.archetype_name(),
            row
        )))
    }

    unsafe fn iter_rows_mut<'world>(archetype : &'world Archetype) -> QueryAcquireResult<impl Iterator<Item = Self::ItemMut<'world>>> {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ Self::iter_rows_ref(archetype) }
    }

    fn validate() -> QueryValidator {
        QueryValidator::empty()
    }

}
unsafe impl ReadOnlyComponentQuery for Entity { }


unsafe impl<C : Component + 'static> ComponentQuery for &C {
    type Item<'item> = &'item C;
    type ItemMut<'item> = Self::Item<'item>;

    fn is_subset_of_archetype(column_types : &[TypeId]) -> bool {
        column_types.contains(&TypeId::of::<C>())
    }

    unsafe fn iter_rows_ref<'world>(archetype : &'world Archetype) -> QueryAcquireResult<impl Iterator<Item = Self::Item<'world>>> {
        if let Some(column) = archetype.get_column_cells_ref::<C>() {
            QueryAcquireResult::Ready(column)
        } else {
            QueryAcquireResult::DoesNotExist {
                #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                name : type_name::<component::marker::Component<C>>()
            }
        }
    }

    unsafe fn iter_rows_mut<'world>(archetype : &'world Archetype) -> QueryAcquireResult<impl Iterator<Item = Self::ItemMut<'world>>> {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ Self::iter_rows_ref(archetype) }
    }

    fn validate() -> QueryValidator {
        QueryValidator::of_immutable::<component::marker::Component<C>>()
    }

}
unsafe impl<C : Component + 'static> ReadOnlyComponentQuery for &C { }


unsafe impl<C : Component + 'static> ComponentQuery for &mut C {
    type Item<'item> = &'item C;
    type ItemMut<'item> = &'item mut C;

    fn is_subset_of_archetype(column_types : &[TypeId]) -> bool {
        column_types.contains(&TypeId::of::<C>())
    }

    unsafe fn iter_rows_ref<'world>(archetype : &'world Archetype) -> QueryAcquireResult<impl Iterator<Item = Self::Item<'world>>> {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ <&C as ComponentQuery>::iter_rows_ref(archetype) }
    }

    unsafe fn iter_rows_mut<'world>(archetype : &'world Archetype) -> QueryAcquireResult<impl Iterator<Item = Self::ItemMut<'world>>> {
        if let Some(column) = archetype.get_column_cells_ptr::<C>() {
            // SAFETY: The caller is responsible for upholding the safety guarantees.
            QueryAcquireResult::Ready(column.map(|cell| unsafe{ &mut*cell }))
        } else {
            QueryAcquireResult::DoesNotExist {
                #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                name : type_name::<component::marker::Component<C>>()
            }
        }
    }

    fn validate() -> QueryValidator {
        QueryValidator::of_mutable::<component::marker::Component<C>>()
    }

}


// TODO: Option<Q>


variadic_no_unit!{ #[doc(fake_variadic)] impl_component_query_for_tuple }
/// Implements [`ComponentQuery`] and [`ReadOnlyComponentQuery`] for a tuples of those types.
macro impl_component_query_for_tuple( $( #[$meta:meta] )* $( $generic:ident ),* $(,)? ) {

    #[allow(non_snake_case)]
    $( #[ $meta ] )*
    unsafe impl< $( $generic : ComponentQuery ),* > ComponentQuery for ( $( $generic , )* ) {
        type Item<'item> = ( $( $generic::Item<'item> , )* );
        type ItemMut<'item> = ( $( $generic::ItemMut<'item> , )* );

        fn is_subset_of_archetype(column_types : &[TypeId]) -> bool {
            true $( && $generic::is_subset_of_archetype(column_types) )*
        }

        unsafe fn iter_rows_ref<'world>(archetype : &'world Archetype) -> QueryAcquireResult<impl Iterator<Item = Self::Item<'world>>> {
            // SAFETY: The caller is responsible for upholding the safety guarantees.
            $( let $generic = match (unsafe{ <$generic as ComponentQuery>::iter_rows_ref(archetype) }) {
                QueryAcquireResult::Ready(out)            => out,
                #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                QueryAcquireResult::DoesNotExist { name } => { return QueryAcquireResult::DoesNotExist { name }; }
                #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
                QueryAcquireResult::DoesNotExist { }      => { return QueryAcquireResult::DoesNotExist { }; }
            }; )*
            QueryAcquireResult::Ready(multizip!( $( $generic , )* ))
        }

        unsafe fn iter_rows_mut<'world>(archetype : &'world Archetype) -> QueryAcquireResult<impl Iterator<Item = Self::ItemMut<'world>>> {
            // SAFETY: The caller is responsible for upholding the safety guarantees.
                // SAFETY: As long as this [`ComponentQuery`] does not violate the archetype rules,
                //         this operation will not access a column that is already mutable accessed
                //         elsewhere, as each column [`Component`] type stored in the [`Archetype`]
                //         is unique.
            $( let $generic = match (unsafe{ <$generic as ComponentQuery>::iter_rows_mut(archetype) }) {
                QueryAcquireResult::Ready(out)            => out,
                #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                QueryAcquireResult::DoesNotExist { name } => { return QueryAcquireResult::DoesNotExist { name }; }
                #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
                QueryAcquireResult::DoesNotExist { }      => { return QueryAcquireResult::DoesNotExist { }; }
            }; )*
            QueryAcquireResult::Ready(multizip!( $( $generic , )* ))
        }

        fn validate() -> QueryValidator {
            let mut qv = QueryValidator::empty();
            $( qv = QueryValidator::join(qv, <$generic as ComponentQuery>::validate()); )*
            qv
        }

    }

    $( #[ $meta ] )*
    unsafe impl< $( $generic : ReadOnlyComponentQuery ),* > ReadOnlyComponentQuery for ( $( $generic , )* ) { }

}
