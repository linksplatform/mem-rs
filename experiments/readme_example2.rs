use platform_mem::{TempFile, RawMem};

fn main() -> Result<(), platform_mem::Error> {
    // Anonymous temporary file - cleaned up on drop
    let mut mem = TempFile::<u8>::new()?;

    mem.grow_from_slice(b"hello world")?;
    assert_eq!(mem.allocated(), b"hello world");

    println!("Example 2 passed: TempFile storage works!");
    Ok(())
}
