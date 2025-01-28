//! `struct`s and `trait`s for filtering [`Entities`](crate::entity::Entities) queries.


use crate::component::Component;
use crate::util::variadic::variadic_no_unit;
use core::any::TypeId;
use core::marker::PhantomData;


/// A [`Component`] filter.
///
/// Can be used in [`Entities`](crate::entity::Entities) queries to narrow down the entities that are selected.
///
/// #### Examples
/// ```rust
/// use axecs::prelude::*;
///
/// #[derive(Component, Debug)]
/// struct Name(String);
///
/// #[derive(Component)]
/// struct Fish;
///
/// #[derive(Component)]
/// struct IsRed;
///
/// fn green_non_red_fish(entities: Entities<&Name, And<(With<Fish>, Without<IsRed>)>>) {
///     for name in &entities {
///         println!("Hello, {:?}!", name);
///     }
/// }
/// ```
pub trait ComponentFilter {

    /// Returns `true` if this filter matches the given [`Archetype`](crate::component::archetype::Archetype).
    ///
    /// The [`TypeId`]s of the [`Component`]s stored by the [`Archetype`](crate::component::archetype::Archetype) are given.
    fn archetype_matches(column_types : &[TypeId]) -> bool;

}


/// A filter that matches entities with a [`Component`] `C`.
///
/// #### Examples
/// ```rust
/// use axecs::prelude::*;
///
/// #[derive(Component, Debug)]
/// struct Name(String);
///
/// #[derive(Component)]
/// struct Person;
///
/// fn greet_people(entities: Entities<&Name, With<Person>>) {
///     for name in &entities {
///         println!("Hello, {:?}!", name);
///     }
/// }
/// ```
pub struct With<C : Component> {
    /// [`PhantomData`] on [`fn(C) -> bool`](prim@fn).
    marker : PhantomData<fn(C) -> bool>
}

impl<C : Component + 'static> ComponentFilter for With<C> {
    fn archetype_matches(column_types : &[TypeId]) -> bool {
        column_types.contains(&TypeId::of::<C>())
    }
}


/// A filter that matches entities without a [`Component`] `C`.
///
/// #### Examples
/// ```rust
/// use axecs::prelude::*;
///
/// #[derive(Component, Debug)]
/// struct Name(String);
///
/// #[derive(Component)]
/// struct Person;
///
/// fn greet_aliens(entities: Entities<&Name, Without<Person>>) {
///     for name in &entities {
///         println!("Hello, {:?}!", name);
///     }
/// }
/// ```
pub struct Without<C : Component> {
    /// [`PhantomData`] on [`fn(C) -> bool`](prim@fn).
    marker : PhantomData<fn(C) -> bool>
}

impl<C : Component + 'static> ComponentFilter for Without<C> {
    fn archetype_matches(column_types : &[TypeId]) -> bool {
        ! column_types.contains(&TypeId::of::<C>())
    }
}


/// A filter that matches all entities.
///
/// This is the default filter on an [`Entities`](crate::entity::Entities) query, but is pretty much useless everywhere else.
pub struct True {
    /// Prevents constructing.
    _private : ()
}

impl ComponentFilter for True {
    fn archetype_matches(_column_types : &[TypeId]) -> bool {
        true
    }
}


/// A filter that matches no entities.
///
/// [`False`] is pretty much useless.
pub struct False {
    /// Prevents constructing.
    _private : ()
}

impl ComponentFilter for False {
    fn archetype_matches(_column_types : &[TypeId]) -> bool {
        false
    }
}


/// A filter that inverts another filter.
///
/// Built-in filters have inverted variants. Prefer those over [`Not`]:
/// - [`With`] / [`Without`]
/// - [`True`] / [`False`]
/// - [`And`]  / [`Nand`]
/// - [`Or`]   / [`Nor`]
/// - [`Xor`]  / [`Xnor`]
pub struct Not<F : ComponentFilter> {
    /// [`PhantomData`] on [`fn(F) -> bool`](prim@fn).
    marker : PhantomData<fn(F) -> bool>
}

impl<F : ComponentFilter> ComponentFilter for Not<F> {
    fn archetype_matches(column_types : &[TypeId]) -> bool {
        ! <F as ComponentFilter>::archetype_matches(column_types)
    }
}


