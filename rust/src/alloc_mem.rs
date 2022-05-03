use std::alloc::{Allocator, Layout};

use crate::base::Base;
use crate::{RawMem, HOPE_PAGE_SIZE};
use std::error::Error;
use std::io;
use std::ptr::NonNull;

pub struct AllocMem<A: Allocator> {
    base: Base,
    alloc: A,
}

impl<A: Allocator> AllocMem<A> {
    pub fn new(alloc: A) -> io::Result<Self> {
        let mut new = Self {
            // SAFETY: immediately call `reserve_impl`
            base: Base::new(NonNull::slice_from_raw_parts(NonNull::dangling(), 0)),
            alloc,
        };
        unsafe {
            new.alloc_impl(HOPE_PAGE_SIZE, false)?;
        }
        Ok(new)
    }

    // TODO: Split to `alloc` and `realloc`
    unsafe fn alloc_impl(
        &mut self,
        capacity: usize,
        reallocate: bool,
    ) -> io::Result<NonNull<[u8]>> {
        let old_capacity = self.base.ptr().len();
        let new_capacity = capacity;

        let result: Result<(), Box<dyn Error + Sync + Send>> = try {
            if !reallocate {
                let layout = Layout::array::<u8>(capacity)?;
                self.base.set_ptr(self.alloc.allocate_zeroed(layout)?);
            } else {
                let old_layout = Layout::array::<u8>(old_capacity)?;
                let new_layout = Layout::array::<u8>(new_capacity)?;

                let ptr = self.base.ptr();

                if old_capacity < new_capacity {
                    self.base.set_ptr(self.alloc.grow_zeroed(
                        ptr.as_non_null_ptr(),
                        old_layout,
                        new_layout,
                    )?);
                } else if old_capacity > new_capacity {
                    self.base.set_ptr(self.alloc.shrink(
                        ptr.as_non_null_ptr(),
                        old_layout,
                        new_layout,
                    )?)
                }
            }
        };

        match result {
            Ok(_) => self.base.alloc(capacity),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    }
}

impl<A: Allocator> RawMem for AllocMem<A> {
    fn ptr(&self) -> NonNull<[u8]> {
        self.base.ptr()
    }
    
    fn alloc(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>> {
        unsafe { self.alloc_impl(capacity, true) }
    }

    fn allocated(&self) -> usize {
        self.base.allocated()
    }

    fn occupy(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>> {
        self.base.occupy(capacity)
    }

    fn occupied(&self) -> usize {
        self.base.occupied()
    }
}

impl<A: Allocator> Drop for AllocMem<A> {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.base.ptr();
            let layout = Layout::for_value_raw(ptr.as_ptr());
            self.alloc.deallocate(ptr.as_non_null_ptr(), layout);
        }
    }
}
