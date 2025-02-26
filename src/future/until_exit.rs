//! TODO: Doc comment


use crate::world::Commands;
use core::pin::Pin;
use core::task::{ Context, Poll };
use pin_project::pin_project;


/// TODO: Doc comment
#[pin_project]
pub struct UntilExitFuture<T, F : Future<Output = T>> {

    /// TODO: Doc comment
    cmds : Commands,

    /// TODO: Doc comment
    #[pin]
    fut  : F

}

impl<T, F : Future<Output = T>> UntilExitFuture<T, F> {

    /// TODO: Doc comment
    pub fn new(cmds : Commands, fut : F) -> Self { Self {
        cmds,
        fut
    } }

}

impl<T, F : Future<Output = T>> Future for UntilExitFuture<T, F> {
    type Output = Option<T>;
    fn poll(self : Pin<&mut Self>, ctx : &mut Context<'_>) -> Poll<Self::Output> {
        if (self.cmds.is_exiting()) { Poll::Ready(None) }
        else {
            self.project().fut.poll(ctx).map(|out| Some(out))
        }
    }
}
