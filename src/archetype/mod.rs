//! Entities grouped by their components.
//!
//! An archetype is a table where each row is an entity, and each column is a component.
//!
//! Each group of components is a unique archetype.
//! `(Health, Damage, Speed)` and `(Damage, Health, Speed)` are part of the same archetype:
//! | Entity ID | Health | Damage | Speed |
//! |-----------|--------|--------|-------|
//! | 0         | 100.0  | 2.5    | 10.0  |
//! | 1         | 12.0   | 3.0    | 7.5   |
//!
//!  but `(Health, Resistance)` is part of a different archetype.
//! | Entity ID | Health | Resistance |
//! |-----------|--------|------------|
//! | 0         | 72.3   | 50.0       |
//! | 1         | 89.0   | 37.5       |
//! | 2         | 1.5    | 95.0       |
//!
//! #### Usage
//! Typically, a [`World`](crate::world::World) will manage the archetypes for you and provide
//! a safe API, but they can be used directly as well.
//!
//! ```rust
//! use axecs::prelude::*;
//! use axecs::archetype::Archetype;
//! use core::any::type_name;
//!
//! #[derive(Component)]
//! struct ComponentOne {
//!     value : usize
//! }
//!
//! #[derive(Component)]
//! struct ComponentTwo {
//!     value : usize
//! }
//!
//! type Bundle = ( ComponentOne, ComponentTwo, );
//!
//! let mut archetype = Archetype::new::<Bundle>(0, type_name::<Bundle>());
//! //                                             |^^^^^^^^^^^^^^^^^^^^^
//! //                                             | This argument only exists in debug mode,
//! //                                             | or when the `keep_debug_names` feature is
//! //                                             | enabled.
//!
//! //                                              | This bundle must contain the exact
//! //                                              | component types that this archetype
//! //                                              | stores. No more, no less.
//! //                                              |vvvvvv
//! let entity_row = unsafe{ archetype.spawn_unchecked::<Bundle>((
//!     ComponentOne { value : 123 },
//!     ComponentTwo { value : 456 }
//! )) };
//!
//! for (one, two,) in unsafe{ archetype.query_unchecked_mut::<
//!     (&mut ComponentOne, &mut ComponentTwo,)
//! // |^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! // | This bundle must be a subset of the component types that this archetype stores. No
//! // | more, no less.
//! >() } {
//!
//! }
//!
//! if (archetype.has_row(entity_row)) {
//!     //                                   | This bundle must contain the exact component
//!     //                                   | types that this archetype stores. No more, no
//!     //                                   | less.
//!     //                                   |vvvvvv
//!     unsafe{ archetype.despawn_unchecked::<Bundle>(entity_row) };
//! // |^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! // | This operation is undefined behaviour if no entity exists at `entity_row`.
//! }
//! ```


mod storage;
pub use storage::*;

mod column;
pub use column::*;


use crate::component::{ Component, ComponentBundle };
use crate::component::query::{ ComponentQuery, ReadOnlyComponentQuery };
#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use crate::util::unqualified::UnqualifiedTypeName;
use core::any::TypeId;
use core::fmt;
use core::cell::UnsafeCell;
use alloc::boxed::Box;
use alloc::vec::Vec;


/// A single table of entities, all with the same componenets.
pub struct Archetype {

    /// The ID of this archetype.
    archetype_id    : usize,

    /// The name of this archetype.
    ///
    /// This is usually the [`type_name`](::core::any::type_name) of a tuple containing the components.
    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
    #[doc(cfg(feature = "keep_debug_names"))]
    archetype_name  : &'static str,

    /// The columns of this [`Archetype`], each storing one [`Component`] type.
    columns         : Box<[UnsafeCell<ArchetypeColumn>]>,

    /// The number of allocated rows.
    rows_dense_next : usize,

    /// Rows that are allocated, but unoccupied.
    /// A newly spawned entity can occupy these rows instead of allocating more memory.
    unoccupied_rows : Vec<usize>

}

impl Archetype {

