#![feature(
    allocator_api,
    unchecked_math,
    maybe_uninit_slice,
    slice_ptr_get,
    ptr_as_uninit,
    inline_const,
    min_specialization
)]
// special lint
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
// rust compiler lints
#![deny(unused_must_use)]
#![warn(missing_debug_implementations)]

mod alloc;
mod file_mapped;
mod prealloc;
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
                    fill: impl FnOnce(&mut [MaybeUninit<Self::Item>]),
                ) -> Result<&mut [Self::Item]> {
                    self.0.grow(addition, fill)
                }

                fn shrink(&mut self, cap: usize) -> Result<()> {
                    self.0.shrink(cap)
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
        pub const fn new() -> Self {
            Self(Alloc::new(GlobalAlloc))
        }
    }
   System<T>(Alloc<T, SystemAlloc>) {
       pub const fn new() -> Self {
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
        M::Item: Clone + PartialEq,
    {
        const GROW: usize = if cfg!(miri) { 100 } else { 10_000 };

        for _ in 0..10 {
            mem.grow_filled(GROW, val.clone())?;
        }
        assert!(mem.allocated() == vec![val; GROW * 10]);

        for _ in 0..10 {
            mem.shrink(GROW)?;
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

#[cfg(test)]
macro_rules! define_impls {
    (impl RawMem: {
        $($ctor:expr $(=> in $cfg:meta)? ),* $(,)?
    } for [ $($test:ident),* $(,)? ]
    ) => {
        mod generic_tests {$(
            #[cfg_attr(all(test, $($cfg)?), test)]
            fn $test() {
                use super::*;
                super::$test(
                    $ctor
                );
            }
        )*}
    };
}

#[cfg(test)]
define_impls! {
    impl RawMem: {
        Global::new(),
        System::new(),
        TempFile::new().unwrap() => in not(miri),
    } for [
        grow_with, shrink, allocated,
    ]
}

fn grow_with(mut mem: impl RawMem<Item = (u8, u16)>) {
    let res = mem.grow_with(10, || (2, 2)).expect("grow");
    assert_eq!(res, [(2, 2); 10]);
}

fn allocated(mut mem: impl RawMem<Item = (u8, u16)>) {
    assert_eq!(mem.allocated().len(), 0);

    let res = mem.grow_with(10, || (2, 2)).expect("grow");
    assert_eq!(res, [(2, 2); 10]);

    assert_eq!(mem.allocated().len(), 10);
}

fn shrink(mut mem: impl RawMem<Item = (u8, u16)>) {
    let res = mem.grow_with(10, || (2, 2)).expect("grow");
    assert_eq!(res, [(2, 2); 10]);

    mem.shrink(5).expect("shrink");
    assert_eq!(mem.allocated(), &[(2, 2); 5]);
}
