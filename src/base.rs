use std::{
    marker::PhantomData,
    ptr::{drop_in_place, NonNull},
};

pub struct Base<T> {
    // fixme: use `Unique`
    pub ptr: NonNull<[T]>,
    expand: Box<dyn FnMut() -> T>,
    // for dropck: `RawMem` usually owns `T`
    marker: PhantomData<T>,
}

impl<T> Base<T> {
    /// currently you can use only `'static` functions
    /// otherwise API it will become more oriented to closure lifetime
    /// ```ignore
    /// // sad :(
    /// let closure: 'a = ...;
    /// let SomeMem<'a, T> = SomeMem::new(closure);
    ///
    /// // base :)
    /// let closure: 'static = ...;
    /// let SomeMem<T> = SomeMem::new(closure);
    /// ```
    pub fn new_with(ptr: NonNull<[T]>, with: impl FnMut() -> T + 'static) -> Self {
        Self {
            ptr,
            expand: Box::new(with),
            marker: PhantomData,
        }
    }

    pub fn dangling_with(with: impl FnMut() -> T + 'static) -> Self {
        Self::new_with(NonNull::slice_from_raw_parts(NonNull::dangling(), 0), with)
    }

    pub unsafe fn handle_narrow(&mut self, capacity: usize) {
        drop_in_place(&mut self.ptr.as_mut()[capacity..]);
    }

    pub const fn allocated(&self) -> usize {
        self.ptr.len()
    }

    pub unsafe fn handle_expand(&mut self, capacity: usize) {
        let ptr = self.ptr.as_mut_ptr();
        for i in capacity..self.allocated() {
            ptr.add(i).write((self.expand)());
        }
    }
}

pub fn default_expand<T: Default>() -> T {
    T::default()
}

impl<T: Default + 'static> Base<T> {
    pub fn new(ptr: NonNull<[T]>) -> Self {
        Self::new_with(ptr, default_expand)
    }

    pub fn dangling() -> Self {
        Self::new(NonNull::slice_from_raw_parts(NonNull::dangling(), 0))
    }
}
