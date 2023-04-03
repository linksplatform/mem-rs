use {
    crate::{
        debug_mem,
        Error::{AllocError, CapacityOverflow},
        RawMem, RawPlace, Result,
    },
    std::{
        alloc::{Allocator, Layout},
        fmt::{self, Debug, Formatter},
        mem::MaybeUninit,
        ptr,
    },
};

pub struct Alloc<T, A: Allocator> {
    buf: RawPlace<T>,
    alloc: A,
}

impl<T, A: Allocator> Alloc<T, A> {
    pub const fn new(alloc: A) -> Self {
        Self { buf: RawPlace::dangling(), alloc }
    }
}

impl<T, A: Allocator> RawMem for Alloc<T, A> {
    type Item = T;

    fn allocated(&self) -> &[Self::Item] {
        unsafe { self.buf.as_slice() }
    }

    fn allocated_mut(&mut self) -> &mut [Self::Item] {
        unsafe { self.buf.as_slice_mut() }
    }

    unsafe fn grow(
        &mut self,
        addition: usize,
        fill: impl FnOnce(&mut [MaybeUninit<Self::Item>]),
    ) -> Result<&mut [Self::Item]> {
        let cap = self.buf.cap().checked_add(addition).ok_or(CapacityOverflow)?;
        let new_layout = Layout::array::<T>(cap).map_err(|_| CapacityOverflow)?;

        let ptr = if let Some((ptr, old_layout)) = self.buf.current_memory() {
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
        unsafe {
            if let Some((ptr, layout)) = self.buf.current_memory() {
                ptr::drop_in_place(self.buf.as_slice_mut());
                self.alloc.deallocate(ptr, layout);
            }
        }
    }
}

// fixme: move into `lib.rs` for all `RawMem` implementors (or remove it as useless)
fn _assert() {
    use std::alloc::Global;

    fn assert_sync_send<T: Sync + Send>() {}

    assert_sync_send::<Alloc<(), Global>>();
}
