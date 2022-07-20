use crate::{base::Base, internal, IsTrue, RawMem, Result};
use std::{
    alloc::{Allocator, Layout},
    cmp::Ordering,
    mem::size_of,
    ptr,
    ptr::NonNull,
};

pub struct Alloc<T, A: Allocator> {
    base: Base<T>,
    alloc: A,
}

impl<T: Default, A: Allocator> Alloc<T, A> {
    pub const fn new(alloc: A) -> Self {
        Self {
            base: Base::dangling(),
            alloc,
        }
    }

    unsafe fn alloc_impl(&mut self, capacity: usize) -> Result<&mut [T]> {
        let old_capacity = self.base.ptr.len();
        let new_capacity = capacity;

        let result: Result<_> = try {
            if self.base.ptr.as_non_null_ptr() == NonNull::dangling() {
                let layout = Layout::array::<T>(capacity)?;
                self.alloc.allocate_zeroed(layout)?
            } else {
                let old_layout = Layout::array::<T>(old_capacity)?;
                let new_layout = Layout::array::<T>(new_capacity)?;

                let ptr = internal::align_from(self.base.ptr);
                match new_capacity.cmp(&old_capacity) {
                    Ordering::Less => {
                        self.base.handle_narrow(new_capacity);
                        self.alloc
                            .shrink(ptr.as_non_null_ptr(), old_layout, new_layout)?
                    }
                    Ordering::Greater => {
                        self.alloc
                            .grow(ptr.as_non_null_ptr(), old_layout, new_layout)?
                    }
                    Ordering::Equal => ptr,
                }
            }
        };

        result.map(|ptr| {
            self.base.ptr = internal::guaranteed_align_to(ptr);
            self.base.handle_expand(old_capacity);
            self.base.ptr.as_mut()
        })
    }
}

impl<T: Default, A: Allocator> RawMem<T> for Alloc<T, A>
where
    (): IsTrue<{ size_of::<T>() != 0 }>,
{
    fn alloc(&mut self, capacity: usize) -> Result<&mut [T]> {
        unsafe { self.alloc_impl(capacity) }
    }

    fn allocated(&self) -> usize {
        self.base.allocated()
    }

    fn occupy(&mut self, capacity: usize) -> Result<()> {
        self.base.occupy(capacity)
    }

    fn occupied(&self) -> usize {
        self.base.occupied
    }
}

impl<T, A: Allocator> Drop for Alloc<T, A> {
    fn drop(&mut self) {
        // SAFETY: ptr is valid slice
        // SAFETY: items is friendly to drop
        unsafe {
            let slice = self.base.ptr.as_mut();
            for item in slice {
                ptr::drop_in_place(item);
            }
        }

        let _: Result<_> = try {
            let ptr = self.base.ptr;
            let layout = Layout::array::<T>(ptr.len())?;
            // SAFETY: ptr is valid slice
            unsafe {
                let ptr = ptr.as_non_null_ptr().cast();
                self.alloc.deallocate(ptr, layout);
            }
        };
    }
}

unsafe impl<T: Sync, A: Allocator + Sync> Sync for Alloc<T, A> {}
unsafe impl<T: Send, A: Allocator + Send> Send for Alloc<T, A> {}
