//! [`Component`]s of data which can be attached to entities.
//!
//! A component is a piece of data that can be attached to an entity.
//! They are normal Rust structs.
//! ```rust
//! use axecs::prelude::*;
//!
//! #[derive(Component)]
//! struct Position {x : f32, y : f32, z : f32 }
//! ```
//!
//! An entity may have any number of components attached, but only
//! one of each type.
//! ```rust should_panic
//! use axecs::prelude::*;
//! # use async_std::main;
//!
//! #[derive(Component)]
//! struct Position { x : f32, y : f32, z : f32 }
//!
//! #[main]
//! async fn main() {
//!     let world = World::new();
//!
//!     // Bundle violates the archetype rules:
//!     //   Already included Component<Position>
//!     world.spawn((
//!         Position { x : 0.0, y : 0.0, z : 0.0 },
//!         Position { x : 1.0, y : 2.0, z : 3.0 }
//!     )).await;
//! }
//! ```


mod bundle;
pub use bundle::*;

pub mod query;


use core::any::TypeId;
#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use core::any::type_name;
use core::alloc::Layout;
use core::ptr::NonNull;
use core::hash::{ Hash, Hasher };


/// A component which can be attached to an entity.
///
///
/// #### Examples
///
/// [`axecs::prelude`](crate::prelude) provides a derive macro which can be applied to items.
/// ```rust
/// use axecs::prelude::*;
///
/// #[derive(Component)]
/// struct Health {
///     current : f32,
///     maximum : f32
/// }
///
/// #[derive(Component)]
/// enum MovementState {
///     Idle,
///     Walk,
///     Jump
/// }
/// ```
pub trait Component { }


/// Information about a [`Component`] type, such as [`TypeId`], [`Layout`], and drop function.
#[derive(Clone, Copy, Debug)]
pub struct ComponentTypeInfo {
    type_id : TypeId,
    layout  : Layout,
    drop    : unsafe fn(NonNull<u8>) -> (),
    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
    name    : &'static str
}

impl ComponentTypeInfo {

    /// Returns the [`ComponentTypeInfo`] for a [`Component`] `C`.
    pub const fn of<C : Component + 'static>() -> Self { Self {
        type_id : TypeId::of::<C>(),
        layout  : Layout::new::<C>(),
        // SAFETY: The value pointed to by `ptr` is of type `C`. It is safe to assume
        //         that value is of type `C`.
        drop    : |ptr| unsafe{ ptr.cast::<C>().drop_in_place() },
        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
        name    : type_name::<C>()
    } }

    /// Returns the [`TypeId`] of the [`Component`].
    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the [`Layout`] of the [`Component`].
    pub const fn layout(&self) -> Layout {
        self.layout
    }

    /// Returns the drop function of the [`Component`].
    ///
    /// The function will take a [`NonNull`] pointer to a value of the type this [`ComponentTypeInfo`] corresponds to.
    /// Passing a pointer to a value of a different type is [*undefined behaviour*](reference@behavior-considered-undefined).
    pub const fn drop(&self) -> unsafe fn(NonNull<u8>) -> () {
        self.drop
    }

    /// Returns the [`type_name`] of the [`Component`].
    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
    #[doc(cfg(feature = "keep_debug_names"))]
    pub const fn name(&self) -> &'static str {
        self.name
    }

}

impl PartialEq for ComponentTypeInfo {
    #[inline]
    fn eq(&self, other : &Self) -> bool {
        self.type_id == other.type_id
    }
}
impl Eq for ComponentTypeInfo { }
impl PartialOrd for ComponentTypeInfo {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}
impl Ord for ComponentTypeInfo {
    fn cmp(&self, other : &Self) -> std::cmp::Ordering {
        self.layout.align().cmp(&other.layout.align())
            .reverse()
            .then_with(|| self.type_id.cmp(&other.type_id))
    }
}
impl Hash for ComponentTypeInfo {
    fn hash<H : Hasher>(&self, state : &mut H) {
        self.type_id.hash(state);
    }
}


/// [`Component`] wrapping marker.
pub(crate) mod marker {
    use core::marker::PhantomData;
    /// Used in error messages and [`TypeId`](::core::any::TypeId) comparisons to indicate that a type is a [`Component`](super::Component).
    pub(super) struct Component<C : super::Component> {
        /// [`PhantomData`] on `C`.
        marker : PhantomData<C>
    }
}