/// A filter that tests if all of the given filters match.
///
/// #### Examples
/// ```rust
/// use axecs::prelude::*;
///
/// #[derive(Component, Debug)]
/// struct Name(String);
///
/// #[derive(Component)]
/// struct Person;
///
/// #[derive(Component)]
/// struct Happy;
///
/// fn greet_happy_people(entities: Entities<&Name, And<(With<Person>, With<Happy>)>>) {
///     for name in &entities {
///         println!("Hello, {:?}!", name);
///     }
/// }
/// ```
pub struct And<F : ComponentFilterGroup> {
    /// [`PhantomData`] on [`fn(F) -> bool`](prim@fn).
    marker : PhantomData<fn(F) -> bool>
}

impl<F : ComponentFilterGroup> ComponentFilter for And<F> {
    fn archetype_matches(column_types : &[TypeId]) -> bool {
        <F as ComponentFilterGroup>::archetype_matches_all(column_types)
    }
}


/// A filter that tests if any of the given filters do not match.
///
/// #### Examples
/// ```rust
/// use axecs::prelude::*;
///
/// #[derive(Component, Debug)]
/// struct Name(String);
///
/// #[derive(Component)]
/// struct Happy;
///
/// #[derive(Component)]
/// struct Smiling;
///
/// fn greet_unhappy_or_unsmiling_people(entities: Entities<&Name, Nand<(With<Happy>, With<Smiling>)>>) {
///     for name in &entities {
///         println!("Hello, {:?}!", name);
///     }
/// }
/// ```
pub struct Nand<F : ComponentFilterGroup> {
    /// [`PhantomData`] on [`fn(F) -> bool`](prim@fn).
    marker : PhantomData<fn(F) -> bool>
}

impl<F : ComponentFilterGroup> ComponentFilter for Nand<F> {
    fn archetype_matches(column_types : &[TypeId]) -> bool {
        ! <F as ComponentFilterGroup>::archetype_matches_all(column_types)
    }
}


/// A filter that tests if any of the given filters match.
///
/// #### Examples
/// ```rust
/// use axecs::prelude::*;
///
/// #[derive(Component, Debug)]
/// struct Name(String);
///
/// #[derive(Component)]
/// struct Square;
///
/// #[derive(Component)]
/// struct Triangle;
///
/// fn greet_sharp_shapes(entities: Entities<&Name, Or<(With<Square>, With<Triangle>)>>) {
///     for name in &entities {
///         println!("Hello, {:?}!", name);
///     }
/// }
/// ```
pub struct Or<F : ComponentFilterGroup> {
    /// [`PhantomData`] on [`fn(F) -> bool`](prim@fn).
    marker : PhantomData<fn(F) -> bool>
}

impl<F : ComponentFilterGroup> ComponentFilter for Or<F> {
    fn archetype_matches(column_types : &[TypeId]) -> bool {
        <F as ComponentFilterGroup>::archetype_matches_any(column_types)
    }
}


/// A filter that tests if all of the given filters do not match.
///
/// #### Examples
/// ```rust
/// use axecs::prelude::*;
///
/// #[derive(Component, Debug)]
/// struct Name(String);
///
/// #[derive(Component)]
/// struct Mercury;
///
/// #[derive(Component)]
/// struct Venus;
///
/// #[derive(Component)]
/// struct Earth;
///
/// #[derive(Component)]
/// struct Mars;
///
/// fn greet_non_inner_planets(entities: Entities<&Name, Nor<(With<Mercury>, With<Venus>, With<Earth>, With<Mars>)>>) {
///     for name in &entities {
///         println!("Hello, {:?}!", name);
///     }
/// }
/// ```
pub struct Nor<F : ComponentFilterGroup> {
    /// [`PhantomData`] on [`fn(F) -> bool`](prim@fn).
    marker : PhantomData<fn(F) -> bool>
}

impl<F : ComponentFilterGroup> ComponentFilter for Nor<F> {
    fn archetype_matches(column_types : &[TypeId]) -> bool {
        ! <F as ComponentFilterGroup>::archetype_matches_any(column_types)
    }
}


