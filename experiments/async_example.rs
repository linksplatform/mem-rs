//! Example demonstrating async memory operations.
//!
//! Run with: cargo run --example async_example --features async

use platform_mem::AsyncFileMem;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Async Memory Example ===\n");

    // Example 1: Basic async memory operations
    println!("1. Basic async memory operations:");
    let mut mem = AsyncFileMem::<u64>::temp().await?;

    // Grow with filled values
    mem.grow_filled(10, 42).await?;
    println!("   Allocated 10 elements filled with 42");
    println!("   Length: {}", mem.len());
    println!("   Values: {:?}", mem.as_slice());

    // Example 2: Modify data
    println!("\n2. Modifying data:");
    mem.set(0, 100);
    mem.set(5, 500);
    println!("   After setting index 0 to 100 and index 5 to 500:");
    println!("   Values: {:?}", mem.as_slice());

    // Example 3: Grow with zeros
    println!("\n3. Growing with zeros:");
    unsafe {
        mem.grow_zeroed(5).await?;
    }
    println!("   After growing by 5 zeroed elements:");
    println!("   Length: {}", mem.len());
    println!("   Values: {:?}", mem.as_slice());

    // Example 4: Shrink
    println!("\n4. Shrinking:");
    mem.shrink(7).await?;
    println!("   After shrinking by 7 elements:");
    println!("   Length: {}", mem.len());
    println!("   Values: {:?}", mem.as_slice());

    // Example 5: Persistent storage
    println!("\n5. Persistent storage example:");
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("async_mem_example.bin");

    // Write data
    {
        let mut persistent = AsyncFileMem::<u64>::create(&file_path).await?;
        persistent.grow_from_slice(&[1, 2, 3, 4, 5]).await?;
        println!("   Wrote: {:?}", persistent.as_slice());
        persistent.sync().await?;
        println!("   Synced to disk: {:?}", file_path);
    }

    // Read data back
    {
        let persistent = AsyncFileMem::<u64>::open(&file_path).await?;
        println!("   Read back: {:?}", persistent.as_slice());
    }

    // Clean up
    std::fs::remove_file(&file_path)?;
    println!("   Cleaned up temp file");

    // Example 6: Concurrent access demonstration
    println!("\n6. Concurrent async operations:");
    let handles: Vec<_> = (0..5)
        .map(|i| {
            tokio::spawn(async move {
                let mut mem = AsyncFileMem::<u32>::temp().await.unwrap();
                mem.grow_filled(100, i).await.unwrap();
                mem.len()
            })
        })
        .collect();

    for (i, handle) in handles.into_iter().enumerate() {
        let len = handle.await?;
        println!("   Task {} created {} elements", i, len);
    }

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
