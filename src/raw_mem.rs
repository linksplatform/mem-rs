use {
    mem::size_of,
    std::{
        alloc::Layout,
        mem::{self, MaybeUninit},
        ptr,
    },
};

/// Error memory allocation
// fixme: maybe we should add `(X bytes)` after `cannot allocate/occupy`
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]

struct Guard<'a, T> {
    slice: &'a mut [MaybeUninit<T>],
    init: usize,
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        // SAFETY: this raw slice will contain only initialized objects
        // that's why, it is allowed to drop it.
        unsafe {
            ptr::drop_in_place(MaybeUninit::slice_assume_init_mut(&mut self.slice[..self.init]));
        }
    }
}

pub enum Error {
    /// Error due to the computed capacity exceeding the maximum
    /// (usually `isize::MAX` bytes).
    ///
    /// ## Examples
    ///
    /// grow more than `isize::MAX` bytes:
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #![feature(assert_matches)]
    /// # use std::alloc::Global;
    /// # use std::assert_matches::assert_matches;
    /// # use platform_mem::{Error, Alloc, RawMem};
    /// let mut mem = Alloc::new(Global);
    /// assert_matches!(mem.grow_filled(usize::MAX, 0u64), Err(Error::CapacityOverflow));
    /// ```
    #[error("exceeding the capacity maximum")]
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

pub trait RawMem<T> {
    type Item;

    fn allocated(&self) -> &[Self::Item];
    fn allocated_mut(&mut self) -> &mut [Self::Item];

    /// # Safety
    /// Caller must guarantee that `fill` makes the uninitialized part valid for
    /// [`MaybeUninit::slice_assume_init_mut`]
    ///
    /// ### Incorrect usage
    /// ```no_run
    /// # #![feature(allocator_api)]
    /// # use std::alloc::Global;
    /// # use std::mem::MaybeUninit;
    /// # use platform_mem::{Alloc, RawMem};
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

    /// [`grow`] which assumes that the memory is already initialized
    ///
    /// # Safety
    ///
    /// When calling this method, you have to ensure that one of the following is true:
    ///
    /// * memory already initialized as [`Item`]
    ///
    /// * memory is initialized bytes and [`Item`] can be represented as bytes
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use platform_mem::{FileMapped, RawMem, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let mut file = FileMapped::from_path("..")?;
    ///     // file is always represents as initialized bytes
    ///     // and usize is transparent as bytes
    ///     let _: &mut [usize] = unsafe { file.grow_assumed(10)? };
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [`grow`]: Self::grow
    /// [`Item`]: Self::Item
    unsafe fn grow_assumed(&mut self, cap: usize) -> Result<&mut [Self::Item]> {
        self.grow(cap, |_| {})
    }

    fn grow_with(
        &mut self,
        addition: usize,
        f: impl FnMut() -> Self::Item,
    ) -> Result<&mut [Self::Item]> {
        fn inner<T>(uninit: &mut [MaybeUninit<T>], mut f: impl FnMut() -> T) {
            let guard = Guard { slice: uninit, init: 0 };
            for el in guard.slice.iter_mut() {
                el.write(f());
            }
            mem::forget(guard);
        }
        unsafe {
            self.grow(addition, |uninit| {
                inner(uninit, f);
            })
        }
    }

    unsafe fn grow_zeroed(
        &mut self,
        cap: usize,
        fill: impl FnOnce(&mut [MaybeUninit<Self::Item>]),
    ) -> Result<&mut [Self::Item]> {
        self.grow(cap, |uninit| {
            ptr::write_bytes(uninit.as_mut_ptr() as *mut u8, 0, uninit.len() * size_of::<T>());
            fill(uninit);
        })
    }

    fn grow_filled(&mut self, cap: usize, value: Self::Item) -> Result<&mut [Self::Item]>
    where
        Self::Item: Clone,
    {
        fn uninit_fill<T: Clone>(uninit: &mut [MaybeUninit<T>], val: T) {
            let mut guard = Guard { slice: uninit, init: 0 };

            if let Some((last, elems)) = guard.slice.split_last_mut() {
                for el in elems.iter_mut() {
                    el.write(val.clone());
                    guard.init += 1;
                }
                last.write(val);
                guard.init += 1;
            }

            mem::forget(guard);
        }

        unsafe {
            self.grow(cap, |uninit| {
                uninit_fill(uninit, value);
            })
        }
    }

    fn shrink(&mut self, cap: usize) -> Result<()>;
}