    /// Returns the ID of this archetype.
    pub fn archetype_id(&self) -> usize {
        self.archetype_id
    }

    /// Returns the name of this archetype.
    ///
    /// This is usually the [`type_name`](::core::any::type_name) of a tuple containing the components.
    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
    #[doc(cfg(feature = "keep_debug_names"))]
    pub fn archetype_name(&self) -> &'static str {
        self.archetype_name
    }

}

impl Archetype {

    /// Creates a new archetype from a [`ComponentBundle`].
    ///
    /// The archetype will contain no rows to start.
    #[doc(cfg(feature = "keep_debug_names"))]
    pub fn new<C : ComponentBundle>(
        archetype_id   : usize,
        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
        archetype_name : &'static str
    ) -> Self { Self {
        archetype_id,
        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
        archetype_name,
        // SAFETY: When this archetype is dropped, `drop_dealloc_except` is called on all columns, passing
        //         in `self.free_rows` as the rows to skip dropping.
        columns         : C::type_info().into_iter().map(|cti| UnsafeCell::new(unsafe{ ArchetypeColumn::new(cti) })).collect::<Box<[_]>>(),
        rows_dense_next : 0,
        unoccupied_rows : Vec::new(),
    } }

    /// Creates a new archetype from a [`ComponentBundle`].
    ///
    /// The archetype will contain no rows to start.
    #[cfg(doc)]
    #[doc(cfg(not(feature = "keep_debug_names")))]
    pub fn new<C : ComponentBundle>() -> Self {
        core::hint::unreachable_unchecked()
    }

