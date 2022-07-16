use std::mem::size_of;
use std::ptr::NonNull;

pub fn align_from<T>(ptr: NonNull<[T]>) -> NonNull<[u8]> {
    NonNull::slice_from_raw_parts(ptr.as_non_null_ptr().cast(), ptr.len() * size_of::<T>())
}

pub fn guaranteed_align_to<U>(ptr: NonNull<[u8]>) -> NonNull<[U]> {
    debug_assert!(ptr.len() % size_of::<U>() == 0, "Types are not aligned");

    NonNull::slice_from_raw_parts(ptr.as_non_null_ptr().cast(), ptr.len() / size_of::<U>())
}

pub fn guaranteed_align_slice<U>(bytes: &mut [u8]) -> &mut [U] {
    debug_assert!(bytes.len() % size_of::<U>() == 0, "Types are not aligned");

    // SAFETY: `U` is safe to `transmute` from `T`
    let (a, slice, b) = unsafe { bytes.align_to_mut() };
    debug_assert!(a.is_empty());
    debug_assert!(b.is_empty());
    slice
}
