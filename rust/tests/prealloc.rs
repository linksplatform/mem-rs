#![feature(allocator_api)]

use platform_mem::{PreAlloc, RawMem};
use quickcheck_macros::quickcheck;
use std::error::Error;

#[test]
fn basic() -> Result<(), Box<dyn Error>> {
    let prealloc = [0usize; 20];

    let mut mem = PreAlloc::new(prealloc);
    let slice = mem.alloc(10)?;

    assert_eq!(slice.len(), 10);

    slice.iter_mut().enumerate().for_each(|(i, x)| *x = i);

    assert_eq!(slice, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let slice = mem.alloc(20)?;
    assert_eq!(slice.len(), 20);
    slice.iter_mut().enumerate().for_each(|(i, x)| *x = i);

    assert_eq!(
        slice,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19
        ]
    );
    Ok(())
}

#[test]
fn with_non_default_inner() -> Result<(), Box<dyn Error>> {
    // fixme: RFC #2920
    //  let prealloc = [String::new(); 20];
    let prealloc: Vec<_> = (0..20).map(|_| String::new()).collect();

    let mut mem = PreAlloc::new(prealloc);
    let slice = mem.alloc(10)?;
    assert_eq!(slice.len(), 10);

    slice
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = i.to_string());

    assert_eq!(slice, ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]);

    let slice = mem.alloc(20)?;
    assert_eq!(slice.len(), 20);
    slice
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = i.to_string());

    assert_eq!(
        slice,
        [
            "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
            "16", "17", "18", "19"
        ]
    );
    Ok(())
}

#[quickcheck]
fn valid_allocated_after_error(prealloc: Vec<usize>, capacity: usize) -> bool {
    let len = prealloc.len();
    let mut mem = PreAlloc::new(prealloc);
    let result = mem.alloc(capacity);

    (if capacity <= len {
        result.is_ok() && mem.allocated() == capacity
    } else {
        result.is_err() && mem.allocated() == 0
    }) && mem.size_hint() >= len
}
