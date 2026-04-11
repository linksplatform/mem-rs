use platform_mem::{Global, RawMem};

fn main() -> Result<(), platform_mem::Error> {
    let mut mem = Global::<u64>::new();

    // Grow memory and fill with value
    mem.grow_filled(10, 42)?;
    assert_eq!(mem.allocated(), &[42u64; 10]);

    // Grow more from a slice
    mem.grow_from_slice(&[1, 2, 3])?;
    assert_eq!(mem.allocated().len(), 13);

    // Shrink by 5 elements
    mem.shrink(5)?;
    assert_eq!(mem.allocated().len(), 8);

    println!("Example 1 passed: Basic memory operations work!");
    Ok(())
}
