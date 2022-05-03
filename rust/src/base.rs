use crate::{RawMem, HOPE_PAGE_SIZE};
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::ptr::NonNull;

pub(crate) struct Base {
    occupied: usize,
    allocated: usize,
    ptr: NonNull<[u8]>,
}

impl Base {
    pub const PAGE_SIZE: usize = HOPE_PAGE_SIZE;
    pub const MINIMUM_CAPACITY: usize = Self::PAGE_SIZE;

    pub fn new(ptr: NonNull<[u8]>) -> Self {
        Self {
            occupied: 0,
            allocated: 0,
            ptr,
        }
    }

    pub fn set_ptr(&mut self, ptr: NonNull<[u8]>) {
        self.ptr = ptr;
    }
}

impl RawMem for Base {
    fn ptr(&self) -> NonNull<[u8]> {
        self.ptr
    }

    #[rustfmt::skip]
    fn alloc(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>> {
        //if capacity >= self.allocated {
        self.allocated = capacity;
        Ok(self.ptr)
        // TODO:
        // } else {
        //     Err(Error::new(
        //         ErrorKind::Other,
        //         "cannot reserve less than the memory occupied",
        //     ))
        // }
    }

    fn allocated(&self) -> usize {
        self.allocated
    }

    fn occupy(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>> {
        if capacity <= self.allocated {
            self.occupied = capacity;
            Ok(self.ptr)
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!(
                    "cannot occupy ({}) greater than the memory allocated ({})",
                    capacity, self.allocated
                ),
            ))
        }
    }

    fn occupied(&self) -> usize {
        self.occupied
    }
}
