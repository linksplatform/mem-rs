#![feature(allocator_api)]

use platform_mem::RawMem;

fn process_data<M: RawMem<Item = u32>>(mem: &mut M) -> Result<(), platform_mem::Error> {
    mem.grow_filled(100, 0)?;

    for (i, slot) in mem.allocated_mut().iter_mut().enumerate() {
        *slot = i as u32;
    }

    Ok(())
}

fn main() -> Result<(), platform_mem::Error> {
    let mut mem = platform_mem::Global::<u32>::new();
    process_data(&mut mem)?;

    // Verify the data
    for (i, slot) in mem.allocated().iter().enumerate() {
        assert_eq!(*slot, i as u32);
    }

    println!("Example 3 passed: Generic code with RawMem works!");
    Ok(())
}
