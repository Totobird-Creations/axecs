//! TODO: Doc comments


use crate::util::either::Either;
use core::pin::Pin;
use core::task::{ Context, Poll };
use core::future::join;
use core::hint::unreachable_unchecked;
use alloc::boxed::Box;
use alloc::vec::Vec;


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
pub struct VecMultijoinFuture<T, F : Future<Output = T>> {
    futures : Option<Vec<Either<Pin<Box<F>>, T>>>
}

impl<T, F : Future<Output = T>> Unpin for VecMultijoinFuture<T, F> { }

impl<T, F : Future<Output = T>> VecMultijoinFuture<T, F> {

    /// TODO: Doc comments
    pub fn new(futures : impl IntoIterator<Item = Pin<Box<F>>>) -> Self { Self {
        futures : Some(futures.into_iter().map(|fut| Either::A(fut)).collect::<Vec<_>>())
    } }

}

impl<T, F : Future<Output = T>> Future for VecMultijoinFuture<T, F> {
    type Output = Vec<T>;

    fn poll(mut self : Pin<&mut Self>, ctx : &mut Context<'_>) -> Poll<Self::Output> {
        let mut done = true;
        for entry in self.futures.as_mut().unwrap() {
            if let Either::A(fut) = entry {
                match (fut.as_mut().poll(ctx)) {
                    Poll::Ready(out) => { *entry = Either::B(out); },
                    Poll::Pending => { done = false; }
                }
            }
        }
        if (done) {
            Poll::Ready(self.futures.take().unwrap().into_iter().map(|fut| {
                // SAFETY: TODO
                if let Either::B(out) = fut { out } else { unsafe{ unreachable_unchecked() } }
            }).collect::<Vec<_>>())
        } else {
            Poll::Pending
        }
    }
}


/// TODO: Doc comments
pub(crate) macro multijoin {

    ( $(,)? ) => { () },

    ( $single:expr $(,)? ) => { ( $single.await , ) },

    ( $( $multiple:expr ),+ $(,)? ) => { join!( $( $multiple , )* ).await }

}
