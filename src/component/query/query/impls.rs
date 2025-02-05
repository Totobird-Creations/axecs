//! Implementations of [`ComponentQuery`] and [`ReadOnlyComponentQuery`] on foreign types.


use crate::entity::Entity;
use crate::component::{ self, Component };
use crate::component::query::{ ComponentQuery, ReadOnlyComponentQuery };
use crate::component::archetype::Archetype;
use crate::query::{ QueryAcquireResult, QueryValidator };
use crate::util::variadic::variadic_no_unit;
use core::any::TypeId;
#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use core::any::type_name;


unsafe impl ComponentQuery for () {
    type Item<'item> = ();
    type ItemMut<'item> = Self::Item<'item>;

    fn is_subset_of_archetype(_column_types : &[TypeId]) -> bool {
        true
    }

    unsafe fn get_row_ref<'world>(_archetype : &'world Archetype, _row : usize) -> QueryAcquireResult<Self::Item<'world>> {
        QueryAcquireResult::Ready(())
    }

    unsafe fn get_row_mut<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::ItemMut<'world>> {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ Self::get_row_ref(archetype, row) }
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

    unsafe fn get_row_ref<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::Item<'world>> {
        QueryAcquireResult::Ready(Entity::new(
            archetype.archetype_id(),
            #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
            archetype.archetype_name(),
            row
        ))
    }

    unsafe fn get_row_mut<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::ItemMut<'world>> {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ Self::get_row_ref(archetype, row) }
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

    unsafe fn get_row_ref<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::Item<'world>> {
        if let Some(column) = archetype.get_column_ref::<C>() {
            // SAFETY: TODO
            QueryAcquireResult::Ready(unsafe{ column.get_ref(row) })
        } else {
            QueryAcquireResult::DoesNotExist {
                #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                name : type_name::<component::marker::Component<C>>()
            }
        }
    }

    unsafe fn get_row_mut<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::ItemMut<'world>> {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ Self::get_row_ref(archetype, row) }
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

    unsafe fn get_row_ref<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::Item<'world>> {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ <&C as ComponentQuery>::get_row_ref(archetype, row) }
    }

    unsafe fn get_row_mut<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::ItemMut<'world>> {
        if let Some(column) = archetype.get_column_ptr::<C>() {
            // SAFETY: TODO
            QueryAcquireResult::Ready(unsafe{ (&mut*column).get_mut(row) })
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


unsafe impl<Q : ComponentQuery> ComponentQuery for Option<Q> {
    type Item<'item> = Option<Q::Item<'item>>;
    type ItemMut<'item> = Option<Q::ItemMut<'item>>;

    fn is_subset_of_archetype(_column_types : &[TypeId]) -> bool {
        true
    }

    unsafe fn get_row_ref<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::Item<'world>> {
        // SAFETY: TODO
        match (unsafe{ <Q as ComponentQuery>::get_row_ref(archetype, row) }) {
            QueryAcquireResult::Ready(out)          => QueryAcquireResult::Ready(Some(out)),
            QueryAcquireResult::DoesNotExist { .. } => QueryAcquireResult::Ready(None)
        }
    }

    unsafe fn get_row_mut<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::ItemMut<'world>> {
        // SAFETY: TODO
        match (unsafe{ <Q as ComponentQuery>::get_row_mut(archetype, row) }) {
            QueryAcquireResult::Ready(out)          => QueryAcquireResult::Ready(Some(out)),
            QueryAcquireResult::DoesNotExist { .. } => QueryAcquireResult::Ready(None)
        }
    }

    fn validate() -> QueryValidator {
        <Q as ComponentQuery>::validate()
    }
}


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

        unsafe fn get_row_ref<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::Item<'world>> {
            // SAFETY: The caller is responsible for upholding the safety guarantees.
            $( let $generic = match (unsafe{ <$generic as ComponentQuery>::get_row_ref(archetype, row) }) {
                QueryAcquireResult::Ready(out)            => out,
                #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                QueryAcquireResult::DoesNotExist { name } => { return QueryAcquireResult::DoesNotExist { name }; }
                #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
                QueryAcquireResult::DoesNotExist { }      => { return QueryAcquireResult::DoesNotExist { }; }
            }; )*
            QueryAcquireResult::Ready(( $( $generic , )* ))
        }

        unsafe fn get_row_mut<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::ItemMut<'world>> {
            // SAFETY: The caller is responsible for upholding the safety guarantees.
            // SAFETY: As long as this [`ComponentQuery`] does not violate the archetype rules,
            //         this operation will not access a column that is already mutable accessed
            //         elsewhere, as each column [`Component`] type stored in the [`Archetype`]
            //         is unique.
            $( let $generic = match (unsafe{ <$generic as ComponentQuery>::get_row_mut(archetype, row) }) {
                QueryAcquireResult::Ready(out)            => out,
                #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
                QueryAcquireResult::DoesNotExist { name } => { return QueryAcquireResult::DoesNotExist { name }; }
                #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
                QueryAcquireResult::DoesNotExist { }      => { return QueryAcquireResult::DoesNotExist { }; }
            }; )*
            QueryAcquireResult::Ready(( $( $generic , )* ))
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
