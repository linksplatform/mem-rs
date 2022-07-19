use std::alloc::{AllocError, LayoutError};

// Bare metal platforms usually have very small amounts of RAM
// (in the order of hundreds of KB)
#[rustfmt::skip]
pub const DEFAULT_PAGE_SIZE: usize = if cfg!(target_os = "espidf") { 512 } else { 8 * 1024 };

// fixme: maybe we should add `(X bytes)` after `cannot allocate/occupy`
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid capacity to RawMem::alloc/occupy/grow/shrink")]
    CapacityOverflow,
    #[error("cannot allocate {to_alloc} - available only {available}")]
    OverAlloc { available: usize, to_alloc: usize },
    #[error("cannot occupy {to_occupy} - allocated only {allocated}")]
    OverOccupy { allocated: usize, to_occupy: usize },
    #[error(transparent)]
    AllocError(#[from] AllocError),
    #[error(transparent)]
    LayoutError(#[from] LayoutError),
    #[error(transparent)]
    System(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait RawMem<T> {
    fn alloc(&mut self, capacity: usize) -> Result<&mut [T]>;
    fn allocated(&self) -> usize;

    fn occupy(&mut self, capacity: usize) -> Result<()>;
    fn occupied(&self) -> usize;

    // fixme: maybe this should be return Option<usize> and None by default?
    fn size_hint(&self) -> usize {
        usize::MAX
    }

    fn grow(&mut self, capacity: usize) -> Result<&mut [T]> {
        self.allocated()
            .checked_sub(capacity)
            .ok_or(Error::CapacityOverflow)
            .and_then(|capacity| self.alloc(capacity))
    }

    fn shrink(&mut self, capacity: usize) -> Result<&mut [T]> {
        self.allocated()
            .checked_sub(capacity)
            .ok_or(Error::CapacityOverflow)
            .and_then(|capacity| self.alloc(capacity))
    }

    fn grow_occupied(&mut self, capacity: usize) -> Result<()> {
        self.occupied()
            .checked_add(capacity)
            .ok_or(Error::CapacityOverflow)
            .and_then(|capacity| self.occupy(capacity))
    }

    fn shrink_occupied(&mut self, capacity: usize) -> Result<()> {
        self.occupied()
            .checked_sub(capacity)
            .ok_or(Error::CapacityOverflow)
            .and_then(|capacity| self.occupy(capacity))
    }
}
