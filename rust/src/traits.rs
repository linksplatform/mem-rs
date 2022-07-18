use std::{
    alloc::{AllocError, LayoutError}, io,
};

// Bare metal platforms usually have very small amounts of RAM
// (in the order of hundreds of KB)
#[rustfmt::skip]
pub const DEFAULT_PAGE_SIZE: usize = if cfg!(target_os = "espidf") { 512 } else { 8 * 1024 };

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid capacity to RawMem::alloc/occupy/grow/shrink")]
    CapacityOverflow,
    #[error("cannot allocate {to_alloc} - available only {available}")]
    OverAlloc { available: usize, to_alloc: usize },
    #[error("cannot occupy {to_occupy} - allocated only {allocated}")]
    OverOccupy { allocated: usize, to_occupy: usize },
    #[error("{0}")]
    AllocError(#[from] AllocError),
    #[error("{0}")]
    LayoutError(#[from] LayoutError),
    #[error("{0}")]
    System(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait RawMem<T> {
    fn alloc(&mut self, capacity: usize) -> Result<&mut [T]>;
    fn allocated(&self) -> usize;

    fn occupy(&mut self, capacity: usize) -> Result<()>;
    fn occupied(&self) -> usize;

    fn size_hint(&self) -> usize {
        usize::MAX
    }

    fn grow(&mut self, capacity: usize) -> Result<&mut [T]> {
        let res: Option<_> = try {
            let to_alloc = self.allocated().checked_add(capacity)?;
            self.alloc(to_alloc)
        };
        res.unwrap_or(Err(Error::CapacityOverflow))
    }

    fn shrink(&mut self, capacity: usize) -> Result<&mut [T]> {
        let res: Option<_> = try {
            let to_alloc = self.allocated().checked_sub(capacity)?;
            self.alloc(to_alloc)
        };
        res.unwrap_or(Err(Error::CapacityOverflow))
    }

    fn grow_occupied(&mut self, capacity: usize) -> Result<()> {
        let res: Option<_> = try {
            let to_occupy = self.allocated().checked_add(capacity)?;
            self.occupy(to_occupy)
        };
        res.unwrap_or(Err(Error::CapacityOverflow))
    }

    fn shrink_occupied(&mut self, capacity: usize) -> Result<()> {
        let res: Option<_> = try {
            let to_occupy = self.allocated().checked_sub(capacity)?;
            self.occupy(to_occupy)
        };
        res.unwrap_or(Err(Error::CapacityOverflow))
    }
}
