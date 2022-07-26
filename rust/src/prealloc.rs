use crate::{Error, RawMem, Result};
use std::{borrow::BorrowMut, marker::PhantomData, ops::Deref};
use tap::TapOptional;

/// [`RawMem`] that own any type that provides refs to memory block
/// (<code>[`AsMut<[T]>`] + [`AsRef<[T]>`]</code>)
pub struct PreAlloc<T, D> {
    data: D,
    allocated: usize,
    // unlike other implementations dropck escape-hatch store in `D`
    // but `T` is unused :)
    marker: PhantomData<T>,
}

impl<T, D> PreAlloc<T, D> {
    /// Constructs new `PreAlloc`
    pub const fn new(data: D) -> Self {
        Self {
            data,
            allocated: 0,
            marker: PhantomData,
        }
    }
}

impl<T, D: AsMut<[T]> + AsRef<[T]>> RawMem<T> for PreAlloc<T, D> {
    fn alloc(&mut self, capacity: usize) -> Result<&mut [T]> {
        let slice = self.data.as_mut();
        let available = slice.len();
        slice
            .get_mut(0..capacity)
            // equivalent `Some::inspect` but stable and has more logic name than `inspect`
            .tap_some(|_| {
                // set `allocated` if data is valid
                self.allocated = capacity;
            })
            .ok_or(Error::OverAlloc {
                available,
                to_alloc: capacity,
            })
    }

    fn allocated(&self) -> usize {
        self.allocated
    }

    fn size_hint(&self) -> usize {
        self.data.as_ref().len()
    }
}
