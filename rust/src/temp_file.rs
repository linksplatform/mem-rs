use crate::{FileMapped, IsTrue, RawMem, Result};
use std::{fs::File, io, mem::size_of, path::Path};

#[repr(transparent)]
pub struct TempFile<T>(FileMapped<T>);

impl<T: Default> TempFile<T> {
    pub fn new() -> io::Result<Self> {
        Self::from_file(tempfile::tempfile())
    }

    pub fn new_in<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::from_file(tempfile::tempfile_in(path))
    }

    fn from_file(file: io::Result<File>) -> io::Result<Self> {
        file.and_then(FileMapped::new).map(Self)
    }
}

impl<T: Default> RawMem<T> for TempFile<T>
where
    (): IsTrue<{ size_of::<T>() != 0 }>,
{
    fn alloc(&mut self, capacity: usize) -> Result<&mut [T]> {
        self.0.alloc(capacity)
    }

    fn allocated(&self) -> usize {
        self.0.allocated()
    }

    // fixme: delegate all functions from `FileMapped`
}
