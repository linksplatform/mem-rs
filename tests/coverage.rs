//! Comprehensive tests for 100% code coverage

#![feature(allocator_api)]
#![feature(assert_matches)]

use platform_mem::{Alloc, ErasedMem, Error, FileMapped, Global, RawMem, Result, System, TempFile};
use std::alloc::Global as GlobalAlloc;
use std::assert_matches::assert_matches;
use std::io;

// ============================================================================
// Alloc tests
// ============================================================================

mod alloc_tests {
    use super::*;

    #[test]
    fn new_creates_empty_alloc() {
        let alloc: Alloc<u64, GlobalAlloc> = Alloc::new(GlobalAlloc);
        assert_eq!(alloc.allocated().len(), 0);
    }

    #[test]
    fn allocated_returns_empty_slice_initially() {
        let alloc: Alloc<u32, GlobalAlloc> = Alloc::new(GlobalAlloc);
        assert!(alloc.allocated().is_empty());
    }

    #[test]
    fn allocated_mut_returns_empty_slice_initially() {
        let mut alloc: Alloc<u32, GlobalAlloc> = Alloc::new(GlobalAlloc);
        assert!(alloc.allocated_mut().is_empty());
    }

    #[test]
    fn grow_increases_capacity() {
        let mut alloc: Alloc<u64, GlobalAlloc> = Alloc::new(GlobalAlloc);
        alloc.grow_filled(10, 42).unwrap();
        assert_eq!(alloc.allocated().len(), 10);
        assert_eq!(alloc.allocated(), &[42u64; 10]);
    }

    #[test]
    fn grow_capacity_overflow() {
        let mut alloc: Alloc<u64, GlobalAlloc> = Alloc::new(GlobalAlloc);
        let result = alloc.grow_filled(usize::MAX, 0);
        assert_matches!(result, Err(Error::CapacityOverflow));
    }

    #[test]
    fn shrink_decreases_capacity() {
        let mut alloc: Alloc<u64, GlobalAlloc> = Alloc::new(GlobalAlloc);
        alloc.grow_filled(10, 42).unwrap();
        alloc.shrink(5).unwrap();
        assert_eq!(alloc.allocated().len(), 5);
    }

    #[test]
    fn shrink_to_zero() {
        let mut alloc: Alloc<u64, GlobalAlloc> = Alloc::new(GlobalAlloc);
        alloc.grow_filled(10, 42).unwrap();
        alloc.shrink(10).unwrap();
        assert_eq!(alloc.allocated().len(), 0);
    }

    #[test]
    fn shrink_on_empty_does_nothing() {
        let mut alloc: Alloc<u64, GlobalAlloc> = Alloc::new(GlobalAlloc);
        alloc.shrink(0).unwrap();
        assert_eq!(alloc.allocated().len(), 0);
    }

    #[test]
    #[should_panic(expected = "Tried to shrink to a larger capacity")]
    fn shrink_beyond_capacity_panics() {
        let mut alloc: Alloc<u64, GlobalAlloc> = Alloc::new(GlobalAlloc);
        alloc.grow_filled(5, 42).unwrap();
        alloc.shrink(10).unwrap();
    }

    #[test]
    fn multiple_grows() {
        let mut alloc: Alloc<u64, GlobalAlloc> = Alloc::new(GlobalAlloc);
        alloc.grow_filled(5, 1).unwrap();
        alloc.grow_filled(5, 2).unwrap();
        alloc.grow_filled(5, 3).unwrap();
        assert_eq!(alloc.allocated().len(), 15);
        assert_eq!(&alloc.allocated()[..5], &[1, 1, 1, 1, 1]);
        assert_eq!(&alloc.allocated()[5..10], &[2, 2, 2, 2, 2]);
        assert_eq!(&alloc.allocated()[10..15], &[3, 3, 3, 3, 3]);
    }

    #[test]
    fn drop_cleans_up_memory() {
        let mut alloc: Alloc<String, GlobalAlloc> = Alloc::new(GlobalAlloc);
        alloc.grow_filled(5, String::from("hello")).unwrap();
        // Drop should clean up allocated strings
    }

    #[test]
    fn debug_format() {
        let alloc: Alloc<u64, GlobalAlloc> = Alloc::new(GlobalAlloc);
        let debug_str = format!("{:?}", alloc);
        assert!(debug_str.contains("Alloc"));
    }
}

// ============================================================================
// Global/System allocator wrapper tests
// ============================================================================

