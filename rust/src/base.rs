use crate::DEFAULT_PAGE_SIZE;
use std::io;
use std::io::Error;

use std::ptr::NonNull;

pub(crate) struct Base<T> {
    pub ptr: NonNull<[T]>,
    pub occupied: usize,
}

impl<T> Base<T> {
    pub const PAGE_SIZE: usize = DEFAULT_PAGE_SIZE;
    pub const MINIMUM_CAPACITY: usize = Self::PAGE_SIZE;

    pub fn new(ptr: NonNull<[T]>) -> Self {
        Self { ptr, occupied: 0 }
    }

    pub fn allocated(&self) -> usize {
        self.ptr.len()
    }

    pub fn occupy(&mut self, capacity: usize) -> io::Result<()> {
        if capacity <= self.allocated() {
            self.occupied = capacity;
            Ok(())
        } else {
            Err(Error::other(format!(
                "cannot occupy {} - allocated only {}",
                capacity,
                self.allocated()
            )))
        }
    }
}
