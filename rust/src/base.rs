use crate::DEFAULT_PAGE_SIZE;
use std::ptr::{drop_in_place, NonNull};

pub(crate) struct Base<T> {
    pub ptr: NonNull<[T]>,
}

impl<T> Base<T> {
    pub const MIN_CAPACITY: usize = DEFAULT_PAGE_SIZE;

    pub const fn new(ptr: NonNull<[T]>) -> Self {
        Self { ptr }
    }

    pub const fn dangling() -> Self {
        Self::new(NonNull::slice_from_raw_parts(NonNull::dangling(), 0))
    }

    pub unsafe fn handle_narrow(&mut self, capacity: usize) {
        drop_in_place(&mut self.ptr.as_mut()[capacity..])
    }

    pub fn allocated(&self) -> usize {
        self.ptr.len()
    }
}

impl<T: Default> Base<T> {
    pub unsafe fn handle_expand(&mut self, capacity: usize) {
        let ptr = self.ptr.as_mut_ptr();
        for i in capacity..self.allocated() {
            ptr.add(i).write(T::default());
        }
    }
}
