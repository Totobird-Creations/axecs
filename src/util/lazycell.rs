//! TODO: Doc comment


use core::mem::MaybeUninit;
use core::marker::PhantomData;


/// TODO: Doc comment
pub(crate) struct LazyCell<T, A, F : FnOnce(A) -> T> {

    /// TODO: Doc comment
    init   : Option<F>,

    /// TODO: Doc comment
    inner  : MaybeUninit<T>,

    /// TODO: Doc comment
    marker : PhantomData<fn(A) -> T>

}

impl<T, A, F : FnOnce(A) -> T> LazyCell<T, A, F> {

    /// TODO: Doc comment
    pub(crate) fn new(f : F) -> Self { Self {
        init   : Some(f),
        inner  : MaybeUninit::uninit(),
        marker : PhantomData
    } }

    /// TODO: Doc comment
    pub(crate) fn get_mut(&mut self, a : A) -> &mut T {
        if let Some(init) = self.init.take() {
            // SAFETY: TODO
            self.inner.write(init(a));
        }
        // SAFETY: TODO
        unsafe{ self.inner.assume_init_mut() }
    }

}

impl<T, A, F : FnOnce(A) -> T> Drop for LazyCell<T, A, F> {
    fn drop(&mut self) {
        if (self.init.is_none()) {
            // SAFETY: TODO
            unsafe{ self.inner.assume_init_drop(); }
        }
    }
}
