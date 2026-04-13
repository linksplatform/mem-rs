//! Example demonstrating async memory operations with a dedicated I/O thread.
//!
//! Run with: cargo run --example async_example --features async

use platform_mem::AsyncFileMem;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Async Memory Example ===\n");

    // Example 1: Basic async memory operations (mmap-backed via I/O thread)
    println!("1. Basic async memory operations:");
    let mem = AsyncFileMem::<u64>::temp()?;

    // Grow with filled values
    mem.grow_filled(10, 42).await?;
    println!("   Allocated 10 elements filled with 42");
    println!("   Length: {}", mem.len().await?);
    let data = mem.read_slice(0, 10).await?;
    println!("   Values: {:?}", data);

    // Example 2: Modify data
    println!("\n2. Modifying data:");
    mem.set(0, 100).await?;
    mem.set(5, 500).await?;
    let data = mem.read_slice(0, 10).await?;
    println!("   After setting index 0 to 100 and index 5 to 500:");
    println!("   Values: {:?}", data);

    // Example 3: Grow with zeros
    println!("\n3. Growing with zeros:");
    unsafe {
        mem.grow_zeroed(5).await?;
    }
    println!("   After growing by 5 zeroed elements:");
    println!("   Length: {}", mem.len().await?);

    // Example 4: Shrink
    println!("\n4. Shrinking:");
    mem.shrink(7).await?;
    println!("   After shrinking by 7 elements:");
    println!("   Length: {}", mem.len().await?);
    let data = mem.read_slice(0, mem.len().await?).await?;
    println!("   Values: {:?}", data);

    // Example 5: Persistent storage
    println!("\n5. Persistent storage example:");
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("async_mem_example.bin");

    // Write data
    {
        let mut persistent = AsyncFileMem::<u64>::from_path(&file_path)?;
        persistent.grow_filled(5, 0).await?;
        persistent.write_slice(0, vec![1, 2, 3, 4, 5]).await?;
        let data = persistent.read_slice(0, 5).await?;
        println!("   Wrote: {:?}", data);
        persistent.shutdown().await?;
        println!("   Synced to disk: {}", file_path.display());
    }

    // Read data back (mmap sees persisted data via grow_assumed)
    {
        let persistent = AsyncFileMem::<u64>::from_path(&file_path)?;
        let len = persistent.len().await?;
        let data = persistent.read_slice(0, len).await?;
        println!("   Read back: {:?}", data);
    }

    // Clean up
    std::fs::remove_file(&file_path)?;
    println!("   Cleaned up temp file");

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
