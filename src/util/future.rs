//! TODO: Doc comments


use std::pin::Pin;
use core::task::{ Context, Poll };
use core::future::join;


/// TODO: Doc comments
pub struct FunctionCallFuture<T, F : FnMut() -> Poll<T>> {
    f : F
}

impl<T, F : FnMut() -> Poll<T>> Unpin for FunctionCallFuture<T, F> { }

impl<T, F : FnMut() -> Poll<T>> FunctionCallFuture<T, F> {

    /// TODO: Doc comments
    pub fn new(f : F) -> Self { Self {
        f
    } }

}

impl<T, F : FnMut() -> Poll<T>> Future for FunctionCallFuture<T, F> {
    type Output = T;

    fn poll(mut self : Pin<&mut Self>, _ctx : &mut Context<'_>) -> Poll<Self::Output> {
        (self.f)()
    }
}


/// TODO: Doc comments
pub(crate) macro multijoin {

    ( $(,)? ) => { () },

    ( $single:expr $(,)? ) => { ( $single.await , ) },

    ( $( $multiple:expr ),+ $(,)? ) => { join!( $( $multiple , )* ).await }

}
