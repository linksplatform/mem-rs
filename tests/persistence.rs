//! Regression tests for preserving file data through the default growth helpers.

use platform_mem::{ErasedMem, FileMapped, RawMem, Result};

#[test]
fn grow_filled_preserves_data_after_reopen() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("filled.bin");
    {
        let mut mem = FileMapped::<u32>::from_path(&path)?;
        mem.grow_filled(3, 42)?;
        mem.allocated_mut()[1] = 123;
    }

    let mut mem = FileMapped::<u32>::from_path(&path)?;
    assert_eq!(mem.grow_filled(3, 99)?, &[42, 123, 42]);
    Ok(())
}

#[test]
fn grow_zeroed_preserves_data_after_reopen() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("zeroed.bin");
    {
        let mut mem = FileMapped::<u32>::from_path(&path)?;
        mem.grow_from_slice(&[42, 123, 42])?;
    }

    let mut mem = FileMapped::<u32>::from_path(&path)?;
    // SAFETY: every u32 bit pattern, including zero, is valid.
    assert_eq!(unsafe { mem.grow_zeroed(3)? }, &[42, 123, 42]);
    Ok(())
}

#[test]
fn grow_filled_initializes_only_the_new_tail() -> Result<()> {
    // Include a short file extended by FileMapped::new and files spanning a page.
    for count in [3, 1024, 1025] {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("filled-tail.bin");
        let bytes = 42u32.to_ne_bytes().repeat(count);
        std::fs::write(&path, bytes)?;

        let mut mem = FileMapped::<u32>::from_path(&path)?;
        let grown = mem.grow_filled(count + 5, 99)?;
        assert_eq!(&grown[..count], vec![42; count]);
        assert_eq!(&grown[count..], &[99; 5]);
    }
    Ok(())
}

#[test]
fn grow_zeroed_initializes_only_the_new_tail() -> Result<()> {
    for count in [3, 1024, 1025] {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("zeroed-tail.bin");
        std::fs::write(&path, 42u32.to_ne_bytes().repeat(count))?;

        let mut mem = FileMapped::<u32>::from_path(&path)?;
        // SAFETY: u32 permits zero initialization.
        let grown = unsafe { mem.grow_zeroed(count + 5)? };
        assert_eq!(&grown[..count], vec![42; count]);
        assert_eq!(&grown[count..], &[0; 5]);
    }
    Ok(())
}

#[test]
fn repeated_growth_preserves_the_remaining_file_prefix() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("repeated.bin");
    {
        let mut mem = FileMapped::<u32>::from_path(&path)?;
        mem.grow_filled(1024, 42)?;
    }

    let mut mem = FileMapped::<u32>::from_path(&path)?;
    assert_eq!(mem.grow_filled(400, 99)?, &[42; 400]);
    let grown = mem.grow_filled(629, 99)?;
    assert_eq!(&grown[..624], &[42; 624]);
    assert_eq!(&grown[624..], &[99; 5]);
    assert_eq!(&mem.allocated()[..1024], &[42; 1024]);
    Ok(())
}

#[test]
fn fresh_file_growth_fills_padding_and_extensions() -> Result<()> {
    let mut mem = FileMapped::<u32>::new(tempfile::tempfile()?)?;
    assert_eq!(mem.grow_filled(3, 42)?, &[42; 3]);
    assert_eq!(mem.grow_filled(2, 99)?, &[99; 2]);
    assert_eq!(mem.grow_filled(1024, 123)?, &[123; 1024]);
    assert_eq!(&mem.allocated()[..5], &[42, 42, 42, 99, 99]);
    Ok(())
}

#[test]
fn zero_growth_preserves_data() -> Result<()> {
    let mut mem = FileMapped::<u32>::new(tempfile::tempfile()?)?;
    mem.grow_filled(3, 42)?;
    assert_eq!(mem.grow_filled(0, 99)?, []);
    // SAFETY: u32 permits zero initialization.
    assert_eq!(unsafe { mem.grow_zeroed(0)? }, []);
    assert_eq!(mem.allocated(), &[42; 3]);
    Ok(())
}

#[test]
fn grow_with_only_calls_the_initializer_for_new_elements() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("with.bin");
    std::fs::write(&path, 42u32.to_ne_bytes().repeat(3))?;

    let mut mem = FileMapped::<u32>::from_path(&path)?;
    let mut calls = 0;
    assert_eq!(mem.grow_with(2, || panic!("persisted elements must be kept"))?, &[42; 2]);
    let grown = mem.grow_with(4, || {
        calls += 1;
        calls
    })?;
    assert_eq!(grown, &[42, 1, 2, 3]);
    assert_eq!(calls, 3);
    Ok(())
}

#[test]
fn erased_growth_preserves_file_data() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("erased.bin");
    {
        let mut mem = FileMapped::<u32>::from_path(&path)?;
        mem.grow_from_slice(&[42, 123, 42])?;
    }

    let mut mem: Box<dyn ErasedMem<Item = u32>> = Box::new(FileMapped::from_path(&path)?);
    assert_eq!(mem.grow_filled(2, 99)?, &[42, 123]);
    // SAFETY: u32 permits zero initialization.
    assert_eq!(unsafe { mem.grow_zeroed(1)? }, &[42]);
    Ok(())
}

#[test]
fn growth_after_shrink_initializes_the_truncated_tail() -> Result<()> {
    let mut mem = FileMapped::<u32>::new(tempfile::tempfile()?)?;
    mem.grow_filled(1024, 42)?;
    mem.shrink(1021)?;
    assert_eq!(mem.grow_filled(5, 99)?, &[99; 5]);
    assert_eq!(mem.allocated(), &[42, 42, 42, 99, 99, 99, 99, 99]);
    Ok(())
}

#[test]
fn partial_file_element_is_initialized_as_a_whole() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("partial.bin");
    let mut bytes = 42u32.to_ne_bytes().to_vec();
    bytes.push(255);
    std::fs::write(&path, bytes)?;

    let mut mem = FileMapped::<u32>::from_path(&path)?;
    assert_eq!(mem.grow_filled(3, 99)?, &[42, 99, 99]);
    Ok(())
}
