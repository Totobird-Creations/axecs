//! A single column in an [`Archetype`](crate::archetype::Archetype), and its cells.


use crate::component::{ Component, ComponentTypeInfo };
use core::any::TypeId;
use core::alloc::Layout;
use core::ptr::NonNull;
use std::alloc::{ alloc, dealloc, handle_alloc_error };


/// A single column of an [`Archetype`](crate::archetype::Archetype), storing a single [`Component`] type.
pub struct ArchetypeColumn {

    /// The [`ComponentTypeInfo`] of the [`Component`] type stored in this column.
    type_info : ComponentTypeInfo,

    /// The allocated cells in this column.
    cells     : Vec<ArchetypeCell>

}

impl ArchetypeColumn {

    /// Creates a new column with the given [`ComponentTypeInfo`].
    ///
    /// # Safety:
    /// [`ArchetypeColumn`] does not properly clean itself up on drop.
    /// [`ArchetypeColumn::drop_dealloc_except`] must be called to properly deallocate.
    pub unsafe fn new(type_info : ComponentTypeInfo) -> Self { Self {
        type_info,
        cells     : Vec::new()
    } }

    /// The [`TypeId`] of the [`Component`] type stored in this column.
    pub fn type_id(&self) -> TypeId {
        self.type_info.type_id()
    }

    /// The [`Layout`] of the [`Component`] type stored in this column.
    pub fn type_layout(&self) -> Layout {
        self.type_info.layout()
    }

    /// The [`Drop`]per of the [`Component`] type stored in this column.
    pub fn type_drop(&self) -> unsafe fn(NonNull<u8>) -> () {
        self.type_info.drop()
    }

    /// The [`type_name`](::core::any::type_name) of the [`Component`] type stored in this column.
    #[cfg(any(debug_assertions, feature = "keep_debug_names"))]
    #[doc(cfg(feature = "keep_debug_names"))]
    pub fn type_name(&self) -> &'static str {
        self.type_info.name()
    }

    /// Returns a reference to the value in a cell by `index`.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell at the given `index` **is occupied**.
    /// - `C` is the type stored in this column.
    pub unsafe fn get_ref<C : Component>(&self, index : usize) -> &C {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.cells.get_unchecked(index).get_ref::<C>() }
    }

    /// Returns a mutable reference to the value in a cell by `index`.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell at the given `index` **is occupied**.
    /// - `C` is the type stored in this column.
    pub unsafe fn get_mut<C : Component>(&mut self, index : usize) -> &mut C {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.cells.get_unchecked_mut(index).get_mut::<C>() }
    }

    /// Returns a pointer to the value in a cell by `index`.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell at the given `index` **is occupied**.
    /// - `C` is the type stored in this column.
    /// - the pointer is not used when the cell is unoccupied, or this column is dropped.
    /// - data-races are prevented.
    pub unsafe fn get_ptr<C : Component>(&self, index : usize) -> *mut C {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.cells.get_unchecked(index).get_ptr::<C>() }
    }

    /// Pushes a new cell onto this column.
    ///
    /// Consider the new cell **occupied**.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that `C` is the type stored in this column.
    pub unsafe fn push<C : Component>(&mut self, component : C) {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        // SAFETY: The caller is responsible for properly disposing of this column and its cells.
        self.cells.push(unsafe{ ArchetypeCell::new(component) });
    }


    /// Replaces a cell on this column by `index`, without dropping the previous value.
    ///
    /// After the operation, consider this cell **occupied**.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that:
    /// - the cell at the given `index` **is not occupied**.
    /// - `C` is the type stored in this column.
    pub unsafe fn write<C : Component>(&mut self, index : usize, component : C) {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.cells.get_unchecked_mut(index).write(component) }
    }

    /// Drops the value stored in a cell by `index`.
    ///
    /// After the operation, consider this cell **unoccupied**.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that:
    /// - the cell at the given `index` **is occupied**.
    /// - `C` is the type stored in this column.
    pub unsafe fn drop<C : Component>(&mut self, index : usize) {
        let drop = self.type_drop();
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.cells.get_unchecked_mut(index).drop(drop); }
    }

    /// Reads the value stored in a cell by `index`, without modifying the memory.
    ///
    /// After the operation, consider this cell **unoccupied**.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that:
    /// - the cell at the given `index` **is occupied**.
    /// - `C` is the type stored in this column.
    pub unsafe fn read<C : Component>(&self, index : usize) -> C {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.cells.get_unchecked(index).read::<C>() }
    }

    /// Drops all cells except the given indices, and deallocates all cells' memory.
    ///
    /// After the operation, this [`ArchetypeColumn`] **must not be used again**.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that:
    /// - `except_indices` contains the index for **every cell that is unoccupied**. No more, no less.
    pub unsafe fn drop_dealloc_except(&mut self, except_indices : &[usize]) {
        let layout = self.type_layout();
        let drop   = self.type_drop();
        for (i, cell) in self.cells.iter_mut().enumerate() {
            if (! except_indices.contains(&i)) {
                // SAFETY: The caller is responsible for upholding the safety guarantees.
                unsafe{ cell.drop(drop); }
            }
            // SAFETY: The caller is responsible for upholding the safety guarantees.
            unsafe{ cell.dealloc(layout); }
        }
    }

}


