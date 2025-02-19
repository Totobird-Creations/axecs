//! TODO: Doc comments


pub mod rwlock;

pub(crate) mod lazycell;

pub(crate) mod either;

pub(crate) mod sparsevec;


pub(crate) mod future;


#[cfg(any(debug_assertions, feature = "keep_debug_names"))]
#[doc(cfg(feature = "keep_debug_names"))]
pub(crate) mod unqualified;


pub(crate) mod variadic;
