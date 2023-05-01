#![feature(
    allocator_api,
    unchecked_math,
    maybe_uninit_slice,
    slice_ptr_get,
    ptr_as_uninit,
    inline_const,
    slice_range,
    maybe_uninit_write_slice
)]
// special lint
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
// rust compiler lints
#![deny(unused_must_use)]
#![warn(missing_debug_implementations)]

mod alloc;
mod file_mapped;
mod raw_mem;
mod raw_place;
mod utils;

pub(crate) use raw_place::RawPlace;
pub use {
    alloc::Alloc,
    file_mapped::FileMapped,
    raw_mem::{Error, RawMem, Result},
};

fn _assertion() {
    fn assert_sync_send<T: Sync + Send>() {}

    assert_sync_send::<FileMapped<()>>();
    assert_sync_send::<Alloc<(), std::alloc::Global>>();
}

macro_rules! delegate_memory {
    ($($me:ident<$param:ident>($inner:ty) { $($body:tt)* } )*) => {$(
        pub struct $me<$param>($inner);

        impl<$param> $me<$param> {
            $($body)*
        }

        const _: () = {
            use std::{
                mem::MaybeUninit,
                fmt::{self, Formatter},
            };

            impl<$param> RawMem for $me<$param> {
                type Item = $param;

                fn allocated(&self) -> &[Self::Item] {
                    self.0.allocated()
                }

                fn allocated_mut(&mut self) -> &mut [Self::Item] {
                    self.0.allocated_mut()
                }

                unsafe fn grow(
                    &mut self,
                    addition: usize,
                    fill: impl FnOnce(usize, (&mut [Self::Item], &mut [MaybeUninit<Self::Item>])),
                ) -> Result<&mut [Self::Item]> {
                    self.0.grow(addition, fill)
                }

                fn shrink(&mut self, cap: usize) -> Result<()> {
                    self.0.shrink(cap)
                }

                fn size_hint(&self) -> Option<usize> {
                    self.0.size_hint()
                }
            }

            impl<T> fmt::Debug for $me<$param> {
                fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                    f.debug_tuple(stringify!($me)).field(&self.0).finish()
                }
            }

        };
    )*};
}

use std::{
    alloc::{Global as GlobalAlloc, System as SystemAlloc},
    fs::File,
    io,
    path::Path,
};

delegate_memory! {
    Global<T>(Alloc<T, GlobalAlloc>) {
        pub fn new() -> Self {
            Self(Alloc::new(GlobalAlloc))
        }
    }
   System<T>(Alloc<T, SystemAlloc>) {
       pub fn new() -> Self {
           Self(Alloc::new(SystemAlloc))
       }
   }
   TempFile<T>(FileMapped<T>) {
       pub fn new() -> io::Result<Self> {
           Self::from_temp(tempfile::tempfile())
       }

       pub fn new_in<P: AsRef<Path>>(path: P) -> io::Result<Self> {
           Self::from_temp(tempfile::tempfile_in(path))
       }

       fn from_temp(file: io::Result<File>) -> io::Result<Self> {
           file.and_then(FileMapped::new).map(Self)
       }
   }
}

// fixme: add flag when it needs in macro
impl<T> Default for Global<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Default for System<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn miri() {
    pub fn inner<M: RawMem>(mut mem: M, val: M::Item) -> Result<()>
    where
        M::Item: Clone,
    {
        for _ in 0..10 {
            mem.grow_filled(10, val.clone())?;
        }
        assert_eq!(mem.allocated().len(), 100);

        for _ in 0..10 {
            mem.shrink(10)?;
        }
        assert_eq!(mem.allocated().len(), 0);

        Ok(())
    }

    let val = "foo".to_string();

    inner(Global::new(), val.clone()).unwrap();
    inner(System::new(), val.clone()).unwrap();

    #[cfg(not(miri))]
    inner(TempFile::new().unwrap(), val).unwrap();
}

#[test]
fn yet() -> Result<()> {
    use std::{fs, io::Write, str};

    // may be use tempfile???
    const FILE: &str = "tmp.file";
    const TAIL_SIZE: usize = 4 * 1024;

    let _ = fs::remove_file(FILE);
    {
        let mut file = File::options() // `create_new` feature
            .write(true)
            .create_new(true)
            .open(FILE)?;
        file.write_all(b"hello world")?;
        file.write_all(&[b'\0'; TAIL_SIZE])?;
    }

    unsafe {
        let mut mem = FileMapped::from_path(FILE)?;

        assert_eq!(b"hello world", mem.grow_assumed(5 + 1 + 5)?); // is size of `hello world`

        mem.grow(10_000, |inited, (_, uninit)| {
            assert_eq!(inited, TAIL_SIZE);
            assert_eq!(10_000, uninit.len());
        })?;
    }

    Ok(())
}
