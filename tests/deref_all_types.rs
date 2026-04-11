use platform_mem::{Alloc, Global, RawMem, Result, System, TempFile};

#[test]
fn global_deref_indexing() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(mem[0], 1);
    assert_eq!(&mem[1..4], &[2, 3, 4]);
    assert_eq!(mem.len(), 5);
}

#[test]
fn system_deref_indexing() {
    let mut mem = System::<u32>::new();
    mem.grow_from_slice(&[100, 200, 300]).unwrap();
    assert_eq!(mem[0], 100);
    assert_eq!(&mem[1..], &[200, 300]);
}

#[test]
fn tempfile_deref_indexing() {
    let mut mem = TempFile::<u32>::new().unwrap();
    mem.grow_from_slice(&[10, 20, 30, 40]).unwrap();
    assert_eq!(mem[0], 10);
    assert_eq!(&mem[1..3], &[20, 30]);
    assert_eq!(mem.len(), 4);
}

#[test]
fn alloc_deref_indexing() {
    let mut alloc: Alloc<u64, allocator_api2::alloc::Global> =
        Alloc::new(allocator_api2::alloc::Global);
    alloc.grow_from_slice(&[5, 10, 15]).unwrap();
    assert_eq!(alloc[0], 5);
    assert_eq!(&alloc[..2], &[5, 10]);
}

#[test]
fn file_mapped_deref_indexing() -> Result<()> {
    let mut mem = TempFile::<u64>::new()?;
    mem.grow_from_slice(&[7, 8, 9]).unwrap();
    assert_eq!(mem[2], 9);
    assert_eq!(&mem[..], &[7, 8, 9]);
    Ok(())
}
