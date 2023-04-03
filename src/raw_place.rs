use std::{
    alloc::Layout,
    fmt::{self, Formatter},
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr::NonNull,
    slice,
};

pub struct RawPlace<T> {
    pub ptr: NonNull<T>,
    len: usize,     // use to drop at panic
    pub cap: usize, // usually `cap` is same `len`
    _marker: PhantomData<T>,
}

impl<T> RawPlace<T> {
    pub const fn dangling() -> Self {
        Self { ptr: NonNull::dangling(), len: 0, cap: 0, _marker: PhantomData }
    }

    pub unsafe fn as_slice(&self) -> &[T] {
        slice::from_raw_parts(self.ptr.as_ptr(), self.len)
    }

    pub unsafe fn as_slice_mut(&mut self) -> &mut [T] {
        slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
    }

    /// # Safety
    /// `RawPlace` must contain valid `ptr` (aligned) and `cap` (valid for `Layout`)
    pub unsafe fn current_memory(&self) -> Option<(NonNull<u8>, Layout)> {
        if self.cap == 0 {
            None
        } else {
            let layout = Layout::from_size_align_unchecked(
                mem::size_of::<T>().unchecked_mul(self.cap),
                mem::align_of::<T>(),
            );
            Some((self.ptr.cast(), layout))
        }
    }

    pub unsafe fn handle_fill(
        &mut self,
        ptr: NonNull<T>,
        cap: usize,
        fill: impl FnOnce(&mut [MaybeUninit<T>]),
    ) -> &mut [T] {
        let uninit = NonNull::slice_from_raw_parts(ptr, cap)
            .get_unchecked_mut(self.cap..)
            .as_uninit_slice_mut();

        self.ptr = ptr;
        self.cap = cap; // `ptr` and `cap` changes after panicking `fill`
        //                 ( alloc memory )

        fill(uninit); // panic out!

        self.len = cap; // `len` is same `cap` only if `uninit` was init

        MaybeUninit::slice_assume_init_mut(uninit)
    }
}

impl<T> fmt::Debug for RawPlace<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}..{})", self.ptr, self.cap)
    }
}

unsafe impl<T: Sync> Sync for RawPlace<T> {}
unsafe impl<T: Send> Send for RawPlace<T> {}