use std::{
    alloc::Layout,
    fmt::{self, Formatter},
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
    slice,
};

pub struct RawPlace<T> {
    pub ptr: NonNull<T>,
    pub cap: usize,
    _marker: PhantomData<T>,
}

impl<T> RawPlace<T> {
    pub const fn dangling() -> Self {
        Self { ptr: NonNull::dangling(), cap: 0, _marker: PhantomData }
    }

    pub unsafe fn as_ref(&self) -> &[T] {
        slice::from_raw_parts(self.ptr.as_ptr(), self.cap)
    }

    pub unsafe fn as_mut(&mut self) -> &mut [T] {
        slice::from_raw_parts_mut(self.ptr.as_ptr(), self.cap)
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

        self.ptr = ptr; // guard will has same ptr but old capacity

        // use `self` as guard and later replace it back
        // `mem::take` may be misleading
        let guard = mem::replace(self, Self::dangling());

        fill(uninit); // panic out!

        // underscore exactly got dangling guard
        // it's `Drop`does nothing
        let _ = mem::replace(self, guard);
        self.cap = cap; // set new capacity only after possible `drop_in_place` with old capacity

        MaybeUninit::slice_assume_init_mut(uninit)
    }
}

impl<T> fmt::Debug for RawPlace<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}..{})", self.ptr, self.cap)
    }
}

impl<T> Drop for RawPlace<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), self.cap));
        }
    }
}

unsafe impl<T: Sync> Sync for RawPlace<T> {}
unsafe impl<T: Send> Send for RawPlace<T> {}
