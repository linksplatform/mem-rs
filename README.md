# platform-mem

[![Crates.io](https://img.shields.io/crates/v/platform-mem.svg)](https://crates.io/crates/platform-mem)
[![License](https://img.shields.io/crates/l/platform-mem.svg)](LICENSE)

A Rust library for low-level memory management with unified interface for allocator-backed and memory-mapped file storage.

## Overview

`platform-mem` provides the `RawMem` trait that abstracts over different memory backends:

- **Allocator-based memory** (`Global`, `System`, `Alloc<T, A>`) - uses Rust's allocator API
- **Memory-mapped files** (`FileMapped`, `TempFile`) - uses `mmap` for persistent or temporary file-backed storage

This allows writing generic code that works with any memory backend, making it easy to switch between heap allocation and file-mapped storage.

## Features

- **Unified `RawMem` trait** - common interface for growing, shrinking, and accessing memory
- **Type-erased memory** via `ErasedMem` - enables dynamic dispatch with `Box<dyn ErasedMem<Item = T>>`
- **Memory-mapped files** - persistent storage with automatic page management
- **Temporary file storage** - anonymous file-backed memory that's cleaned up on drop
- **Safe growth operations** - `grow_filled`, `grow_zeroed`, `grow_from_slice`, and more
- **Thread-safe** - all memory types implement `Send + Sync`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
platform-mem = "0.1"
```

**Note:** This crate requires nightly Rust for the `allocator_api` feature.

```bash
rustup override set nightly
```

## Usage

### Basic Example with Global Allocator

```rust,ignore
#![feature(allocator_api)]

use platform_mem::{Global, RawMem};

fn main() -> Result<(), platform_mem::Error> {
    let mut mem = Global::<u64>::new();

    // Grow memory and fill with value
    mem.grow_filled(10, 42)?;
    assert_eq!(mem.allocated(), &[42u64; 10]);

    // Grow more from a slice
    mem.grow_from_slice(&[1, 2, 3])?;
    assert_eq!(mem.allocated().len(), 13);

    // Shrink by 5 elements
    mem.shrink(5)?;
    assert_eq!(mem.allocated().len(), 8);

    Ok(())
}
```

### Memory-Mapped File Storage

```rust,ignore
#![feature(allocator_api)]

use platform_mem::{FileMapped, RawMem};

fn main() -> Result<(), platform_mem::Error> {
    // Create memory mapped to a file
    let mut mem = FileMapped::<u64>::from_path("data.bin")?;

    // Data persists across program runs
    unsafe {
        mem.grow_zeroed(1000)?;
    }

    // Modify the memory
    mem.allocated_mut()[0] = 123;

    Ok(())
}
```

### Temporary File Storage

```rust,ignore
#![feature(allocator_api)]

use platform_mem::{TempFile, RawMem};

fn main() -> Result<(), platform_mem::Error> {
    // Anonymous temporary file - cleaned up on drop
    let mut mem = TempFile::<u8>::new()?;

    mem.grow_from_slice(b"hello world")?;
    assert_eq!(mem.allocated(), b"hello world");

    Ok(())
}
```

### Generic Code with `RawMem`

```rust,ignore
#![feature(allocator_api)]

use platform_mem::RawMem;

fn process_data<M: RawMem<Item = u32>>(mem: &mut M) -> Result<(), platform_mem::Error> {
    mem.grow_filled(100, 0)?;

    for (i, slot) in mem.allocated_mut().iter_mut().enumerate() {
        *slot = i as u32;
    }

    Ok(())
}
```

### Type-Erased Memory with `ErasedMem`

```rust,ignore
#![feature(allocator_api)]

use platform_mem::{ErasedMem, Global, RawMem};

fn main() {
    // Use dynamic dispatch when the memory type isn't known at compile time
    let mem: Box<dyn ErasedMem<Item = u64> + Send + Sync> =
        Box::new(Global::<u64>::new());
}
```

## API Overview

### `RawMem` Trait

The core trait providing memory operations:

| Method | Description |
|--------|-------------|
| `allocated()` | Returns a slice of the initialized memory |
| `allocated_mut()` | Returns a mutable slice of the initialized memory |
| `grow(addition, fill)` | Grows memory by `addition` elements with custom initialization |
| `shrink(cap)` | Shrinks memory by `cap` elements |
| `grow_filled(cap, value)` | Grows and fills with cloned values |
| `grow_zeroed(cap)` | Grows and zero-initializes (unsafe for non-zeroable types) |
| `grow_from_slice(src)` | Grows and copies from a slice |
| `grow_with(addition, f)` | Grows and initializes with a closure |

### Memory Types

| Type | Description |
|------|-------------|
| `Global<T>` | Uses Rust's global allocator |
| `System<T>` | Uses the system allocator |
| `Alloc<T, A>` | Generic over any `Allocator` |
| `FileMapped<T>` | Memory-mapped file storage |
| `TempFile<T>` | Temporary file-backed memory |

## Error Handling

The crate defines an `Error` enum with these variants:

- `CapacityOverflow` - Requested capacity exceeds `isize::MAX` bytes
- `OverGrow` - Tried to grow more than available space
- `AllocError` - Allocator failed to allocate/reallocate
- `System` - I/O error from file operations

## License

This project is released into the public domain under the [Unlicense](LICENSE).

## Related Projects

- [doublets-rs](https://github.com/linksplatform/doublets-rs) - Doublet links data structure using this memory library
- [LinksPlatform](https://github.com/linksplatform) - The Links Platform organization
