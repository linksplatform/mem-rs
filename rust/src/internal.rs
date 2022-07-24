use std::{mem::size_of, ptr::NonNull};

pub fn to_bytes<T>(ptr: NonNull<[T]>) -> NonNull<[u8]> {
    NonNull::slice_from_raw_parts(ptr.as_non_null_ptr().cast(), ptr.len() * size_of::<T>())
}

pub fn guaranteed_from_bytes<U>(ptr: NonNull<[u8]>) -> NonNull<[U]> {
    debug_assert!(
        ptr.len() % size_of::<U>() == 0,
        "Types are not aligned; len: {}, size_of: {}",
        ptr.len(),
        size_of::<U>()
    );

    NonNull::slice_from_raw_parts(ptr.as_non_null_ptr().cast(), ptr.len() / size_of::<U>())
}

// for more explicit unsafe zones
#[allow(unused_unsafe)]
pub unsafe fn from_bytes_slice<U>(bytes: &mut [u8]) -> &mut [U] {
    debug_assert!(bytes.len() % size_of::<U>() == 0, "Types are not aligned");

    // SAFETY: Caller must guarantee that transmute<u8, U> no has side effects.
    let (a, slice, b) = unsafe { bytes.align_to_mut() };
    assert!(a.is_empty());
    assert!(b.is_empty());
    slice
}

// UNSAFE
// wrapper for `.pipe` function
pub(crate) fn safety_from_bytes_slice<U>(bytes: &mut [u8]) -> &mut [U] {
    // SAFETY: the safety contract for `self::from_bytes_slice` must
    // be upheld by the caller.
    unsafe { from_bytes_slice::<U>(bytes) }
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

        let new_ptr: NonNull<_> = guaranteed_from_bytes(to_bytes(ptr));

        let new_slice = unsafe { ptr.as_ref() };
        ptr == new_ptr && slice == new_slice
    }

    #[quickcheck]
    fn align_slice(mut data: Vec<u8>) -> bool {
        let cloned = data.clone();
        let slice = data.as_mut_slice();

        let new_slice: &[u8] = unsafe { from_bytes_slice(slice) };
        new_slice == cloned.as_slice()
    }
}