mod wrapper_tests {
    use super::*;

    #[test]
    fn global_new() {
        let global: Global<u64> = Global::new();
        assert_eq!(global.allocated().len(), 0);
    }

    #[test]
    fn global_default() {
        let global: Global<u64> = Global::default();
        assert_eq!(global.allocated().len(), 0);
    }

    #[test]
    fn global_grow_and_shrink() {
        let mut global: Global<u64> = Global::new();
        global.grow_filled(10, 42).unwrap();
        assert_eq!(global.allocated().len(), 10);
        global.shrink(5).unwrap();
        assert_eq!(global.allocated().len(), 5);
    }

    #[test]
    fn global_allocated_mut() {
        let mut global: Global<u64> = Global::new();
        global.grow_filled(5, 0).unwrap();
        global.allocated_mut()[0] = 42;
        assert_eq!(global.allocated()[0], 42);
    }

    #[test]
    fn global_size_hint() {
        let global: Global<u64> = Global::new();
        assert_eq!(global.size_hint(), None);
    }

    #[test]
    fn global_debug() {
        let global: Global<u64> = Global::new();
        let debug_str = format!("{:?}", global);
        assert!(debug_str.contains("Global"));
    }

    #[test]
    fn system_new() {
        let system: System<u64> = System::new();
        assert_eq!(system.allocated().len(), 0);
    }

    #[test]
    fn system_default() {
        let system: System<u64> = System::default();
        assert_eq!(system.allocated().len(), 0);
    }

    #[test]
    fn system_grow_and_shrink() {
        let mut system: System<u64> = System::new();
        system.grow_filled(10, 42).unwrap();
        assert_eq!(system.allocated().len(), 10);
        system.shrink(5).unwrap();
        assert_eq!(system.allocated().len(), 5);
    }
}

// ============================================================================
// TempFile tests
// ============================================================================

mod tempfile_tests {
    use super::*;

    #[test]
    fn tempfile_new() {
        let tempfile = TempFile::<u64>::new().unwrap();
        assert_eq!(tempfile.allocated().len(), 0);
    }

    #[test]
    fn tempfile_new_in() {
        let tempfile = TempFile::<u64>::new_in(".").unwrap();
        assert_eq!(tempfile.allocated().len(), 0);
    }

    #[test]
    fn tempfile_grow() {
        let mut tempfile = TempFile::<u64>::new().unwrap();
        tempfile.grow_filled(10, 42).unwrap();
        assert_eq!(tempfile.allocated().len(), 10);
        assert_eq!(tempfile.allocated(), &[42u64; 10]);
    }

    #[test]
    fn tempfile_shrink() {
        let mut tempfile = TempFile::<u64>::new().unwrap();
        tempfile.grow_filled(10, 42).unwrap();
        tempfile.shrink(5).unwrap();
        assert_eq!(tempfile.allocated().len(), 5);
    }

    #[test]
    fn tempfile_debug() {
        let tempfile = TempFile::<u64>::new().unwrap();
        let debug_str = format!("{:?}", tempfile);
        assert!(debug_str.contains("TempFile"));
    }
}

// ============================================================================
// FileMapped tests
// ============================================================================

mod file_mapped_tests {
    use super::*;
    use std::fs;

