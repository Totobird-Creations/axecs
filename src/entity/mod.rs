//! Entities are 'things' which [`Component`](crate::component::Component)s can be attached to.
//! They live in [`World`](crate::world::World)s.
//!
//! An entity can be spawned using [`World::spawn`](crate::world::World::spawn) or similar.
//! ```rust
//! use axecs::prelude::*;
//! # use async_std::main;
//!
//! # #[derive(Component)]
//! # struct Position { x : f32, y : f32, z : f32 }
//!
//! # #[derive(Component)]
//! # struct Velocity { x : f32, y : f32, z : f32 }
//!
//! #[main]
//! async fn main() {
//!
//!     let world = World::new();
//!
//!     let entity_0 = world.spawn(()).await; // Spawns an entity with no components.
//!
//!     let entity_1 = world.spawn((
//!         Position { x : 0.0, y : 1.0, z : 2.0 },
//!         Velocity { x : 2.5, y : -0.23, z : 10.0 }
//!     )).await; // Spawns an entity with the `Position` and `Velocity` components.
//!
//!     world.despawn(entity_0);
//!
//! }
//! ```


mod query;
pub use query::*;


#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use crate::util::unqualified::UnqualifiedTypeName;
use core::fmt;


/// Lightweight identifier of an entity.
#[derive(Clone, Copy)]
pub struct Entity {

    /// The ID of the [`Archetype`](crate::component::archetype::Archetype) that this [`Entity`] belongs to.
    archetype_id   : usize,

    /// The name of the [`Archetype`](crate::component::archetype::Archetype) that this [`Entity`] belongs to.
    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
    #[doc(cfg(feature = "keep_debug_names"))]
    archetype_name : &'static str,

    /// The index of the [`Archetype`](crate::component::archetype::Archetype) row that this [`Entity`] is in.
    archetype_row  : usize,

    // TODO: Generational indices, for handling entities being despawned.

}

impl Entity {

    /// Creates a new [`Entity`] identifier.
    #[doc(cfg(feature = "keep_debug_names"))]
    pub(crate) fn new(
        archetype_id   : usize,
        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
        archetype_name : &'static str,
        archetype_row  : usize
    ) -> Self { Self {
        archetype_id,
        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
        archetype_name,
        archetype_row
    } }

    /// Creates a new [`Entity`] identifier.
    #[cfg(doc)]
    #[doc(cfg(not(feature = "keep_debug_names")))]
    pub(crate) fn new(archetype_id : usize, archetype_row : usize) -> Self {
        core::hint::unreachable_unchecked()
    }

    /// Returns the ID of the [`Archetype`](crate::component::archetype::Archetype) that this [`Entity`] belongs to.
    pub fn archetype_id(&self) -> usize {
        self.archetype_id
    }

    /// Returns the name of the [`Archetype`](crate::component::archetype::Archetype) that this [`Entity`] belongs to.
    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
    #[doc(cfg(feature = "keep_debug_names"))]
    pub fn archetype_name(&self) -> &'static str {
        self.archetype_name
    }

    /// Returns the index of the [`Archetype`](crate::component::archetype::Archetype) row that this [`Entity`] is in.
    pub fn archetype_row(&self) -> usize {
        self.archetype_row
    }

}

impl fmt::Debug for Entity {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity(")?;
        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
        // SAFETY: `self.archetype_name` is a value previously generated using `core::any::type_name`.
        write!(f, "a<{}>", unsafe{ UnqualifiedTypeName::from_unchecked(self.archetype_name) })?;
        #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
        write!(f, "a{}", self.archetype_id)?;
        write!(f, ":r{})", self.archetype_row)?;
        Ok(())
    }
}
