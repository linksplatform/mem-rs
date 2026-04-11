use platform_mem::{Global, RawMem};

#[test]
fn len_via_deref() {
    let mut mem = Global::<u64>::new();
    mem.grow_filled(10, 0).unwrap();
    assert_eq!(mem.len(), 10);
}

#[test]
fn is_empty_via_deref() {
    let mem = Global::<u64>::new();
    assert!(mem.is_empty());
}

#[test]
fn iter_via_deref() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[1, 2, 3]).unwrap();
    let sum: u64 = mem.iter().sum();
    assert_eq!(sum, 6);
}

#[test]
fn iter_mut_via_deref() {
    let mut mem = Global::<u64>::new();
    mem.grow_filled(3, 1).unwrap();
    for val in mem.iter_mut() {
        *val *= 10;
    }
    assert_eq!(&*mem, &[10, 10, 10]);
}

#[test]
fn contains_via_deref() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[10, 20, 30]).unwrap();
    assert!(mem.contains(&20));
    assert!(!mem.contains(&15));
}

#[test]
fn sort_via_deref_mut() {
    let mut mem = Global::<u64>::new();
    mem.grow_from_slice(&[3, 1, 4, 1, 5, 9, 2, 6]).unwrap();
    mem.sort();
    assert_eq!(&mem[..], &[1, 1, 2, 3, 4, 5, 6, 9]);
}
