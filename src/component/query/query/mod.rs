//! `trait`s for querying [`Component`](crate::component::Component)s from [`Archetype`]s.


mod impls;


use crate::component::archetype::Archetype;
use crate::query::{ QueryAcquireResult, QueryValidator };
use core::any::TypeId;


/// A query requesting access to [`Component`](crate::component::Component)s attached to entities.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid component query"
)]
pub unsafe trait ComponentQuery {

    /// The type that this [`ComponentQuery`] returns when iterated immutably.
    type Item<'item>;

    /// The type that this [`ComponentQuery`] returns when iterated mutably.
    type ItemMut<'item>;

    /// Returns `true` if this [`ComponentQuery`] only requests types in the given [`Archetype`].
    fn is_subset_of_archetype(column_types : &[TypeId]) -> bool;

    /// Gets a row in the [`Archetype`] by row.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that:
    /// - this query does not violate the borrow checker rules.
    /// - the given row exists.
    unsafe fn get_row_ref<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::Item<'world>>;

    /// Gets a row in the [`Archetype`] by row.
    ///
    /// Implementors will likely have to use [`Archetype::get_column_ptr`] or [`Archetype::get_column_cells_ptr`].
    ///
    /// # Safety
    /// The caller is responsible for ensuring that:
    /// - this query does not violate the borrow checker rules.
    /// - the given row exists.
    unsafe fn get_row_mut<'world>(archetype : &'world Archetype, row : usize) -> QueryAcquireResult<Self::ItemMut<'world>>;

    /// Traverses the types in this [`ComponentQuery`], joining them to a [`QueryValidator`].
    ///
    /// After the entire [`QueryValidator`] has been constructed, [`QueryValidator::panic_on_violation`] will be called.
    /// The implementation of this method should not call [`QueryValidator::panic_on_violation`].
    ///
    /// See [`QueryValidator::empty`], [`QueryValidator::of_immutable`], [`QueryValidator::of_mutable`], [`QueryValidator::of_owned`], and [`QueryValidator::join`].
    ///
    /// # Safety
    /// The implementation of this method **must** include and join every type that this [`ComponentQuery`] requests, each with the correct access type.
    fn validate() -> QueryValidator;

}

/// A marker that promises a [`ComponentQuery`] will not grant mutable or owned access to any values.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid read-only component query"
)]
pub unsafe trait ReadOnlyComponentQuery : ComponentQuery { }
