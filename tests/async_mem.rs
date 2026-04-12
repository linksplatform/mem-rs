//! Tests for AsyncFileMem (moved from src/async_mem.rs per project conventions)

use platform_mem::AsyncFileMem;

#[tokio::test]
async fn test_async_file_mem_temp_and_grow() {
    let mem = AsyncFileMem::<u64>::temp().unwrap();
    assert!(mem.is_empty().await.unwrap());

    mem.grow_filled(10, 42).await.unwrap();
    assert_eq!(mem.len().await.unwrap(), 10);
    assert_eq!(mem.get(0).await.unwrap(), Some(42));
    assert_eq!(mem.get(9).await.unwrap(), Some(42));
    assert_eq!(mem.get(10).await.unwrap(), None);
}

#[tokio::test]
async fn test_async_file_mem_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("persist.bin");

    // Write data
    {
        let mut mem = AsyncFileMem::<u64>::from_path(&path).unwrap();
        mem.grow_filled(5, 123).await.unwrap();
        mem.set(2, 456).await.unwrap();
        mem.shutdown().await.unwrap();
    }

    // Read data back — use grow_assumed to map existing file data
    {
        let mem = AsyncFileMem::<u64>::from_path(&path).unwrap();
        unsafe { mem.grow_assumed(5).await.unwrap() };
        assert_eq!(mem.len().await.unwrap(), 5);
        assert_eq!(mem.get(0).await.unwrap(), Some(123));
        assert_eq!(mem.get(2).await.unwrap(), Some(456));
    }
}

#[tokio::test]
async fn test_async_file_mem_shrink() {
    let mem = AsyncFileMem::<u64>::temp().unwrap();
    mem.grow_filled(20, 1).await.unwrap();
    assert_eq!(mem.len().await.unwrap(), 20);

    mem.shrink(5).await.unwrap();
    assert_eq!(mem.len().await.unwrap(), 15);
}

#[tokio::test]
async fn test_async_file_mem_grow_zeroed() {
    let mem = AsyncFileMem::<u64>::temp().unwrap();
    unsafe {
        mem.grow_zeroed(10).await.unwrap();
    }
    assert_eq!(mem.len().await.unwrap(), 10);
    for i in 0..10 {
        assert_eq!(mem.get(i).await.unwrap(), Some(0));
    }
}

#[tokio::test]
async fn test_async_file_mem_read_write_slice() {
    let mem = AsyncFileMem::<u64>::temp().unwrap();
    mem.grow_filled(10, 0).await.unwrap();

    mem.write_slice(3, vec![10, 20, 30]).await.unwrap();

    let data = mem.read_slice(3, 3).await.unwrap();
    assert_eq!(data, vec![10, 20, 30]);

    assert_eq!(mem.get(2).await.unwrap(), Some(0));
    assert_eq!(mem.get(6).await.unwrap(), Some(0));
}

#[tokio::test]
async fn test_async_file_mem_set_get() {
    let mem = AsyncFileMem::<u64>::temp().unwrap();
    mem.grow_filled(5, 0).await.unwrap();

    assert!(mem.set(0, 100).await.unwrap());
    assert!(mem.set(4, 400).await.unwrap());
    assert!(!mem.set(5, 500).await.unwrap()); // out of bounds

    assert_eq!(mem.get(0).await.unwrap(), Some(100));
    assert_eq!(mem.get(4).await.unwrap(), Some(400));
}

#[tokio::test]
async fn test_async_file_mem_multiple_grows() {
    let mem = AsyncFileMem::<u64>::temp().unwrap();

    mem.grow_filled(5, 1).await.unwrap();
    mem.grow_filled(5, 2).await.unwrap();
    mem.grow_filled(5, 3).await.unwrap();

    assert_eq!(mem.len().await.unwrap(), 15);
    assert_eq!(mem.get(0).await.unwrap(), Some(1));
    assert_eq!(mem.get(5).await.unwrap(), Some(2));
    assert_eq!(mem.get(10).await.unwrap(), Some(3));
}

#[tokio::test]
async fn test_async_file_mem_sequential_operations() {
    let mem = AsyncFileMem::<u64>::temp().unwrap();
    mem.grow_filled(100, 0).await.unwrap();

    for i in 0..10u64 {
        mem.set(i as usize, i * 10).await.unwrap();
    }

    for i in 0..10u64 {
        assert_eq!(mem.get(i as usize).await.unwrap(), Some(i * 10));
    }
}