/// A single cell in an [`Archetype`](crate::archetype::Archetype).
pub struct ArchetypeCell {

    /// A pointer to the contained value.
    data_ptr : NonNull<u8>

}

impl ArchetypeCell {

    /// Creates a new cell with the given [`Component`] type.
    ///
    /// Consider the new cell **occupied**.
    ///
    /// # Safety:
    /// [`ArchetypeCell`] does not properly clean itself up on drop.
    /// [`ArchetypeCell::drop`] and [`ArchetypeCell::dealloc`] must be called to properly deallocate.
    pub unsafe fn new<C : Component>(component : C) -> Self {
        let layout = Layout::new::<C>();
        let data_ptr = unsafe{ alloc(layout) };
        if (data_ptr.is_null()) {
            handle_alloc_error(layout)
        }
        unsafe{ data_ptr.cast::<C>().write(component); }
        Self {
            // SAFETY: An alloc error was emitted above, if `data_ptr` was `is_null`.
            data_ptr : unsafe{ NonNull::new_unchecked(data_ptr) }
        }
    }

    /// Returns a reference to the value in the cell.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell **is occupied**.
    /// - `C` is the type stored in this cell.
    pub unsafe fn get_ref<C : Component>(&self) -> &C {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.data_ptr.cast::<C>().as_ref() }
    }

    /// Returns a mutable reference to the value in the cell.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell **is occupied**.
    /// - `C` is the type stored in this cell.
    pub unsafe fn get_mut<C : Component>(&mut self) -> &mut C {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.data_ptr.cast::<C>().as_mut() }
    }

    /// Returns a pointer to the value in the cell.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell **is occupied**.
    /// - `C` is the type stored in this cell.
    /// - the pointer is not used when the cell is unoccupied or has been dropped.
    /// - data-races are prevented.
    pub unsafe fn get_ptr<C : Component>(&self) -> *mut C {
        self.data_ptr.cast::<C>().as_ptr()
    }

    /// Reads the value stored in the cell, without modifying the memory.
    ///
    /// After the operation, consider this cell **unoccupied**.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell **is occupied**.
    /// - `C` is the type stored in this cell.
    pub unsafe fn read<C : Component>(&self) -> C {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.data_ptr.cast::<C>().read() }
    }

    /// Replaces the value stored in the cell, without dropping the previous value.
    ///
    /// After the operation, consider this cell **occupied**.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell **is unoccupied**.
    /// - `C` is the type stored in this cell.
    pub unsafe fn write<C : Component>(&mut self, component : C) {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ self.data_ptr.cast::<C>().write(component); }
    }

    /// Drops the value stored in this cell.
    ///
    /// After the operation, consider this cell **unoccupied**.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell **is occupied**.
    /// - `C` is the type stored in this cell.
    pub unsafe fn drop(&mut self, destructor : unsafe fn(NonNull<u8>) -> ()) {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ destructor(self.data_ptr); }
    }

    /// Deallocates this cell's memory.
    ///
    /// After the operation, this [`ArchetypeCell`] **must not be used again**.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that
    /// - the cell **is unoccupied**.
    /// - `layout` matches the [`Layout`] of the value stored in this cell.
    pub unsafe fn dealloc(&mut self, layout : Layout) {
        // SAFETY: The caller is responsible for upholding the safety guarantees.
        unsafe{ dealloc(self.data_ptr.as_ptr(), layout); }
    }

}
