#![feature(allocator_api)]

use platform_mem::{PreAlloc, RawMem};
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
    let prealloc: [String; 20] = (0..20)
        .map(|_| String::new())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

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
