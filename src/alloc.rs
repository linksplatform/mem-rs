use crate::{Error::CapacityOverflow, RawMem, Result};
use std::{
    alloc::{Allocator, Layout, System},
    marker::PhantomData,
    mem,
    mem::MaybeUninit,
    ptr,
    ptr::NonNull,
    slice,
};
//use tap::Pipe;

pub struct Alloc<T, A: Allocator> {
    ptr: NonNull<T>,
    len: usize,
    alloc: A,
    _marker: PhantomData<T>,
}

impl<T, A: Allocator> Alloc<T, A> {
    pub const fn new(alloc: A) -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            alloc,
            _marker: PhantomData,
        }
    }

    fn current_memory(&self) -> Option<(NonNull<u8>, Layout)> {
        if self.len == 0 {
            None
        } else {
            // SAFETY: we would use `Layout::array`, but memory is allocated yet
            // and it's size+align is always valid (because we already alloc it by `Layout::array`)
            unsafe {
                let layout = Layout::from_size_align_unchecked(
                    mem::size_of::<T>().unchecked_mul(self.len),
                    mem::align_of::<T>(),
                );
                Some((self.ptr.cast(), layout))
            }
        }
    }
}

impl<T, A: Allocator> RawMem for Alloc<T, A> {
    type Item = T;

    unsafe fn grow(
        &mut self,
        cap: usize,
        fill: impl FnOnce(&mut [MaybeUninit<Self::Item>]),
    ) -> Result<&mut [Self::Item]> {
        let req_cap = self.len.checked_add(cap).ok_or(CapacityOverflow)?;
        let new_layout = Layout::array::<T>(req_cap)?;
        let ptr = if let Some((ptr, old_layout)) = self.current_memory() {
            self.alloc.grow(ptr, old_layout, new_layout)?
        } else {
            self.alloc.allocate(new_layout)?
        };
        let uninit = NonNull::slice_from_raw_parts(ptr.cast::<T>(), req_cap)
            .get_unchecked_mut(self.len..)
            .as_uninit_slice_mut();
        fill(uninit);
        self.ptr = ptr.cast::<T>();
        self.len = req_cap;
        Ok(MaybeUninit::slice_assume_init_mut(uninit))
    }

    fn shrink(&mut self, cap: usize) -> Result<()> {
        todo!()
    }
}

impl<T, A: Allocator> Drop for Alloc<T, A> {}

unsafe impl<T: Sync, A: Allocator + Sync> Sync for Alloc<T, A> {}
unsafe impl<T: Send, A: Allocator + Send> Send for Alloc<T, A> {}
