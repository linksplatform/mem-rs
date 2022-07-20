#![feature(allocator_api)]

mod internal;

use platform_mem::{RawMem, Result};

fn basic_impl(mut mem: impl RawMem<usize>) -> Result<()> {
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

fn non_default_inner_impl(mut mem: impl RawMem<String>) -> Result<()> {
    let slice = mem.alloc(10)?;

    slice
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = i.to_string());

    assert_eq!(slice, ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]);

    let slice = mem.alloc(20)?;
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

test_for_all_mem!(basic, basic_impl);
test_for_all_mem!(non_default_inner, non_default_inner_impl);
