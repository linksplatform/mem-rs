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
use std::mem::MaybeUninit;
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
        $($ctor:expr /* -- */ $(=> in $cfg:meta)? ),+ $(,)?
    } for [
        $($test:path as $name:ident),* $(,)?
    ]) => {
        define_impls! { @loop
            [/* empty result */]
            [ $($ctor $(=> $cfg)? )*]
            [ $($test as $name |)* ]
        }
    };

   (@loop [ $($result:tt)* ] // result accumulation
           [ $($ctor:expr $(=> $cfg:meta)? )* ] // each ctor with our cfg `not(miri)`
           [ $test:path as $name:ident | $($tail:tt)* ] // match test with name + tail
    ) => {
        define_impls! { @loop
            [
                $($result)*

                #[test]
                fn $name() {
                    $( $(#[cfg($cfg)])? $test($ctor);)*
                }
            ]
            [$($ctor $(=> $cfg)? )*]
            [ $($tail)* ]
        }
    };

    (@loop [ $($result:tt)* ] [ $($_:tt)* ] [ /* tests still coming */ ] ) => {
        $($result)*
    };
}

#[cfg(test)]
define_impls! {
    impl RawMem: {
        Global::<u32>::new(),
        System::<u32>::new(),
        TempFile::<u32>::new().unwrap() => in not(miri),
    } for [
        grow as grow_test,
        grow_with as grow_with_test,
        allocated as allocated_test,
        shrink as shrink_test,
    ]
}

fn grow<T>(mut mem: impl RawMem<Item = T>) {
    unsafe {
        mem.grow(10, |_uninit| {}).expect("error");
    }
    assert!(mem.allocated().len() == 10);
}

fn grow_with<T: Copy + From<u8>>(mut mem: impl RawMem<Item = T>) {
    let value = T::from(2);
    mem.grow_with(10, || value).expect("grow");
    assert!(mem.allocated().len() == 10);
}

fn allocated<T: Copy + From<u8>>(mut mem: impl RawMem<Item = T>) {
    assert_eq!(mem.allocated().len(), 0);
    let value = T::from(2);
    mem.grow_with(10, || value).expect("grow");
    assert!(mem.allocated().len() == 10);
}

fn shrink<T: Copy + From<u8>>(mut mem: impl RawMem<Item = T>) {
    let value = T::from(2);
    mem.grow_with(10, || value).expect("grow");
    assert!(mem.allocated().len() == 10);

    mem.shrink(5).expect("shrink");
    assert!(mem.allocated().len() == 5);
}