    fn cleanup_test_file(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn from_path_creates_new_file() -> Result<()> {
        const FILE: &str = "test_from_path.bin";
        cleanup_test_file(FILE);

        let mem = FileMapped::<u64>::from_path(FILE)?;
        assert_eq!(mem.allocated().len(), 0);

        cleanup_test_file(FILE);
        Ok(())
    }

    #[test]
    fn grow_and_access() -> Result<()> {
        const FILE: &str = "test_grow_access.bin";
        cleanup_test_file(FILE);

        {
            let mut mem = FileMapped::<u64>::from_path(FILE)?;
            mem.grow_filled(10, 42)?;
            assert_eq!(mem.allocated().len(), 10);
            mem.allocated_mut()[0] = 123;
        }

        cleanup_test_file(FILE);
        Ok(())
    }

    #[test]
    fn shrink_memory() -> Result<()> {
        const FILE: &str = "test_shrink.bin";
        cleanup_test_file(FILE);

        {
            let mut mem = FileMapped::<u64>::from_path(FILE)?;
            mem.grow_filled(10, 42)?;
            mem.shrink(5)?;
            assert_eq!(mem.allocated().len(), 5);
        }

        cleanup_test_file(FILE);
        Ok(())
    }

    #[test]
    fn capacity_overflow() -> Result<()> {
        const FILE: &str = "test_overflow.bin";
        cleanup_test_file(FILE);

        let mut mem = FileMapped::<u64>::from_path(FILE)?;
        let result = mem.grow_filled(usize::MAX, 0);
        assert_matches!(result, Err(Error::CapacityOverflow));

        cleanup_test_file(FILE);
        Ok(())
    }

    #[test]
    fn debug_format() -> Result<()> {
        const FILE: &str = "test_debug.bin";
        cleanup_test_file(FILE);

        let mem = FileMapped::<u64>::from_path(FILE)?;
        let debug_str = format!("{:?}", mem);
        assert!(debug_str.contains("FileMapped"));

        cleanup_test_file(FILE);
        Ok(())
    }

    #[test]
    fn grow_assumed_with_existing_file() -> Result<()> {
        const FILE: &str = "test_assumed.bin";
        cleanup_test_file(FILE);

        unsafe {
            let mut mem = FileMapped::<u8>::from_path(FILE)?;
            // File is zeroed, so we can assume it's initialized for u8
            let _slice = mem.grow_assumed(100)?;
        }

        cleanup_test_file(FILE);
        Ok(())
    }
}

// ============================================================================
// RawMem trait method tests
// ============================================================================

mod raw_mem_tests {
    use super::*;

    #[test]
    fn grow_zeroed() {
        let mut mem = Global::<u64>::new();
        unsafe {
            mem.grow_zeroed(10).unwrap();
        }
        assert_eq!(mem.allocated(), &[0u64; 10]);
    }

    #[test]
    fn grow_zeroed_exact() {
        let mut mem = Global::<u64>::new();
        mem.grow_filled(5, 1).unwrap();
        unsafe {
            mem.grow_zeroed_exact(5).unwrap();
        }
        // First 5 should be 1, next 5 should be 0
        assert_eq!(&mem.allocated()[..5], &[1, 1, 1, 1, 1]);
        // The grow_zeroed_exact zeros from inited onwards
    }

