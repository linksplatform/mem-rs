use std::alloc::Layout;
use std::cmp::max;
use std::ops::Add;

use crate::{Base, RawMem};
use std::ptr::NonNull;
use std::{alloc, io, ptr};

pub struct GlobalMem {
    base: Base,
}

impl GlobalMem {
    pub fn reserve_new(mut capacity: usize) -> io::Result<Self> {
        capacity = max(capacity, Base::MINIMUM_CAPACITY);
        let mut new = GlobalMem {
            base: Base::new(NonNull::slice_from_raw_parts(NonNull::dangling(), 0)),
        };
        unsafe {
            new.on_reserved_impl(capacity, false)?;
        }
        Ok(new)
    }

    pub fn new() -> std::io::Result<Self> {
        Self::reserve_new(Base::MINIMUM_CAPACITY)
    }

    fn layout_impl(capacity: usize) -> io::Result<Layout> {
        Layout::array::<u8>(capacity).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    unsafe fn on_reserved_impl(
        &mut self,
        new_capacity: usize,
        reallocate: bool,
    ) -> io::Result<NonNull<[u8]>> {
        let old_capacity = self.base.allocated();
        self.base.alloc(old_capacity)?;

        if !reallocate {
            let layout = Self::layout_impl(new_capacity)?;
            let ptr = alloc::alloc_zeroed(layout);
            self.base.set_ptr(NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(ptr),
                layout.size(),
            ));
        } else {
            let ptr = self.ptr();
            let layout = Self::layout_impl(old_capacity)?;
            let new = alloc::realloc(ptr.as_mut_ptr(), layout, new_capacity);
            if old_capacity < new_capacity {
                let offset = new_capacity - old_capacity;
                ptr::write_bytes(new.add(old_capacity), 0, offset);
            }
            self.base.set_ptr(NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(new),
                new_capacity,
            ));
        }
        Ok(self.ptr())
    }
}

impl RawMem for GlobalMem {
    fn ptr(&self) -> NonNull<[u8]> {
        self.base.ptr()
    }

    fn alloc(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>> {
        unsafe { self.on_reserved_impl(capacity, true) }
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

impl Drop for GlobalMem {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.ptr();
            let layout = Layout::array::<u8>(ptr.len()).unwrap(); // TODO: check later
            alloc::dealloc(ptr.as_mut_ptr(), layout)
        }
    }
}
