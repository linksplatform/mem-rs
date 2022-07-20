use crate::{internal, Base, IsTrue, RawMem, Result};
use std::{
    alloc::{self, Layout},
    mem::size_of,
    ptr::{self, NonNull},
};

pub struct Global<T>(Base<T>);

impl<T> Global<T> {
    pub const fn new() -> Self {
        Self(Base::dangling())
    }

    fn layout_impl(capacity: usize) -> Result<Layout> {
        Layout::array::<T>(capacity).map_err(Into::into)
    }
}

impl<T: Default> Global<T> {
    unsafe fn on_reserved_impl(&mut self, new_capacity: usize) -> Result<&mut [T]> {
        let old_capacity = self.0.allocated();
        let ptr = if self.0.ptr.as_non_null_ptr() == NonNull::dangling() {
            let layout = Self::layout_impl(new_capacity)?;
            let ptr = alloc::alloc_zeroed(layout);
            NonNull::slice_from_raw_parts(NonNull::new_unchecked(ptr), layout.size())
        } else {
            if new_capacity < old_capacity {
                self.0.handle_narrow(new_capacity);
            }

            let new_capacity = new_capacity * size_of::<T>();
            let ptr = internal::align_from(self.0.ptr);
            let layout = Self::layout_impl(old_capacity)?;
            let new = alloc::realloc(ptr.as_mut_ptr(), layout, new_capacity);
            NonNull::slice_from_raw_parts(NonNull::new_unchecked(new), new_capacity)
        };

        self.0.ptr = internal::guaranteed_align_to(ptr);
        self.0.handle_expand(old_capacity);
        Ok(self.0.ptr.as_mut())
    }
}

impl<T: Default> const Default for Global<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default> RawMem<T> for Global<T>
where
    (): IsTrue<{ size_of::<T>() != 0 }>,
{
    fn alloc(&mut self, capacity: usize) -> Result<&mut [T]> {
        unsafe { self.on_reserved_impl(capacity) }
    }

    fn allocated(&self) -> usize {
        self.0.allocated()
    }

    fn occupy(&mut self, capacity: usize) -> Result<()> {
        self.0.occupy(capacity)
    }

    fn occupied(&self) -> usize {
        self.0.occupied
    }
}

impl<T> Drop for Global<T> {
    fn drop(&mut self) {
        // SAFETY: ptr is valid slice
        // SAFETY: items is friendly to drop
        unsafe {
            let slice = self.0.ptr.as_mut();
            for item in slice {
                ptr::drop_in_place(item);
            }
        }

        let _: Result<_> = try {
            let ptr = self.0.ptr;
            let layout = Self::layout_impl(ptr.len())?;
            // SAFETY: ptr is valid slice
            unsafe {
                let ptr = ptr.as_non_null_ptr().cast();
                alloc::dealloc(ptr.as_ptr(), layout);
            }
        };
    }
}

unsafe impl<T: Sync> Sync for Global<T> {}
unsafe impl<T: Send> Send for Global<T> {}
