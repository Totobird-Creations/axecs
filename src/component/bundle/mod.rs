//! Bundles of [`Component`]s.


mod validate;
pub use validate::*;


use crate::archetype::Archetype;
use crate::component::{ self, Component, ComponentTypeInfo };
use crate::util::variadic::variadic_no_unit;


/// A bundle of [`Component`]s.
///
/// A bundle must not contain multiple of the same [`Component`] type.
/// Since this is unenforcable at compile-time, it is checked at run-time through [`ComponentBundle::validate`].
///
/// Generally, the methods on this trait **should not be called manually**. Instead use methods like [`World::spawn`](crate::world::World::spawn).
///
/// #### Safety
/// If this trait is not implemented properly, it may return bad data which causes [*undefined behaviour*](reference@behavior-considered-undefined).
/// Check the safety information for each method.
pub unsafe trait ComponentBundle {

    /// Returns a [`ComponentTypeInfo`] for each [`Component`] type in this bundle.
    ///
    /// # Safety
    /// The returned [`Vec`] must be sorted by alignment, then [`TypeId`](::core::any::TypeId).
    /// [`ComponentTypeInfo`] implements [`Ord`] and can be properly sorted using [`[ComponentTypeInfo]::sort_unstable`](prim@slice#method.sort_unstable).
    fn type_info() -> Vec<ComponentTypeInfo>;

    /// Pushes this bundle into an [`Archetype`] as a new row.
    ///
    /// See [`ArchetypeColumn::push`](crate::archetype::ArchetypeColumn::push).
    ///
    /// # Safety
    /// The implementation of this method **must not** be no-op.
    /// The caller is responsible for ensuring that the given [`Archetype`] contains the exact [`Component`]s in this bundle. No more, no less.
    unsafe fn push_into(self, archetype : &mut Archetype);

    /// Overwrites a row in an [`Archetype`] with this bundle.
    ///
    /// See [`ArchetypeColumn::write`](crate::archetype::ArchetypeColumn::write).
    ///
    /// # Safety
    /// The implementation of this method **must not** be no-op.
    /// The caller is responsible for ensuring that:
    /// - the given [`Archetype`] contains the exact [`Component`]s in this bundle. No more, no less.
    /// - the given `row` in the [`Archetype`] is not currently occupied.
    unsafe fn write_into(self, archetype : &mut Archetype, row : usize);

    /// Drops a row in an [`Archetype`].
    ///
    /// See [`ArchetypeColumn::drop`](crate::archetype::ArchetypeColumn::drop).
    ///
    /// # Safety
    /// The implementation of this method **must not** be no-op.
    /// The caller is responsible for ensuring that:
    /// - the given [`Archetype`] contains the exact [`Component`]s in this bundle. No more, no less.
    /// - the given `row` in the [`Archetype`] is currently occupied.
    /// - the given `row` in the [`Archetype`] is not dropped again until it is repopulated.
    unsafe fn drop_in(archetype : &mut Archetype, row : usize);

    /// Traverses the types in this bundle, joining them to a [`BundleValidator`].
    ///
    /// After the entire [`BundleValidator`] has been constructed, [`BundleValidator::panic_on_violation`] will be called.
    /// The implementation of this method should not call [`BundleValidator::panic_on_violation`].
    ///
    /// See [`BundleValidator::empty`], [`BundleValidator::of_included`], and [`BundleValidator::join`].
    ///
    /// # Safety
    /// The implementation of this method **must** include and join every type that this [`ComponentBundle`] contains.
    fn validate() -> BundleValidator;

}


unsafe impl<C : Component + 'static> ComponentBundle for C {

    fn type_info() -> Vec<ComponentTypeInfo> {
        vec![ ComponentTypeInfo::of::<C>() ]
    }

    unsafe fn push_into(self, archetype : &mut Archetype) {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ archetype.get_column_mut::<Self>().unwrap_unchecked().push::<Self>(self); }
    }

    unsafe fn write_into(self, archetype : &mut Archetype, row : usize) {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ archetype.get_column_mut::<Self>().unwrap_unchecked().write::<Self>(row, self); }
    }

    unsafe fn drop_in(archetype : &mut Archetype, row : usize) {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ archetype.get_column_mut::<Self>().unwrap_unchecked().drop::<Self>(row); }
    }

    fn validate() -> BundleValidator {
        BundleValidator::of_included::<component::marker::Component<C>>()
    }

}


/// This is an exception to the "must not be no-op" guarantee.
///
/// The [`ComponentQuery`](crate::component::query::ComponentQuery) implementation for [`()`](prim@unit) does not access its column in [`Archetype`].
unsafe impl ComponentBundle for () {

    #[inline]
    fn type_info() -> Vec<ComponentTypeInfo> {
        Vec::new()
    }

    unsafe fn push_into(self, _archetype : &mut Archetype) { }

    unsafe fn write_into(self, _archetype : &mut Archetype, _row : usize) { }

    unsafe fn drop_in(_archetype : &mut Archetype, _row : usize) { }

    fn validate() -> BundleValidator {
        BundleValidator::empty()
    }

}


variadic_no_unit!{ impl_component_bundle_for_tuple }
/// Implements [`ComponentBundle`] for a tuple of [`ComponentBundle`]s.
macro impl_component_bundle_for_tuple( $( #[$meta:meta] )* $( $generic:ident ),* $(,)? ) {

    #[allow(non_snake_case)]
    $( #[ $meta ] )*
    unsafe impl< $( $generic : ComponentBundle ),* > ComponentBundle for ( $( $generic , )* ) {

        fn type_info() -> Vec<ComponentTypeInfo> {
            $( let mut $generic = <$generic as ComponentBundle>::type_info(); )*
            let mut out = Vec::with_capacity( 0 $( + $generic.len() )* );
            $( out.append( &mut $generic ); )*
            out.sort_unstable();
            out
        }

        unsafe fn push_into(self, archetype : &mut Archetype) {
            // SAFETY: The caller is responsible for upholding the safety guarantees.
            $( unsafe{ <$generic as ComponentBundle>::push_into(self.${index()}, archetype); } )*
        }

        unsafe fn write_into(self, archetype : &mut Archetype, row : usize) {
            // SAFETY: The caller is responsible for upholding the safety guarantees.
            $( unsafe{ <$generic as ComponentBundle>::write_into(self.${index()}, archetype, row); } )*
        }

        unsafe fn drop_in(archetype : &mut Archetype, row : usize) {
            // SAFETY: The caller is responsible for upholding the safety guarantees.
            $( unsafe{ <$generic as ComponentBundle>::drop_in(archetype, row); } )*
        }

        fn validate() -> BundleValidator {
            let mut qv = BundleValidator::empty();
            $( qv = BundleValidator::join(qv, <$generic as ComponentBundle>::validate()); )*
            qv
        }

    }

}
