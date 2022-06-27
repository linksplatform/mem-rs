use crate::{base::Base, internal, RawMem};
use std::alloc::LayoutError;
use std::cmp::Ordering;
use std::{
    alloc::{Allocator, Layout},
    error::Error,
    io, ptr,
    ptr::NonNull,
};

pub struct AllocMem<T, A: Allocator> {
    base: Base<T>,
    alloc: A,
}

impl<T: Default, A: Allocator> AllocMem<T, A> {
    pub fn new(alloc: A) -> Self {
        Self {
            base: Base::new(NonNull::slice_from_raw_parts(NonNull::dangling(), 0)),
            alloc,
        }
    }

    unsafe fn alloc_impl(&mut self, capacity: usize) -> io::Result<&mut [T]> {
        let old_capacity = self.base.ptr.len();
        let new_capacity = capacity;

        let result: Result<_, Box<dyn Error + Sync + Send>> = try {
            if self.base.ptr.as_non_null_ptr() == NonNull::dangling() {
                let layout = Layout::array::<T>(capacity)?;
                self.alloc.allocate_zeroed(layout)?
            } else {
                let old_layout = Layout::array::<T>(old_capacity)?;
                let new_layout = Layout::array::<T>(new_capacity)?;

                let ptr = internal::align_from(self.base.ptr);
                match old_capacity.cmp(&new_capacity) {
                    Ordering::Less => {
                        self.alloc
                            .grow_zeroed(ptr.as_non_null_ptr(), old_layout, new_layout)?
                    }
                    Ordering::Greater => {
                        self.alloc
                            .shrink(ptr.as_non_null_ptr(), old_layout, new_layout)?
                    }
                    Ordering::Equal => ptr,
                }
            }
        };

        result
            .map(|ptr| {
                self.base.ptr = internal::guaranteed_align_to(ptr);
                for i in old_capacity..new_capacity {
                    self.base.ptr.as_mut_ptr().add(i).write(T::default());
                }
                self.base.ptr.as_mut()
            })
            .map_err(io::Error::other)
    }
}

impl<T: Default, A: Allocator> RawMem<T> for AllocMem<T, A> {
    fn alloc(&mut self, capacity: usize) -> io::Result<&mut [T]> {
        unsafe { self.alloc_impl(capacity) }
    }

    fn allocated(&self) -> usize {
        self.base.ptr.len()
    }

    fn occupy(&mut self, capacity: usize) -> io::Result<()> {
        self.base.occupy(capacity)
    }

    fn occupied(&self) -> usize {
        self.base.occupied
    }
}

impl<T, A: Allocator> Drop for AllocMem<T, A> {
    fn drop(&mut self) {
        // SAFETY: ptr is valid slice
        // SAFETY: items is friendly to drop
        unsafe {
            let slice = self.base.ptr.as_mut();
            for item in slice {
                ptr::drop_in_place(item);
            }
        }

        let _: Result<_, LayoutError> = try {
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
