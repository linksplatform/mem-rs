use std::{
    alloc::Layout,
    mem::{self, MaybeUninit},
    ptr,
};

/// Error memory allocation
// fixme: maybe we should add `(X bytes)` after `cannot allocate/occupy`
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
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

struct Guard<'a, T> {
    slice: &'a mut [MaybeUninit<T>],
    init: usize,
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        debug_assert!(self.init <= self.slice.len());
        // SAFETY: this raw slice will contain only initialized objects
        // that's why, it is allowed to drop it.
        unsafe {
            ptr::drop_in_place(MaybeUninit::slice_assume_init_mut(
                self.slice.get_unchecked_mut(..self.init),
            ));
        }
    }
}

pub trait RawMem {
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
    /// # use platform_mem::Result;
    /// use platform_mem::{Alloc, RawMem};
    ///
    /// let mut alloc = Alloc::new(Global);
    /// unsafe {
    ///     alloc.grow(10, |_uninit: &mut [MaybeUninit<u64>]| {
    ///         // `RawMem` relies on the fact that we initialize memory
    ///         // even if they are primitives
    ///     })?;
    /// }
    /// # Result::Ok(())
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
    /// # use platform_mem::Result;
    /// use platform_mem::{FileMapped, RawMem};
    ///
    /// let mut file = FileMapped::from_path("..")?;
    /// // file is always represents as initialized bytes
    /// // and usize is transparent as bytes
    /// let _: &mut [usize] = unsafe { file.grow_assumed(10)? };
    /// # Result::Ok(())
    /// ```
    ///
    /// [`grow`]: Self::grow
    /// [`Item`]: Self::Item
    unsafe fn grow_assumed(&mut self, cap: usize) -> Result<&mut [Self::Item]> {
        self.grow(cap, |_| {})
    }

    /// # Safety
    /// [`Item`] must satisfy [initialization invariant][inv] for [`mem::zeroed`]
    ///
    /// [`Item`]: Self::Item
    ///  [inv]: MaybeUninit#initialization-invariant
    ///
    /// # Examples
    /// Correct usage of this function: initializing an integral-like types with zeroes:
    /// ```
    /// # #![feature(allocator_api)]
    /// # use platform_mem::Error;
    /// use platform_mem::{Global, RawMem};
    ///
    /// let mut alloc = Global::new();
    /// let zeroes: &mut [(u8, u16)] = unsafe {
    ///     alloc.grow_zeroed(10)?
    /// };
    ///
    /// assert_eq!(zeroes, [(0, 0); 10]);
    /// # Ok::<_, Error>(())
    /// ```
    ///
    /// Incorrect usage of this function: initializing a reference with zero:
    /// ```no_run
    /// # #![feature(allocator_api)]
    ///  # use platform_mem::Error;
    /// use platform_mem::{Global, RawMem};
    ///
    /// let mut alloc = Global::new();
    /// let _: &mut [&'static str] = unsafe {
    ///     alloc.grow_zeroed(10)? // Undefined behavior!
    /// };
    ///
    /// # Ok::<_, Error>(())
    /// ```
    ///
    unsafe fn grow_zeroed(&mut self, cap: usize) -> Result<&mut [Self::Item]> {
        self.grow(cap, |uninit| {
            uninit.as_mut_ptr().write_bytes(0u8, uninit.len());
        })
    }
    /// Fill the memory with the result of the closure.
    ///
    /// # Examples
    /// Correct usage of this function:
    /// ```
    /// # #![feature(allocator_api)]
    /// use platform_mem::{Global, RawMem};
    /// let mut alloc = Global::new();
    /// let res: &mut [(u8, u16)] = unsafe {
    ///    alloc.grow_with(10, || {
    ///       (2, 2)
    ///   })?
    /// };
    /// assert_eq!(res, [(2, 2); 10]);
    /// # Ok::<_, Error>(())
    /// ```
    fn grow_with(
        &mut self,
        addition: usize,
        f: impl FnMut() -> Self::Item,
    ) -> Result<&mut [Self::Item]> {
        fn inner<T>(uninit: &mut [MaybeUninit<T>], mut fill: impl FnMut() -> T) {
            let mut guard = Guard { slice: uninit, init: 0 };

            for el in guard.slice.iter_mut() {
                el.write(fill());
                guard.init += 1;
            }

            mem::forget(guard);
        }
        unsafe {
            self.grow(addition, |uninit| {
                inner(uninit, f);
            })
        }
    }
    /// Fills initialized memory with a given value.
    /// # Examples
    /// Correct usage of this function:
    /// ```
    /// # #![feature(allocator_api)]
    /// use platform_mem::{Global, RawMem};
    /// let mut alloc = Global::new();
    /// let res: &mut [(u8, u16)] = unsafe {
    ///   alloc.grow_filled(10, (2, 2))?
    /// };
    /// assert_eq!(res, [(2, 2); 10]);
    /// # Ok::<_, Error>(())
    /// ```
    fn grow_filled(&mut self, cap: usize, value: Self::Item) -> Result<&mut [Self::Item]>
    where
        Self::Item: Clone,
    {
        trait SpecFill<T> {
            fn fill(&mut self, val: T);
        }

        impl<T: Clone> SpecFill<T> for [MaybeUninit<T>] {
            default fn fill(&mut self, val: T) {
                let mut guard = Guard { slice: self, init: 0 };

                if let Some((last, elems)) = guard.slice.split_last_mut() {
                    for el in elems {
                        el.write(val.clone());
                        guard.init += 1;
                    }
                    last.write(val);
                    guard.init += 1;
                }

                mem::forget(guard);
            }
        }

        impl<T: Copy> SpecFill<T> for [MaybeUninit<T>] {
            fn fill(&mut self, val: T) {
                for item in self {
                    item.write(val);
                }
            }
        }

        fn uninit_fill<T: Clone>(uninit: &mut [MaybeUninit<T>], val: T) {
            SpecFill::fill(uninit, val);
        }

        unsafe {
            self.grow(cap, |uninit| {
                uninit_fill(uninit, value);
            })
        }
    }
    /// Shrinks the capacity of the memory to fit its length.
    /// # Examples
    /// ```
    /// # #![feature(allocator_api)]
    /// use platform_mem::{Global, RawMem};
    /// let mut alloc = Global::new();
    /// let mut res: &mut [(u8, u16)] = unsafe {
    ///  alloc.grow(10, || {
    ///    (2, 2)
    /// })?
    /// };
    /// res.shrink(5)?;
    /// assert_eq!(res, [(2, 2); 5]);
    /// # Ok::<_, Error>(())
    /// ```
    fn shrink(&mut self, cap: usize) -> Result<()>;
}