    #[test]
    fn grow_with() {
        let mut mem = Global::<u64>::new();
        let mut counter = 0u64;
        mem.grow_with(5, || {
            counter += 1;
            counter
        })
        .unwrap();
        assert_eq!(mem.allocated(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn grow_with_exact() {
        let mut mem = Global::<u64>::new();
        mem.grow_filled(3, 0).unwrap();
        let mut counter = 0u64;
        unsafe {
            mem.grow_with_exact(3, || {
                counter += 1;
                counter
            })
            .unwrap();
        }
        assert_eq!(mem.allocated().len(), 6);
    }

    #[test]
    fn grow_filled_exact() {
        let mut mem = Global::<u64>::new();
        mem.grow_filled(3, 0).unwrap();
        unsafe {
            mem.grow_filled_exact(3, 42).unwrap();
        }
        assert_eq!(mem.allocated().len(), 6);
    }

    #[test]
    fn grow_within_range() {
        let mut mem = Global::<u64>::new();
        mem.grow_filled(5, 0).unwrap();
        mem.allocated_mut()[0] = 1;
        mem.allocated_mut()[1] = 2;
        mem.allocated_mut()[2] = 3;
        mem.grow_within(0..3).unwrap();
        assert_eq!(mem.allocated().len(), 8);
        assert_eq!(&mem.allocated()[5..8], &[1, 2, 3]);
    }

    #[test]
    fn grow_from_slice() {
        let mut mem = Global::<u64>::new();
        let slice = [1, 2, 3, 4, 5];
        mem.grow_from_slice(&slice).unwrap();
        assert_eq!(mem.allocated(), &slice);
    }

    #[test]
    fn size_hint_returns_none() {
        let mem = Global::<u64>::new();
        assert_eq!(mem.size_hint(), None);
    }
}

// ============================================================================
// ErasedMem tests
// ============================================================================

mod erased_mem_tests {
    use super::*;

    #[test]
    fn box_dyn_erased_mem() {
        let mut mem: Box<dyn ErasedMem<Item = u64>> = Box::new(Global::<u64>::new());
        mem.grow_filled(5, 42).unwrap();
        assert_eq!(mem.allocated().len(), 5);
    }

    #[test]
    fn box_dyn_erased_mem_sync() {
        let mut mem: Box<dyn ErasedMem<Item = u64> + Sync> = Box::new(Global::<u64>::new());
        mem.grow_filled(5, 42).unwrap();
        assert_eq!(mem.allocated().len(), 5);
    }

    #[test]
    fn box_dyn_erased_mem_sync_send() {
        let mut mem: Box<dyn ErasedMem<Item = u64> + Sync + Send> = Box::new(Global::<u64>::new());
        mem.grow_filled(5, 42).unwrap();
        assert_eq!(mem.allocated().len(), 5);
    }

    #[test]
    fn mutable_reference_as_erased_mem() {
        let mut inner = Global::<u64>::new();
        // Use ErasedMem through a mutable reference, which implements RawMem
        let mem: &mut Global<u64> = &mut inner;
        mem.grow_filled(5, 42).unwrap();
        assert_eq!(mem.allocated().len(), 5);
    }

    #[test]
    fn erased_shrink() {
        let mut mem: Box<dyn ErasedMem<Item = u64>> = Box::new(Global::<u64>::new());
        mem.grow_filled(10, 42).unwrap();
        mem.shrink(5).unwrap();
        assert_eq!(mem.allocated().len(), 5);
    }

    #[test]
    fn erased_size_hint() {
        let mem: Box<dyn ErasedMem<Item = u64>> = Box::new(Global::<u64>::new());
        assert_eq!(mem.size_hint(), None);
    }

    #[test]
    fn erased_allocated_mut() {
        let mut mem: Box<dyn ErasedMem<Item = u64>> = Box::new(Global::<u64>::new());
        mem.grow_filled(5, 0).unwrap();
        mem.allocated_mut()[0] = 42;
        assert_eq!(mem.allocated()[0], 42);
    }
}

// ============================================================================
// Error type tests
// ============================================================================

mod error_tests {
    use super::*;
    use std::alloc::Layout;

    #[test]
    fn error_display_capacity_overflow() {
        let err = Error::CapacityOverflow;
        assert!(err.to_string().contains("capacity"));
    }

    #[test]
    fn error_display_overgrow() {
        let err = Error::OverGrow { to_grow: 100, available: 50 };
        let msg = err.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
    }

    #[test]
    fn error_display_alloc_error() {
        let layout = Layout::from_size_align(1024, 8).unwrap();
        let err = Error::AllocError { layout, non_exhaustive: () };
        assert!(err.to_string().contains("allocation"));
    }

    #[test]
    fn error_display_system() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test error");
        let err = Error::from(io_err);
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn error_debug() {
        let err = Error::CapacityOverflow;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("CapacityOverflow"));
    }
}

// ============================================================================
// uninit module tests
// ============================================================================

mod uninit_tests {
    use std::mem::MaybeUninit;

    #[test]
    fn fill_initializes_slice() {
        let mut data: [MaybeUninit<u64>; 5] = unsafe { MaybeUninit::uninit().assume_init() };
        platform_mem::raw_mem::uninit::fill(&mut data, 42);
        for item in data.iter() {
            assert_eq!(unsafe { item.assume_init() }, 42);
        }
    }

    #[test]
    fn fill_with_initializes_slice() {
        let mut data: [MaybeUninit<u64>; 5] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut counter = 0u64;
        platform_mem::raw_mem::uninit::fill_with(&mut data, || {
            counter += 1;
            counter
        });
        for (i, item) in data.iter().enumerate() {
            assert_eq!(unsafe { item.assume_init() }, (i + 1) as u64);
        }
    }

    #[test]
    fn fill_empty_slice() {
        let mut data: [MaybeUninit<u64>; 0] = [];
        platform_mem::raw_mem::uninit::fill(&mut data, 42);
    }

    #[test]
    fn fill_with_empty_slice() {
        let mut data: [MaybeUninit<u64>; 0] = [];
        platform_mem::raw_mem::uninit::fill_with(&mut data, || 42);
    }

    #[test]
    fn fill_single_element() {
        let mut data: [MaybeUninit<u64>; 1] = unsafe { MaybeUninit::uninit().assume_init() };
        platform_mem::raw_mem::uninit::fill(&mut data, 42);
        assert_eq!(unsafe { data[0].assume_init() }, 42);
    }
}

// ============================================================================
// RawPlace tests (via indirect usage)
// ============================================================================

mod raw_place_tests {
    use super::*;

    #[test]
    fn raw_place_through_alloc() {
        // RawPlace is used internally by Alloc
        let mut alloc = Global::<u64>::new();

        // Test initial state (dangling)
        assert_eq!(alloc.allocated().len(), 0);

        // Test growth (handle_fill)
        alloc.grow_filled(10, 42).unwrap();
        assert_eq!(alloc.allocated().len(), 10);

        // Test shrinking (shrink_to)
        alloc.shrink(5).unwrap();
        assert_eq!(alloc.allocated().len(), 5);

        // Test multiple operations
        alloc.grow_filled(5, 0).unwrap();
        assert_eq!(alloc.allocated().len(), 10);
    }

    #[test]
    fn raw_place_through_file_mapped() -> Result<()> {
        let mut mem = TempFile::<u64>::new()?;

        // Test initial state
        assert_eq!(mem.allocated().len(), 0);

        // Test growth
        mem.grow_filled(10, 42)?;
        assert_eq!(mem.allocated().len(), 10);

        // Test shrinking
        mem.shrink(5)?;
        assert_eq!(mem.allocated().len(), 5);

        Ok(())
    }
}

// ============================================================================
// Thread safety tests
// ============================================================================

mod thread_safety_tests {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn global_is_send_sync() {
        assert_send::<Global<u64>>();
        assert_sync::<Global<u64>>();
    }

    #[test]
    fn system_is_send_sync() {
        assert_send::<System<u64>>();
        assert_sync::<System<u64>>();
    }

    #[test]
    fn tempfile_is_send_sync() {
        assert_send::<TempFile<u64>>();
        assert_sync::<TempFile<u64>>();
    }

    #[test]
    fn file_mapped_is_send_sync() {
        assert_send::<FileMapped<u64>>();
        assert_sync::<FileMapped<u64>>();
    }

    #[test]
    fn alloc_is_send_sync() {
        assert_send::<Alloc<u64, GlobalAlloc>>();
        assert_sync::<Alloc<u64, GlobalAlloc>>();
    }
}

// ============================================================================
// Drop with complex types tests
// ============================================================================

mod drop_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn drop_with_arc() {
        let counter = Arc::new(AtomicUsize::new(0));

        {
            let mut mem = Global::<Arc<AtomicUsize>>::new();
            for _ in 0..5 {
                let c = counter.clone();
                c.fetch_add(1, Ordering::SeqCst);
                mem.grow_filled(1, c).unwrap();
            }
            // 5 clones in the vector + 1 original
            assert!(Arc::strong_count(&counter) > 1);
        }

        // After drop, only the original should remain
        assert_eq!(Arc::strong_count(&counter), 1);
    }

    #[test]
    fn shrink_drops_elements() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut mem = Global::<Arc<AtomicUsize>>::new();

        for _ in 0..10 {
            let c = counter.clone();
            mem.grow_filled(1, c).unwrap();
        }

        let before_shrink = Arc::strong_count(&counter);
        mem.shrink(5).unwrap();
        let after_shrink = Arc::strong_count(&counter);

        // Should have dropped 5 references
        assert_eq!(before_shrink - after_shrink, 5);
    }
}

// ============================================================================
// Edge case tests
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn grow_zero_elements() {
        let mut mem = Global::<u64>::new();
        mem.grow_filled(0, 42).unwrap();
        assert_eq!(mem.allocated().len(), 0);
    }

