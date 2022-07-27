#![feature(allocator_api)]
#![feature(thread_local)]

mod internal;

use platform_mem::{Alloc, Global, RawMem, Result};
use std::alloc;

static mut DROP_COUNT: usize = 0;

// Not allowed zero size types
#[derive(Default)]
struct DropCounter(usize);

impl Drop for DropCounter {
    fn drop(&mut self) {
        unsafe {
            DROP_COUNT += 1;
        }
    }
}

fn shrink_drop_impl(mut mem: impl RawMem<DropCounter>) -> Result<()> {
    let _ = mem.alloc(20)?;
    let _ = mem.shrink(5)?;

    unsafe {
        assert_eq!(DROP_COUNT, 5);
        drop(mem);
        DROP_COUNT = 0;
    }

    Ok(())
}

#[test]
fn shrink_drop() {
    shrink_drop_impl(Global::new()).unwrap();
    shrink_drop_impl(Alloc::new(alloc::Global)).unwrap();
    #[cfg(not(miri))]
    shrink_drop_impl(platform_mem::TempFile::new().unwrap()).unwrap();
}
