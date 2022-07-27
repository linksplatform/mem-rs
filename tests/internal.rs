#[allow(dead_code)]
pub fn pre_allocated<T: Default>(len: usize) -> Vec<T> {
    (0..len).map(|_| T::default()).collect()
}

#[macro_export]
macro_rules! test_for_all_mem {
    ($test:ident, $impl:ident) => {
        paste::paste! {
            #[test]
            fn [<$test _global>]() { $impl(platform_mem::Global::new()).unwrap(); }

            #[test]
            fn [<$test _alloc>]() { $impl(platform_mem::Alloc::new(std::alloc::Global)).unwrap(); }

            // also test `FileMapped`
            #[test]
            #[cfg(not(miri))]
            fn [<$test _temp_file>]() { $impl(platform_mem::TempFile::new().unwrap()).unwrap(); }

            #[test]
            fn [<$test _pre_alloc>]() { $impl(platform_mem::PreAlloc::new(internal::pre_allocated(100))).unwrap(); }
        }
    };
}
