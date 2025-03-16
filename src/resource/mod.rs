//! TODO: Doc comments


mod storage;
pub use storage::*;

mod query;
pub use query::*;


/// TODO: Doc comments
pub trait Resource : Sized + Send + Sync { }


/// [`Resource`] wrapping marker.
pub(crate) mod marker {
    use core::marker::PhantomData;
    /// Used in error messages and [`TypeId`](::core::any::TypeId) comparisons to indicate that a type is a [`Resource`](super::Resource).
    pub(super) struct Resource<C : super::Resource> {
        /// [`PhantomData`] on `C`.
        marker : PhantomData<C>
    }
}
