//! Keep exact zeroing bounded to the uninitialized tail without risking an
//! out-of-allocation write when testing the regression.

use platform_mem::{RawMem, Result, raw_mem::uninit};
use std::mem::MaybeUninit;

struct GuardedMem {
    // The last three elements are sentinels in the same allocation, outside the
    // six-element region passed to the initializer.
    data: [MaybeUninit<u32>; 9],
    len: usize,
}

impl RawMem for GuardedMem {
    type Item = u32;

    fn allocated(&self) -> &[u32] {
        // SAFETY: every element is initialized at construction and u32 has no
        // invalid bit patterns.
        unsafe { std::slice::from_raw_parts(self.data.as_ptr().cast(), self.len) }
    }

    fn allocated_mut(&mut self) -> &mut [u32] {
        // SAFETY: every element is initialized at construction.
        unsafe { uninit::assume_init_mut(&mut self.data[..self.len]) }
    }

    unsafe fn grow(
        &mut self,
        cap: usize,
        fill: impl FnOnce(usize, (&mut [u32], &mut [MaybeUninit<u32>])),
    ) -> Result<&mut [u32]> {
        assert_eq!(self.len, 0);
        assert_eq!(cap, 6);
        fill(3, (&mut [], &mut self.data[..cap]));
        self.len = cap;
        Ok(self.allocated_mut())
    }

    fn shrink(&mut self, _: usize) -> Result<()> {
        unimplemented!("this fixture only tests a single growth operation")
    }
}

#[test]
fn grow_zeroed_exact_keeps_the_initialized_prefix_and_guard() -> Result<()> {
    let mut mem = GuardedMem { data: [MaybeUninit::new(42); 9], len: 0 };
    // SAFETY: u32 permits zero initialization. The backing allocation includes
    // enough sentinel space to contain the historical oversized write.
    let grown = unsafe { mem.grow_zeroed_exact(6)? };
    assert_eq!(grown, &[42, 42, 42, 0, 0, 0]);
    // SAFETY: all elements in the fixture are initialized.
    let data = unsafe { uninit::assume_init_mut(&mut mem.data) };
    assert_eq!(&data[6..], &[42; 3]);
    Ok(())
}
