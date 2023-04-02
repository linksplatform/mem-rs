use std::{alloc::Layout, mem::MaybeUninit};

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
    ///
    /// # Examples
    ///
    /// try grow/shrink more than `usize::MAX` bytes:
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #![feature(assert_matches)]
    /// # use std::alloc::Global;
    /// use std::assert_matches::assert_matches;
    /// # use platform_mem::{Error, Alloc, RawMem};
    ///
    /// let mut mem = Alloc::new(Global);
    ///
    /// assert_matches!(mem.grow_filled(usize::MAX, 0u64), Err(Error::CapacityOverflow));
    /// ```
    #[error("invalid capacity to RawMem::alloc/occupy/grow/shrink")]
    CapacityOverflow,

    #[error("cannot allocate {to_alloc} - available only {available}")]
    OverAlloc { available: usize, to_alloc: usize },

    /// The memory allocator returned an error
    #[error("memory allocation of {layout:?} failed")]
    AllocError {
        /// The layout of allocation request that failed
        layout: Layout,

        #[doc(hidden)]
        non_exhaustive: (),
    },

    /// System error memory allocation occurred
    #[error(transparent)]
    System(#[from] std::io::Error),
}

/// Alias for `Result<T, Error>` to return from `RawMem` methods
pub type Result<T> = std::result::Result<T, Error>;

pub trait RawMem {
    type Item;

    fn allocated(&mut self) -> &mut [Self::Item];

    /// # Safety
    /// Caller must guarantee that `fill` makes the uninitialized part valid for
    /// [`MaybeUninit::slice_assume_init_mut`]
    ///
    /// ### Incorrect usage
    /// ```no_run
    /// # #![feature(allocator_api)]
    /// # use std::alloc::Global;
    /// use std::mem::MaybeUninit;
    /// # use platform_mem::{Alloc, RawMem};
    ///
    /// let mut alloc = Alloc::new(Global);
    /// unsafe {
    ///     alloc.grow(10, |_uninit: &mut [MaybeUninit<u64>]| {
    ///         // `RawMem` relies on the fact that we initialize memory
    ///         // even if they are primitives
    ///     }).unwrap();
    /// }
    /// ```
    unsafe fn grow(
        &mut self,
        cap: usize,
        fill: impl FnOnce(&mut [MaybeUninit<Self::Item>]),
    ) -> Result<&mut [Self::Item]>;

    fn grow_filled(&mut self, cap: usize, value: Self::Item) -> Result<&mut [Self::Item]>
    where
        Self::Item: Clone,
    {
        fn uninit_fill<T: Clone>(uninit: &mut [MaybeUninit<T>], val: T) {
            if let Some((last, elems)) = uninit.split_last_mut() {
                for el in elems.iter_mut() {
                    el.write(val.clone());
                }
                last.write(val);
            }
        }

        unsafe {
            self.grow(cap, |uninit| {
                uninit_fill(uninit, value);
            })
        }
    }

    /// Attempts to shrink the memory block.
    ///
    /// # Errors
    ///
    /// Returns error if the `allocated - capacity` overflowing
    fn shrink(&mut self, cap: usize) -> Result<()>;
}