/// A filter that tests if exactly one of the given filters match.
///
/// #### Examples
/// ```rust
/// use axecs::prelude::*;
///
/// #[derive(Component, Debug)]
/// struct Name(String);
///
/// #[derive(Component)]
/// struct Happy;
///
/// #[derive(Component)]
/// struct Sad;
///
/// fn greet_happy_xor_sad_people(entities: Entities<&Name, Xor<(With<Happy>, With<Sad>)>>) {
///     for name in &entities {
///         println!("Hello, {:?}!", name);
///     }
/// }
/// ```
pub struct Xor<F : ComponentFilterGroup> {
    /// [`PhantomData`] on [`fn(F) -> bool`](prim@fn).
    marker : PhantomData<fn(F) -> bool>
}

impl<F : ComponentFilterGroup> ComponentFilter for Xor<F> {
    fn archetype_matches(column_types : &[TypeId]) -> bool {
        <F as ComponentFilterGroup>::archetype_matches_one(column_types)
    }
}


/// A filter that tests if not exactly one of the given filters match.
///
/// #### Examples
/// ```rust
/// use axecs::prelude::*;
///
/// #[derive(Component, Debug)]
/// struct Name(String);
///
/// #[derive(Component)]
/// struct Happy;
///
/// #[derive(Component)]
/// struct Sad;
///
/// fn greet_neutral_people(entities: Entities<&Name, Xnor<(With<Happy>, With<Sad>)>>) {
///     for name in &entities {
///         println!("Hello, {:?}!", name);
///     }
/// }
/// ```
pub struct Xnor<F : ComponentFilterGroup> {
    /// [`PhantomData`] on [`fn(F) -> bool`](prim@fn).
    marker : PhantomData<fn(F) -> bool>
}

impl<F : ComponentFilterGroup> ComponentFilter for Xnor<F> {
    fn archetype_matches(column_types : &[TypeId]) -> bool {
        ! <F as ComponentFilterGroup>::archetype_matches_one(column_types)
    }
}


/// A group of [`ComponentFilter`]s.
pub unsafe trait ComponentFilterGroup {

    /// Returns `true` if all of the filters in this group match the given [`Archetype`](crate::component::archetype::Archetype).
    fn archetype_matches_all(column_types : &[TypeId]) -> bool;

    /// Returns `true` if any of the filters in this group match the given [`Archetype`](crate::component::archetype::Archetype).
    fn archetype_matches_any(column_types : &[TypeId]) -> bool;

    /// Returns `true` if exactly one of the filters in this group match the given [`Archetype`](crate::component::archetype::Archetype).
    fn archetype_matches_one(column_types : &[TypeId]) -> bool;

}

unsafe impl<F : ComponentFilter> ComponentFilterGroup for F {
    fn archetype_matches_all(column_types : &[TypeId]) -> bool {
        <F as ComponentFilter>::archetype_matches(column_types)
    }
    fn archetype_matches_any(column_types : &[TypeId]) -> bool {
        <F as ComponentFilter>::archetype_matches(column_types)
    }
    fn archetype_matches_one(column_types : &[TypeId]) -> bool {
        <F as ComponentFilter>::archetype_matches(column_types)
    }
}

variadic_no_unit!{ #[doc(fake_variadic)] impl_component_filter_group_for_tuple }
/// Implements [`ComponentFilterGroup`] for a tuple of [`ComponentFilterGroup`]s.
macro impl_component_filter_group_for_tuple( $( #[$meta:meta] )* $( $generic:ident ),* $(,)? ) {

    $( #[ $meta ] )*
    unsafe impl< $( $generic : ComponentFilterGroup ),* > ComponentFilterGroup for ( $( $generic , )* ) {
        fn archetype_matches_all(column_types : &[TypeId]) -> bool {
            true $( && <$generic as ComponentFilterGroup>::archetype_matches_all(column_types) )*
        }
        fn archetype_matches_any(column_types : &[TypeId]) -> bool {
            false $( || <$generic as ComponentFilterGroup>::archetype_matches_any(column_types) )*
        }
        fn archetype_matches_one(column_types : &[TypeId]) -> bool {
            [ $( <$generic as ComponentFilterGroup>::archetype_matches_one(column_types) , )* ].into_iter().filter(|&c| c).count() == 1
        }
    }

}