    /// Returns an [`Iterator`] to a column from this archetype.
    ///
    /// If this archetype does not contain a column of type `C`, `None` is returned.
    pub fn get_column_ref<C : Component + 'static>(&self) -> Option<&ArchetypeColumn> {
        let type_id = TypeId::of::<C>();
        self.columns.iter().find_map(|column| {
            // SAFETY: `self` is borrowed immutably, preventing it from being accessed mutably throughout
            //         the lifetime of the returned value.
            let column = unsafe{ &*column.get() };
            (column.type_id() == type_id).then_some(column)
        })
    }

    /// Returns an [`Iterator`] over the cells in a column from this archetype.
    ///
    /// If this archetype does not contain a column of type `C`, `None` is returned.
    pub fn get_column_cells_ref<'l, C : Component + 'static>(&'l self) -> Option<impl Iterator<Item = &'l C>> {
        let column = self.get_column_ref::<C>()?;
        // SAFETY: `self` is borrowed immutably, preventing it from being accessed mutably throughout
        //         the lifetime of the returned value.
        Some(self.rows().map(|row| unsafe{ column.get_ref(row) }))
    }

    /// Returns a mutable reference to a column from this archetype.
    ///
    /// If this archetype does not contain a column of type `C`, `None` is returned.
    pub fn get_column_mut<C : Component + 'static>(&mut self) -> Option<&mut ArchetypeColumn> {
        let type_id = TypeId::of::<C>();
        self.columns.iter_mut().find_map(|column| {
            let column = column.get_mut();
            (column.type_id() == type_id).then_some(column)
        })
    }

    /// Returns an [`Iterator`] over the cells in a column from this archetype.
    ///
    /// If this archetype does not contain a column of type `C`, `None` is returned.
    pub fn get_column_cells_mut<'l, C : Component + 'static>(&'l mut self) -> Option<impl Iterator<Item = &'l mut C>> {
        let column = self.get_column_ref::<C>()?;
        // SAFETY: `self` is borrowed mutably, preventing it from being accessed throughout
        //         the lifetime of the returned value.
        Some(self.rows().map(|row| unsafe{ &mut*column.get_ptr(row) }))
    }

    /// Returns a pointer to a column from this archetype.
    ///
    /// If this archetype does not contain a column of type `C`, `None` is returned.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that:
    /// - the pointer is not used after this archetype is dropped.
    /// - data-races are prevented.
    pub fn get_column_ptr<C : Component + 'static>(&self) -> Option<*mut ArchetypeColumn> {
        let type_id = TypeId::of::<C>();
        self.columns.iter().find_map(|column| {
            let column = column.get();
            // SAFETY: The caller is responsible for upholding the safety guarantees.
            (unsafe{ &*column }.type_id() == type_id).then_some(column)
        })
    }

    /// Returns an [`Iterator`] over the cells in a column from this archetype.
    ///
    /// If this archetype does not contain a column of type `C`, `None` is returned.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that:
    /// - the pointers are not used when their corresponding cells are unoccupied.
    /// - data-races are prevented.
    pub fn get_column_cells_ptr<C : Component + 'static>(&self) -> Option<impl Iterator<Item = *mut C>> {
        let column = self.get_column_ref::<C>()?;
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        Some(self.rows().map(|row| unsafe{ column.get_ptr(row) }))
    }

    /// Returns `true` if the archetype has a given `row` populated.
    pub fn has_row(&self, row : usize) -> bool {
        (row < self.rows_dense_next) && (! self.unoccupied_rows.contains(&row))
    }

    /// Returns an [`Iterator`] over the populated rows in this archetype.
    pub fn rows(&self) -> impl Iterator<Item = usize> {
        (0..self.rows_dense_next).filter(|row| ! self.unoccupied_rows.contains(row))
    }

    /// Adds a row to this archetype, "spawning" an entity.
    ///
    /// If there is a row that was previously unoccupied through [`Archetype::despawn_unchecked`] or similar, the memory of that row will be used instead.
    ///
    /// # Returns
    /// Returns the row index of the entity that was spawned.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given bundle:
    /// - contains the exact [`Component`]s in this archetype. No more, no less.
    /// - does not violate the archetype rules. See [`BundleValidator`](crate::component::BundleValidator).
    pub unsafe fn spawn_unchecked<C : ComponentBundle>(&mut self, bundle : C) -> usize {
        let row = if let Some(row) = self.unoccupied_rows.pop() {
            // SAFETY: `row` is in `free_rows`, meaning it was previously dropped and is safe to overwrite.
            //         The row was unoccupied by the `pop` in the line above, preventing it from being overwritten again.
            unsafe{ bundle.write_into(self, row); }
            row
        } else {
            let row = self.rows_dense_next;
            let (a, b) = self.rows_dense_next.overflowing_add(1);
            if (b) { panic!("attempt to add with overflow") }
            self.rows_dense_next = a;
            // SAFETY: The caller is responsible for ensuring that the bundle contains the exact [`Component`]s in
            //         this archetype. No more, no less.
            unsafe{ bundle.push_into(self); }
            row
        };
        row
    }

    /// Removes a row from this archetype, "despawning" and entity.
    ///
    /// In memory, the row is not actually modified. Its destructor is run and the row is marked as unoccupied.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that:
    /// - the given bundle type `C` contains the exact [`Component`]s in this archetype. No more, no less.
    /// - the given bundle type `C` does not violate the archetype rules. See [`BundleValidator`](crate::component::BundleValidator).
    /// - the given `row` is currently occupied
    pub unsafe fn despawn_unchecked<C : ComponentBundle>(&mut self, row : usize) {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ C::drop_in(self, row) }
        self.unoccupied_rows.push(row);
    }

    /// Returns an [`Iterator`] over the requested columns in this archetype, without checking if the given [`ReadOnlyComponentQuery`] is valid.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ReadOnlyComponentQuery`]:
    /// - does not violate the archetype rules. See [`BundleValidator`](crate::component::BundleValidator).
    /// - is a **subset** of the components stored in this archetype.
    pub unsafe fn query_unchecked<Q : ReadOnlyComponentQuery>(&self) -> impl Iterator<Item = Q::Item<'_>> {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ Q::iter_rows_ref(self).unwrap() }
    }

    /// Returns an [`Iterator`] over the requested columns in this archetype, without checking if the given [`ComponentQuery`] is valid.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the given [`ComponentQuery`]:
    /// - does not violate the archetype rules. See [`BundleValidator`](crate::component::BundleValidator).
    /// - is a **subset** of the components stored in this archetype.
    pub unsafe fn query_unchecked_mut<Q : ComponentQuery>(&mut self) -> impl Iterator<Item = Q::ItemMut<'_>> {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        // SAFETY: `self` is borrowed mutably, preventing it from being accessed throughout
        //         the lifetime of the returned value.
        unsafe{ Q::iter_rows_mut(self).unwrap() }
    }

}

