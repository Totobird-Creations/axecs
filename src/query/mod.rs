//! TODO: Doc comments


mod impls;

mod validate;
pub use validate::*;

mod state;
pub use state::*;


use crate::world::World;
#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
use crate::util::unqualified::UnqualifiedTypeName;
use core::task::Poll;


/// TODO: Doc comments
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid query"
)]
pub unsafe trait Query : Sized {

    /// TODO: Doc comments
    type Item<'item>;

    /// TODO: Doc comments
    type State<'state> = ();

    /// TODO: Doc comments
    fn init_state<'world>(world : &'world World) -> Self::State<'world>;

    /// TODO: Doc comments
    ///
    /// SAFETY
    /// The caller is responsible for ensuring that the query does not violate any borrow checker or archetype rules.
    /// See [`QueryValidator`] and [`BundleValidator`](crate::component::BundleValidator).
    unsafe fn acquire<'world>(world : &'world World, state : &mut Self::State<'world>) -> Poll<QueryAcquireResult<Self::Item<'world>>>;

    /// Traverses the types in this bundle, joining them to a [`QueryValidator`].
    ///
    /// After the entire [`QueryValidator`] has been constructed, [`QueryValidator::panic_on_violation`] will be called.
    /// The implementation of this method should not call [`QueryValidator::panic_on_violation`].
    ///
    /// See [`QueryValidator::empty`], [`QueryValidator::of_immutable`], [`QueryValidator::of_mutable`], [`QueryValidator::of_owned`], and [`QueryValidator::join`].
    ///
    /// # Safety
    /// The implementation of this method **must** include and join every type that this [`Query`] requests, each with the correct access type.
    fn validate() -> QueryValidator;
}
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid read-only query"
)]

/// A marker that promises a [`Query`] will not grant mutable or owned access to any values.
pub unsafe trait ReadOnlyQuery : Query { }


/// The result of a [`Query`].
pub enum QueryAcquireResult<T> {

    /// The [`Query`] acquired the value successfully.
    Ready(T),

    /// The requested value does not exist.
    DoesNotExist {
        /// The [`type_name`](::core::any::type_name) of the value that does not exist.
        #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
        #[doc(cfg(feature = "keep_debug_names"))]
        name : &'static str
    }

}

impl<T> QueryAcquireResult<T> {

    /// Returns the value in [`QueryAcquireResult::Ready`], or panics if the value could not be acquired for some reason.
    #[track_caller]
    pub fn unwrap(self) -> T {
        match (self) {

            QueryAcquireResult::Ready(out) => out,

            #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
            QueryAcquireResult::DoesNotExist { name } => { panic!("Query requested non-existant {}", unsafe{ UnqualifiedTypeName::from_unchecked(name) }) }
            #[cfg(not(any(debug_assertions, feature = "keep_debug_names")))]
            QueryAcquireResult::DoesNotExist { } => { panic!("Query requested non-existant item") }

        }
    }

}
