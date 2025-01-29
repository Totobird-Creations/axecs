#![doc = include_str!("../README.md")]


#![feature(
    decl_macro,
    const_type_id,
    const_type_name,
    macro_metavar_expr,
    associated_type_defaults,
    impl_trait_in_assoc_type,
    async_fn_track_caller,
    future_join
)]
#![feature(
    assert_matches,
    rustdoc_internals,
    doc_cfg
)]

#![no_std]
extern crate alloc;


// TODO: pub mod app;
// TODO: pub mod scheduler;

pub mod world;
pub mod resource;

pub mod entity;
pub mod component;

pub mod query;
pub mod system;

pub mod schedule;

pub mod util;


/// Common types for quick and easy setup.
pub mod prelude {

    #[doc(inline)]
    pub use crate::world::World;

    /// Implements [`Resource`](crate::resource::Resource) on an item.
    ///
    /// #### Examples
    /// ```rust
    /// use axecs::prelude::*;
    ///
    /// #[derive(Resource)]
    /// struct ClientConfig {
    ///     sensitivity : f32,
    ///     brightness  : f32,
    ///     language    : String
    /// }
    /// ```
    #[cfg(feature = "derive")]
    #[doc(cfg(feature = "derive"))]
    pub use axecs_macro::Resource;

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

    #[doc(inline)]
    pub use crate::system::{ IntoSystem, In, Local };

    #[doc(inline)]
    pub use crate::schedule::system::{ IntoScheduledSystemConfig, IntoConditionallyScheduledSystemConfig };

}
