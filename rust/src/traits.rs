use std::alloc::{AllocError, LayoutError};

// Bare metal platforms usually have very small amounts of RAM
// (in the order of hundreds of KB)
/// RAM page size which is likely to be the same on most systems
#[rustfmt::skip]
pub const DEFAULT_PAGE_SIZE: usize = if cfg!(target_os = "espidf") { 512 } else { 8 * 1024 };

/// Error memory allocation
// fixme: maybe we should add `(X bytes)` after `cannot allocate/occupy`
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Error due to the computed capacity exceeding the maximum
    /// (usually `usize::MAX` bytes).
    #[error("invalid capacity to RawMem::alloc/occupy/grow/shrink")]
    CapacityOverflow,
    /// Cannot to `allocate` more than `available`
    #[error("cannot allocate {to_alloc} - available only {available}")]
    OverAlloc { available: usize, to_alloc: usize },
    /// Cannot to `occupy` more than `allocated`
    #[error("cannot occupy {to_occupy} - allocated only {allocated}")]
    OverOccupy { allocated: usize, to_occupy: usize },
    /// Memory allocator return an error
    #[error(transparent)]
    AllocError(#[from] AllocError),
    /// Memory allocator accept incorrect [`Layout`](std::alloc::Layout)
    #[error(transparent)]
    LayoutError(#[from] LayoutError),
    /// System error memory allocation occurred
    #[error(transparent)]
    System(#[from] std::io::Error),
}

/// Alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// The implementation of `RawMem` can allocate, increase, decrease one arbitrary block
/// of elements of the `T` type
///
/// Only one block can exist at time, so mut slice `&mut [T]` is returned to it
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// #![feature(allocator_api)]
///
/// use std::alloc::Global;
/// use platform_mem::{RawMem, Alloc};
///
/// // `RawMem` when alloc memory via any `Allocator`
/// let mut mem = Alloc::<usize, _>::new(Global);
/// let slice = mem.alloc(10).unwrap();
///
/// slice.copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
///
/// // get new ref after realloc
/// let slice = mem.grow(10).unwrap();
/// assert_eq!(slice, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
///
/// slice[0..5].reverse();
///
/// let slice = mem.shrink(15).unwrap();
/// assert_eq!(slice, &[5, 4, 3, 2, 1]);
/// ```
pub trait RawMem<T> {
    /// Allocate or reserve a block of memory of the given `capacity`.
    /// If block is already allocated, it will be shrink or grow with data retention.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// // alloc mem via `std::alloc`
    /// use platform_mem::{RawMem, Global};
    ///
    /// let mut mem = Global::<usize>::new();
    ///
    /// let slice = mem.alloc(10).unwrap();
    /// assert_eq!(slice.len(), 10);
    ///
    /// let slice = mem.alloc(20).unwrap();
    /// assert_eq!(slice.len(), 20);
    fn alloc(&mut self, capacity: usize) -> Result<&mut [T]>;

    /// Current allocated elements count. Must be equal `alloc` result length.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use platform_mem::{RawMem, Global};
    ///
    /// let mut mem = Global::<usize>::new();
    ///
    /// let slice = mem.alloc(10).unwrap();
    /// assert_eq!(slice.len(), mem.allocated());
    /// ```
    fn allocated(&self) -> usize;

    /// Occupy a block of memory of the given `capacity`.
    fn occupy(&mut self, capacity: usize) -> Result<()>;

    /// Current occupied elements count.
    fn occupied(&self) -> usize;

    /// Returns the boundary (in count of elements) on the available elements.
    ///
    /// A [`usize::MAX`] here means that `RawMem` can allocate memory indefinitely
    /// (as long as the system allows)
    ///
    /// # Implementation notes
    ///
    /// It is not enforced that an `RawMem` implementation yields the declared available elements.
    /// A buggy `RawMem` may yield less than  the upper bound of elements.
    ///
    /// `size_hint()` is primarily intended to be used for limited `RawMem` implementors,
    /// for example, reserving space without getting an error
    /// when the available memory limit is exceeded
    ///
    /// The default implementation returns [`usize::MAX`] which is correct for any `RawMem`,
    /// but it can interfere when approaching the boundary of available elements
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::cmp::min;
    /// use platform_mem::{PreAlloc, RawMem};
    ///
    /// let mut mem = PreAlloc::new(vec![0; 100]);
    ///
    /// let crazy_capacity = usize::MAX;
    /// let _ = mem.alloc(crazy_capacity).unwrap_err();
    ///
    /// let smart_capacity = min(crazy_capacity, mem.size_hint());
    /// let block = mem.alloc(smart_capacity).unwrap();
    ///
    /// assert_eq!(block.len(), 100);
    /// ```
    // fixme: maybe this should be return Option<usize> and None by default?
    fn size_hint(&self) -> usize {
        usize::MAX
    }

    /// Attempts to grow occupied memory.
    ///
    /// # Errors
    ///
    /// Returns error if the `allocated + capacity` overflowing
    fn grow(&mut self, capacity: usize) -> Result<&mut [T]> {
        self.allocated()
            .checked_add(capacity)
            .ok_or(Error::CapacityOverflow)
            .and_then(|capacity| self.alloc(capacity))
    }

    /// Attempts to shrink the memory block.
    ///
    /// # Errors
    ///
    /// Returns error if the `allocated - capacity` overflowing
    fn shrink(&mut self, capacity: usize) -> Result<&mut [T]> {
        self.allocated()
            .checked_sub(capacity)
            .ok_or(Error::CapacityOverflow)
            .and_then(|capacity| self.alloc(capacity))
    }

    /// Attempts to grow occupied memory.
    ///
    /// # Errors
    ///
    /// Returns error if the occupied memory is already at the upper bound
    /// (that is, when `occupied + capacity` is great than `allocated` or overflowing).
    fn grow_occupied(&mut self, capacity: usize) -> Result<()> {
        self.occupied()
            .checked_add(capacity)
            .ok_or(Error::CapacityOverflow)
            .and_then(|capacity| self.occupy(capacity))
    }

    /// Attempts to shrink occupied memory.
    ///
    /// # Errors
    ///
    /// Returns error if the occupied memory is less than `capacity` (that is, when overflowing).
    fn shrink_occupied(&mut self, capacity: usize) -> Result<()> {
        self.occupied()
            .checked_sub(capacity)
            .ok_or(Error::CapacityOverflow)
            .and_then(|capacity| self.occupy(capacity))
    }
}
