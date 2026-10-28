//! Reproduce issue #36 with doublets 0.5.0 and the local platform-mem package.

use doublets::{
    Doublets,
    mem::FileMapped,
    unit::{LinkPart, Store},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("persistent.doublets");
    let (a, b, first, second) = {
        let mut store = Store::<u32, _>::new(FileMapped::<LinkPart<u32>>::from_path(&path)?)?;
        let a = store.create_link(0, 0)?;
        let b = store.create_link(a, a)?;
        assert_eq!(store.count(), 2);
        println!("wrote count={}", store.count());
        (a, b, store.get_link(a), store.get_link(b))
    };

    {
        let mut store = Store::<u32, _>::new(FileMapped::<LinkPart<u32>>::from_path(&path)?)?;
        println!("reopened count={}", store.count());
        assert_eq!(store.count(), 2);
        assert_eq!(store.get_link(a), first);
        assert_eq!(store.get_link(b), second);
        store.create_link(a, b)?;
        assert_eq!(store.count(), 3);
    }

    let store = Store::<u32, _>::new(FileMapped::<LinkPart<u32>>::from_path(&path)?)?;
    assert_eq!(store.count(), 3);
    assert_eq!(store.get_link(a), first);
    assert_eq!(store.get_link(b), second);
    println!("reopened after append count={}", store.count());
    Ok(())
}
