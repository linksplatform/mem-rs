use platform_mem::{Global, RawMem};

#[test]
fn deref_returns_slice() {
    let mut mem = Global::<u64>::new();
    mem.grow_filled(5, 42).unwrap();
    let slice: &[u64] = &*mem;
    assert_eq!(slice, &[42; 5]);
}

#[test]
fn deref_mut_returns_slice() {
    let mut mem = Global::<u64>::new();
    mem.grow_filled(5, 0).unwrap();
    let slice: &mut [u64] = &mut *mem;
    slice[0] = 99;
    assert_eq!(mem[0], 99);
}

#[test]
fn index_single_element() {
    let mut mem = Global::<u64>::new();
    mem.grow_filled(3, 0).unwrap();
    mem[0] = 10;
    mem[1] = 20;
    mem[2] = 30;
    assert_eq!(mem[0], 10);
    assert_eq!(mem[1], 20);
    assert_eq!(mem[2], 30);
}

#[test]
fn index_range() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(&mem[1..4], &[2, 3, 4]);
}

#[test]
fn index_range_from() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(&mem[2..], &[3, 4, 5]);
}

#[test]
fn index_range_to() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(&mem[..3], &[1, 2, 3]);
}

#[test]
fn index_range_full() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(&mem[..], &[1, 2, 3, 4, 5]);
}

#[test]
fn index_range_inclusive() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(&mem[1..=3], &[2, 3, 4]);
}

#[test]
fn empty_deref() {
    let mem = Global::<u64>::new();
    let slice: &[u64] = &*mem;
    assert!(slice.is_empty());
}
