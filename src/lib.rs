#![doc = include_str!("../README.md")]


#![feature(
    decl_macro,
    const_type_id,
    const_type_name,
    macro_metavar_expr,
    associated_type_defaults,
    impl_trait_in_assoc_type,
    async_fn_track_caller,
    future_join,
    map_try_insert
)]
#![feature(
    assert_matches,
    rustdoc_internals,
    doc_cfg
)]

#![no_std]
extern crate alloc;


pub mod app;

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
    pub use crate::app::{ App, AppExit };
    #[doc(inline)]
    pub use crate::app::plugin::Plugin;
    #[doc(inline)]
    pub use crate::app::plugin::CycleSchedulerPlugin;

    #[doc(inline)]
    pub use crate::world::{ World, Commands };

    #[doc(inline)]
    pub use crate::resource::Res;

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

    /// Implements [`ComponentBundle`](crate::component::bundle::ComponentBundle) on an item.
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
    /// struct AttackDamage {
    ///     amount : f32
    /// }
    ///
    /// #[derive(Bundle)]
    /// struct EnemyMob {
    ///     health : Health,
    ///     damage : AttackDamage
    /// }
    /// ```
    #[cfg(feature = "derive")]
    #[doc(cfg(feature = "derive"))]
    pub use axecs_macro::Bundle;

    #[doc(inline)]
    pub use crate::component::query::{ With, Without, And, Nand, Or, Nor, Xor, Xnor };

    #[doc(inline)]
    pub use crate::query::Scoped;

    #[doc(inline)]
    pub use crate::system::{ IntoSystem, In, Local };

    #[doc(inline)]
    pub use crate::schedule::label::{ PreStartup, Startup, Cycle, Shutdown, PostShutdown };
    #[doc(inline)]
    pub use crate::schedule::system::{ IntoScheduledSystemConfig, IntoConditionallyScheduledSystemConfig };

    /// TODO: Doc comment
    #[cfg(feature = "derive")]
    #[doc(cfg(feature = "derive"))]
    pub use axecs_macro::Label;

}
