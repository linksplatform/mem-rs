use {
    crate::{
        debug_mem,
        Error::{AllocError, CapacityOverflow},
        RawMem, RawPlace, Result,
    },
    std::{
        alloc::{Allocator, Layout},
        fmt::{self, Debug, Formatter},
        mem::{ManuallyDrop, MaybeUninit},
        ptr::NonNull,
        slice,
    },
};

pub struct Alloc<T, A: Allocator> {
    buf: ManuallyDrop<RawPlace<T>>,
    alloc: A,
}

impl<T, A: Allocator> Alloc<T, A> {
    pub const fn new(alloc: A) -> Self {
        Self { buf: ManuallyDrop::new(RawPlace::dangling()), alloc }
    }

    fn current_memory(&self) -> Option<(NonNull<u8>, Layout)> {
        // SAFETY: we would use `Layout::array`, but memory is allocated yet
        // and it's size+align is always valid (because we already alloc it by `Layout::array`)
        unsafe { RawPlace::current_memory(self.buf.ptr, self.buf.cap) }
    }
}

impl<T, A: Allocator> RawMem for Alloc<T, A> {
    type Item = T;

    fn allocated(&self) -> &[Self::Item] {
        unsafe { slice::from_raw_parts(self.buf.ptr.as_ptr(), self.buf.cap) }
    }

    fn allocated_mut(&mut self) -> &mut [Self::Item] {
        unsafe { slice::from_raw_parts_mut(self.buf.ptr.as_ptr(), self.buf.cap) }
    }

    unsafe fn grow(
        &mut self,
        addition: usize,
        fill: impl FnOnce(&mut [MaybeUninit<Self::Item>]),
    ) -> Result<&mut [Self::Item]> {
        let cap = self.buf.cap.checked_add(addition).ok_or(CapacityOverflow)?;
        let new_layout = Layout::array::<T>(cap).map_err(|_| CapacityOverflow)?;

        let ptr = if let Some((ptr, old_layout)) = self.current_memory() {
            self.alloc.grow(ptr, old_layout, new_layout)
        } else {
            self.alloc.allocate(new_layout)
        }
        .map_err(|_| AllocError { layout: new_layout, non_exhaustive: () })?
        .cast();

        Ok(self.buf.handle_fill(ptr, cap, fill))
    }

    fn shrink(&mut self, _: usize) -> Result<()> {
        todo!()
    }
}

impl<T, A: Allocator + Debug> Debug for Alloc<T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        debug_mem(f, &self.buf, "Alloc")?.field("alloc", &self.alloc).finish()
    }
}

impl<T, A: Allocator> Drop for Alloc<T, A> {
    fn drop(&mut self) {
        if let Some((ptr, layout)) = self.current_memory() {
            unsafe {
                // we should to drop this before `Self` because it is like `RawVec`, but in reverse
                // `RawPlace` - drop memory
                // `Self` - deallocate memory
                ManuallyDrop::drop(&mut self.buf);

                self.alloc.deallocate(ptr, layout);
            }
        }
    }
}