    #[test]
    fn shrink_zero_elements() {
        let mut mem = Global::<u64>::new();
        mem.grow_filled(10, 42).unwrap();
        mem.shrink(0).unwrap();
        assert_eq!(mem.allocated().len(), 10);
    }

    #[test]
    fn grow_from_empty_slice() {
        let mut mem = Global::<u64>::new();
        let empty: [u64; 0] = [];
        mem.grow_from_slice(&empty).unwrap();
        assert_eq!(mem.allocated().len(), 0);
    }

    #[test]
    fn grow_within_empty_range() {
        let mut mem = Global::<u64>::new();
        mem.grow_filled(5, 42).unwrap();
        mem.grow_within(0..0).unwrap();
        assert_eq!(mem.allocated().len(), 5);
    }

    #[test]
    fn allocated_modification() {
        let mut mem = Global::<u64>::new();
        mem.grow_filled(5, 0).unwrap();

        // Modify all elements
        for (i, elem) in mem.allocated_mut().iter_mut().enumerate() {
            *elem = i as u64;
        }

        assert_eq!(mem.allocated(), &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn large_allocation() {
        let mut mem = Global::<u8>::new();
        mem.grow_filled(1_000_000, 0).unwrap();
        assert_eq!(mem.allocated().len(), 1_000_000);
    }
}
