use platform_mem::{Global, RawMem};

#[test]
fn deref_consistency_with_allocated() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[1, 2, 3, 4, 5]).unwrap();

    // Deref and allocated() should return the same data
    let deref_slice: &[u64] = &*mem;
    let allocated_slice: &[u64] = mem.allocated();
    assert_eq!(deref_slice, allocated_slice);
}

#[test]
fn deref_mut_consistency_with_allocated_mut() {
    let mut mem = Global::<u64>::new();
    mem.grow_filled(5, 0).unwrap();

    // Modify through deref_mut
    mem[0] = 42;
    // Read through allocated
    assert_eq!(mem.allocated()[0], 42);

    // Modify through allocated_mut
    mem.allocated_mut()[1] = 99;
    // Read through deref
    assert_eq!((*mem)[1], 99);
}

#[test]
fn deref_after_grow_and_shrink() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(&mem[..], &[1, 2, 3, 4, 5]);

    mem.shrink(2).unwrap();
    assert_eq!(&mem[..], &[1, 2, 3]);

    mem.grow_filled(2, 99).unwrap();
    assert_eq!(&mem[..], &[1, 2, 3, 99, 99]);
}
