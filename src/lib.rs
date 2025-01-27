#![doc = include_str!("../README.md")]


#![feature(
    decl_macro,
    const_type_id,
    const_type_name,
    macro_metavar_expr,
    associated_type_defaults,
    impl_trait_in_assoc_type,
    async_fn_track_caller
)]
#![feature(
    assert_matches,
    rustdoc_internals,
    doc_cfg
)]


pub mod world;
// TODO: pub mod resource;

pub mod archetype;
pub mod entity;
pub mod component;

pub mod query;
// TODO: pub mod system;

pub mod util;


/// Common types for quick and easy setup.
pub mod prelude {

    #[doc(inline)]
    pub use crate::world::World;

    #[doc(inline)]
    pub use crate::entity::{ Entity, Entities };

    /// Implements [`Component`](crate::component::Component) on an item.
    ///
    /// #### Examples
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
    #[cfg(feature = "derive")]
    #[doc(cfg(feature = "derive"))]
    pub use axecs_macro::Component;

    #[doc(inline)]
    pub use crate::component::query::{ With, Without, And, Nand, Or, Nor, Xor, Xnor };

}
