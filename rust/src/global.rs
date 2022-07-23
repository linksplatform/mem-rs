use crate::{internal, Base, IsTrue, RawMem, Result};
use std::{
    alloc::{self, Layout},
    mem::size_of,
    ptr::{self, NonNull},
};
use tap::Pipe;

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
        let new_in_bytes = new_capacity * size_of::<T>();
        let ptr = if self.0.ptr.as_non_null_ptr() == NonNull::dangling() {
            Self::layout_impl(new_capacity)?
                .pipe(|layout| alloc::alloc(layout))
                .pipe(|ptr| NonNull::new_unchecked(ptr))
                .pipe(|ptr| NonNull::slice_from_raw_parts(ptr, new_in_bytes))
        } else {
            if new_capacity < old_capacity {
                self.0.handle_narrow(new_capacity);
            }

            let ptr = internal::to_bytes(self.0.ptr).as_mut_ptr();
            let layout = Self::layout_impl(old_capacity)?;

            alloc::realloc(ptr, layout, new_in_bytes)
                .pipe(|ptr| NonNull::new_unchecked(ptr))
                .pipe(|ptr| NonNull::slice_from_raw_parts(ptr, new_in_bytes))
        };

        self.0.ptr = internal::guaranteed_from_bytes(ptr);
        self.0.handle_expand(old_capacity);
        self.0.ptr.as_mut().pipe(Ok)
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
}

impl<T> Drop for Global<T> {
    fn drop(&mut self) {
        // SAFETY: ptr is valid slice
        // items is friendly to drop
        unsafe { self.0.ptr.as_mut().pipe(|slice| ptr::drop_in_place(slice)) }

        let _: Result<_> = try {
            let ptr = self.0.ptr;
            let layout = Self::layout_impl(ptr.len())?;
            // SAFETY: ptr is valid slice
            unsafe {
                ptr.as_non_null_ptr()
                    .cast::<u8>()
                    .as_ptr()
                    .pipe(|ptr| alloc::dealloc(ptr, layout))
            }
        };
    }
}

unsafe impl<T: Sync> Sync for Global<T> {}
unsafe impl<T: Send> Send for Global<T> {}
