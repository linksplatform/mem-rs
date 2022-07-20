use std::{mem::size_of, ptr::NonNull};

pub fn align_from<T>(ptr: NonNull<[T]>) -> NonNull<[u8]> {
    NonNull::slice_from_raw_parts(ptr.as_non_null_ptr().cast(), ptr.len() * size_of::<T>())
}

pub fn guaranteed_align_to<U>(ptr: NonNull<[u8]>) -> NonNull<[U]> {
    let size_of = size_of::<U>();
    if size_of != 0 {
        debug_assert!(ptr.len() % size_of == 0, "Types are not aligned");
    }

    NonNull::slice_from_raw_parts(
        ptr.as_non_null_ptr().cast(),
        ptr.len() / if size_of == 0 { 1 } else { size_of },
    )
}

// for more explicit unsafe zones
#[allow(unused_unsafe)]
pub unsafe fn guaranteed_align_slice<U>(bytes: &mut [u8]) -> &mut [U] {
    let size_of = size_of::<U>();
    if size_of != 0 {
        debug_assert!(bytes.len() % size_of == 0, "Types are not aligned");
    }

    // SAFETY: Caller must guarantee that transmute<u8, U> no has side effects.
    let (a, slice, b) = unsafe { bytes.align_to_mut() };
    assert!(a.is_empty());
    assert!(b.is_empty());
    slice
}

// to constraint `RawMem` implementations
pub trait IsTrue<const COND: bool> {}

impl IsTrue<true> for () {}

#[cfg(test)]
mod quick_tests {
    use super::*;
    use quickcheck_macros::quickcheck;
    use std::ptr::NonNull;

    #[quickcheck]
    fn align_to_from(data: Vec<usize>) -> bool {
        let slice = data.as_slice();
        let ptr = NonNull::from(slice);

        let new_ptr: NonNull<_> = guaranteed_align_to(align_from(ptr));

        let new_slice = unsafe { ptr.as_ref() };
        ptr == new_ptr && slice == new_slice
    }

    #[quickcheck]
    fn align_slice(mut data: Vec<u8>) -> bool {
        let cloned = data.clone();
        let slice = data.as_mut_slice();

        let new_slice: &[u8] = unsafe { guaranteed_align_slice(slice) };
        new_slice == cloned.as_slice()
    }
}
