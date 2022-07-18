use crate::{Error, Result, DEFAULT_PAGE_SIZE};
use std::ptr::NonNull;

pub(crate) struct Base<T> {
    pub ptr: NonNull<[T]>,
    pub occupied: usize,
}

impl<T> Base<T> {
    pub const MIN_CAPACITY: usize = DEFAULT_PAGE_SIZE;

    pub const fn new(ptr: NonNull<[T]>) -> Self {
        Self { ptr, occupied: 0 }
    }

    pub const fn dangling() -> Self {
        Self::new(NonNull::slice_from_raw_parts(NonNull::dangling(), 0))
    }

    pub fn allocated(&self) -> usize {
        self.ptr.len()
    }

    pub fn occupy(&mut self, capacity: usize) -> Result<()> {
        let allocated = self.allocated();
        if capacity <= allocated {
            self.occupied = capacity;
            Ok(())
        } else {
            Err(Error::OverOccupy {
                allocated,
                to_occupy: capacity,
            })
        }
    }
}