impl fmt::Debug for Archetype {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Archetype<(")?;
        for (i, column) in self.columns.iter().enumerate() {
            // SAFETY: `self` is borrowed immutably, preventing it from being accessed mutably.
            let column = unsafe{ &*column.get() };
            if (i > 0) { write!(f, " ")?; }
            #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
            // SAFETY: `column.type_name()` returns a value previously generated using `core::any::type_name`.
            write!(f, "{},", unsafe{ UnqualifiedTypeName::from_unchecked(column.type_name()) })?;
            #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
            write!(f, "w{}a{},", column.type_layout().size(), column.type_layout().align())?;
        }
        write!(f, ")>[_; {}]", self.rows_dense_next - self.unoccupied_rows.len())?;
        Ok(())
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        for column in &mut self.columns {
            // SAFETY: `self.free_rows` contains the indices of the rows that were previously dropped.
            //         Passing them here prevents them from being dropped again.
            unsafe{ column.get_mut().drop_dealloc_except(&self.unoccupied_rows); }
        }
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
    use core::any::type_name;

    struct ComponentOne {
        value : usize
    }
    impl Component for ComponentOne { }

    struct ComponentTwo {
        value : usize
    }
    impl Component for ComponentTwo { }

    struct ComponentThree {
        _value : usize
    }
    impl Component for ComponentThree { }

    type Bundle = ( ComponentOne, ComponentTwo, );

    #[test]
    fn miri_archetype_column_mut_iter() {
        // Create a new archetype.
        let mut archetype = Archetype::new::<Bundle>(
            0,
            #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
            type_name::<Bundle>()
        );

        // Spawn entities.
        let entity0_row = unsafe{ archetype.spawn_unchecked::<Bundle>((
            ComponentOne { value : 123 },
            ComponentTwo { value : 456 }
        )) };
        assert_eq!(entity0_row, 0);

        let entity1_row = unsafe{ archetype.spawn_unchecked::<Bundle>((
            ComponentOne { value : 789 },
            ComponentTwo { value : 101112 }
        )) };
        assert_eq!(entity1_row, 1);

        // Despawn entity.
        unsafe{ archetype.despawn_unchecked::<Bundle>(entity0_row) };

        // Reuse open rows.
        let entity2_row = unsafe{ archetype.spawn_unchecked::<Bundle>((
            ComponentOne { value : 131415 },
            ComponentTwo { value : 161718 }
        )) };
        assert_eq!(entity2_row, 0);

        // Columns don't exist.
        let Some(_) = archetype.get_column_cells_mut::<ComponentOne>() else { panic!("Column for ComponentOne should exist, but it does not.") };
        let Some(_) = archetype.get_column_cells_mut::<ComponentTwo>() else { panic!("Column for ComponentTwo should exist, but it does not.") };
        let None = archetype.get_column_cells_mut::<ComponentThree>() else { panic!("Column for ComponentThree should not exist, but it does.") };

        // Mutable query sanity check
        for (i, (one, two,)) in unsafe{ archetype.query_unchecked_mut::<(&mut ComponentOne, &mut ComponentTwo,)>() }.enumerate() {
            match (i) {
                /* entity1 */ 1 => { assert_eq!(one.value, 789); assert_eq!(two.value, 101112); },
                /* entity2 */ 0 => { assert_eq!(one.value, 131415); assert_eq!(two.value, 161718); },
                _ => unreachable!()
            }
        }

        // Drop all occupied rows.
        drop(archetype);
    }

}
