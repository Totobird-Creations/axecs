//! TODO: Doc comments


use core::mem::MaybeUninit;
use alloc::vec::Vec;


/// TODO: Doc comment
pub struct SparseVec<T> {
    items        : Vec<MaybeUninit<T>>,
    free_indices : Vec<usize>
}

impl<T> SparseVec<T> {

    /// TODO: Doc comment
    pub fn new() -> Self { Self {
        items        : Vec::new(),
        free_indices : Vec::new()
    } }

    /// TODO: Doc comment
    pub fn is_empty(&self) -> bool {
        self.free_indices.len() >= self.items.len()
    }

    /// TODO: Doc comment
    pub fn push(&mut self, item : T) {
        if let Some(index) = self.free_indices.pop() {
            // SAFETY: TODO
            unsafe{ self.items.get_unchecked_mut(index) }.write(item);
        } else {
            self.items.push(MaybeUninit::new(item));
        }
    }

    /// TODO: Doc comment
    pub fn append(&mut self, other : &mut Vec<T>) {
        for item in other.drain(..) {
            self.push(item);
        }
    }

    /// TODO: Doc comment
    pub fn retain<F : FnMut(&mut T) -> bool>(&mut self, mut f : F) {
        for i in 0..self.items.len() {
            if (! self.free_indices.contains(&i)) {
                // SAFETY: TODO
                let item = unsafe{ self.items.get_unchecked_mut(i) };
                // SAFETY: TODO
                if (! f(unsafe{ item.assume_init_mut() })) {
                    self.free_indices.push(i);
                    // SAFETY: TODO
                    unsafe{ item.assume_init_drop(); }
                }
            }
        }
    }

}

impl<T> Drop for SparseVec<T> {
    fn drop(&mut self) {
        for i in 0..self.items.len() {
            if (! self.free_indices.contains(&i)) {
                // SAFETY: TODO
                unsafe{ self.items.get_unchecked_mut(i).assume_init_drop(); }
            }
        }
    }
}
